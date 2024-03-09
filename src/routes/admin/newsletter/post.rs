use actix_web::{http::header, web, HttpResponse, ResponseError};
use anyhow::Context;
use reqwest::{header::HeaderValue, StatusCode};
use sqlx::PgPool;

use crate::{
    authentication::UserId, domain::SubscriberEmail, email_client::EmailClient,
    routes::error_chain_fmt, utils::see_other,
};

#[derive(serde::Deserialize)]
pub struct NewsletterForm {
    pub title: String,
    pub html_content: String,
    pub content: String,
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

pub async fn send_newsletter(
    user_id: web::ReqData<UserId>,
    form: web::Form<NewsletterForm>,
    email_client: web::Data<EmailClient>,
    pool: web::Data<PgPool>,
) -> Result<HttpResponse, PublishError> {
    let user_id = user_id.into_inner();
    let form = form.into_inner();
    tracing::Span::current().record("user_id", &tracing::field::display(user_id));

    let confirmed_subscribers = gets_confirmed_subscriber(&pool).await?;
    for confirmed_subscriber in confirmed_subscribers {
        match confirmed_subscriber {
            Ok(confirmed_subscriber) => {
                let _ = email_client
                    .send_email(
                        &confirmed_subscriber.email,
                        &form.title,
                        &form.html_content,
                        &form.content,
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
    Ok(see_other("/admin/dashboard"))
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
