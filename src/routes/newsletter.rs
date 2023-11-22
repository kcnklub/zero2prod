use actix_web::http::header::{self, HeaderMap, HeaderValue};
use actix_web::http::StatusCode;
use actix_web::HttpRequest;
use actix_web::{web, HttpResponse, ResponseError};
use anyhow::Context;
use argon2::Argon2;
use argon2::PasswordHash;
use argon2::PasswordVerifier;
use base64::Engine;
use secrecy::{ExposeSecret, Secret};
use sqlx::PgPool;

use crate::domain::SubscriberEmail;
use crate::email_client::EmailClient;
use crate::telemetry::spawn_blocking_with_tracing;

use super::error_chain_fmt;

#[derive(serde::Deserialize)]
pub struct BodyData {
    title: String,
    content: Content,
}

#[derive(serde::Deserialize)]
pub struct Content {
    html: String,
    text: String,
}

struct ConfirmedSubscriber {
    email: SubscriberEmail,
}

#[derive(thiserror::Error)]
pub enum PublishError {
    #[error(transparent)]
    UnexpectedError(#[from] anyhow::Error),
    #[error("Authorization error")]
    AuthorizationError(#[source] anyhow::Error),
}

impl std::fmt::Debug for PublishError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        error_chain_fmt(self, f)
    }
}

impl ResponseError for PublishError {
    fn error_response(&self) -> HttpResponse {
        match self {
            PublishError::UnexpectedError(_) => HttpResponse::InternalServerError().finish(),
            PublishError::AuthorizationError(_) => {
                let mut response = HttpResponse::new(StatusCode::UNAUTHORIZED);
                let header_value = "Basic realm=\"publish\"";
                response.headers_mut().insert(
                    header::WWW_AUTHENTICATE,
                    HeaderValue::from_str(header_value).unwrap(),
                );
                response
            }
        }
    }
}

#[tracing::instrument(
    name = "Publish a newsletter issue",
    skip(_body, pool, email_client, http_request),
    fields(
        username = tracing::field::Empty, user_id = tracing::field::Empty,
    )
)]
pub async fn newsletter(
    _body: web::Json<BodyData>,
    pool: web::Data<PgPool>,
    email_client: web::Data<EmailClient>,
    http_request: HttpRequest,
) -> Result<HttpResponse, PublishError> {
    let creds =
        basic_authentication(http_request.headers()).map_err(PublishError::AuthorizationError)?;
    let user_id = validate_credentials(&pool, creds).await?;
    tracing::Span::current().record("user_id", &tracing::field::display(user_id));
    let confirmed_subscribers = gets_confirmed_subscriber(&pool).await?;
    for confirmed_subscriber in confirmed_subscribers {
        match confirmed_subscriber {
            Ok(confirmed_subscriber) => {
                let _ = email_client
                    .send_email(
                        &confirmed_subscriber.email,
                        "Newsletter",
                        "HTML content",
                        "Newsletter content",
                    )
                    .await
                    .with_context(|| {
                        format!(
                            "Failed to send newsletter to {:?}",
                            confirmed_subscriber.email
                        )
                    });
            }
            Err(error) => {
                tracing::warn!("Failed to retrieve confirmed subscriber: {:?}", error);
            }
        }
    }
    Ok(HttpResponse::Ok().finish())
}

#[tracing::instrument(name = "Get confirmed subscribers", skip(pool))]
async fn gets_confirmed_subscriber(
    pool: &PgPool,
) -> Result<Vec<Result<ConfirmedSubscriber, anyhow::Error>>, anyhow::Error> {
    let rows = sqlx::query!(
        r#"
        SELECT email
        FROM subscriptions
        WHERE status = 'confirmed'
        "#,
    )
    .fetch_all(pool)
    .await?
    .into_iter()
    .map(|row| match SubscriberEmail::parse(row.email) {
        Ok(email) => Ok(ConfirmedSubscriber { email }),
        Err(error) => Err(anyhow::anyhow!(error)),
    })
    .collect();

    Ok(rows)
}

struct Credentials {
    username: String,
    password: Secret<String>,
}

#[tracing::instrument(name = "Validate credentials", skip(pool, credentials))]
async fn validate_credentials(
    pool: &PgPool,
    credentials: Credentials,
) -> Result<uuid::Uuid, PublishError> {
    let mut user_id = None;
    let mut expected_password_hash = Secret::new(
        "$argon2id$v=19$m=15000,t=2,p=1$\
        gZiV/M1gPc22ElAH/Jh1Hw$\
        CWOrkoo7oJBQ/iyn7uJ0LO2aLEfrHwTWllSAxT0zRno"
            .to_string(),
    );

    if let Some((stored_user_id, stored_password_hash)) =
        get_stored_credentials(&credentials.username, pool)
            .await
            .map_err(PublishError::UnexpectedError)?
    {
        user_id = Some(stored_user_id);
        expected_password_hash = stored_password_hash;
    }

    spawn_blocking_with_tracing(|| {
        verify_password_hash(expected_password_hash, credentials.password)
    })
    .await
    .context("Failed to spawn blocking task")
    .map_err(PublishError::UnexpectedError)??;

    user_id.ok_or_else(|| PublishError::AuthorizationError(anyhow::anyhow!("Invalid credentials")))
}

fn verify_password_hash(
    expected_password_hash: Secret<String>,
    password: Secret<String>,
) -> Result<(), PublishError> {
    let expected_password_hash = PasswordHash::new(&expected_password_hash.expose_secret())
        .context("Failed to parse password hash")
        .map_err(PublishError::UnexpectedError)?;

    Argon2::default()
        .verify_password(password.expose_secret().as_bytes(), &expected_password_hash)
        .context("failed to verify password hash")
        .map_err(PublishError::AuthorizationError)?;
    Ok(())
}

async fn get_stored_credentials(
    username: &str,
    pool: &PgPool,
) -> Result<Option<(uuid::Uuid, Secret<String>)>, anyhow::Error> {
    let user = sqlx::query!(
        r#"
        SELECT user_id, password_hash
        FROM users
        WHERE name = $1
        "#,
        username,
    )
    .fetch_optional(pool)
    .await?
    .map(|row| (row.user_id, Secret::new(row.password_hash)));

    Ok(user)
}

fn basic_authentication(headers: &HeaderMap) -> Result<Credentials, anyhow::Error> {
    let header_value = headers
        .get("Authorization")
        .context("Missing authorization header")?
        .to_str()
        .context("Failed to parse authorization header")?;

    let base64encoded = header_value
        .strip_prefix("Basic ")
        .context("Invalid authorization header")?;

    let decoded_bytes = base64::engine::general_purpose::STANDARD
        .decode(base64encoded)
        .context("Failed to decode")?;
    let decoded = String::from_utf8(decoded_bytes).context("Failed to parse UTF-8")?;

    let mut split = decoded.splitn(2, ':');

    let username = split
        .next()
        .context("Missing username in authorization header")?
        .to_string();

    let password = split
        .next()
        .context("Missing password in authorization header")?
        .to_string();

    Ok(Credentials {
        username,
        password: Secret::new(password),
    })
}
