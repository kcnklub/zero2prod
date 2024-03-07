use actix_web::{
    web::{Data, Form},
    HttpResponse, ResponseError,
};
use anyhow::Context;
use chrono::Utc;
use rand::distributions::Alphanumeric;
use rand::{thread_rng, Rng};
use sqlx::Transaction;
use sqlx::{PgPool, Postgres};
use uuid::Uuid;

use crate::domain::NewSubscriber;
use crate::domain::SubscriberEmail;
use crate::domain::SubscriberName;
use crate::email_client::EmailClient;

#[derive(serde::Deserialize)]
pub struct FormData {
    name: String,
    email: String,
}

impl TryFrom<FormData> for NewSubscriber {
    type Error = String;
    fn try_from(value: FormData) -> Result<Self, Self::Error> {
        let name = SubscriberName::parse(value.name)?;
        let email = SubscriberEmail::parse(value.email)?;
        Ok(NewSubscriber { email, name })
    }
}

#[derive(thiserror::Error)]
pub enum SubscribeError {
    #[error("{0}")]
    ValidationError(String),
    #[error(transparent)]
    UnexpectedError(#[from] anyhow::Error),
}

impl std::fmt::Debug for SubscribeError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        error_chain_fmt(self, f)
    }
}

impl From<String> for SubscribeError {
    fn from(error: String) -> Self {
        Self::ValidationError(error)
    }
}

impl ResponseError for SubscribeError {
    fn status_code(&self) -> actix_web::http::StatusCode {
        match self {
            Self::ValidationError(_) => actix_web::http::StatusCode::BAD_REQUEST,
            Self::UnexpectedError(_) => actix_web::http::StatusCode::INTERNAL_SERVER_ERROR,
        }
    }
}

#[tracing::instrument(
    name = "Adding a new subscriber",
    skip(form, pool, email_client, base_url),
    fields(
        subscriber_email = %form.email,
        subscriber_name = %form.name,
    )
)]
pub async fn subscribe(
    Form(form): Form<FormData>,
    pool: Data<PgPool>,
    email_client: Data<EmailClient>,
    base_url: Data<String>,
) -> Result<HttpResponse, SubscribeError> {
    println!("Adding a new subscriber");
    let new_subscriber = form.try_into().map_err(SubscribeError::ValidationError)?;
    let mut transaction = pool
        .begin()
        .await
        .context("Failed to acquire a Postgres connection")?;
    let subscriber_id = insert_subscriber(&new_subscriber, &mut transaction)
        .await
        .context("Failed to insert a new subscriber")?;
    let token = generate_subscription_token();
    store_token(&mut transaction, &subscriber_id, &token)
        .await
        .context("Failed to store subscription token")?;
    send_confirmation_email(new_subscriber, &email_client, &base_url, &token)
        .await
        .context("Failed to send confirmation email")?;
    transaction
        .commit()
        .await
        .context("Failed to commit SQL transaction")?;
    Ok(HttpResponse::Ok().finish())
}

pub async fn insert_subscriber(
    new_subscriber: &NewSubscriber,
    pool: &mut Transaction<'_, Postgres>,
) -> Result<Uuid, sqlx::Error> {
    let subscriber_id = Uuid::new_v4();
    sqlx::query!(
        r#"
    INSERT INTO subscriptions (id, email, name, subscribed_at, status)
    VALUES ($1, $2, $3, $4, 'pending_confirmation')
    "#,
        subscriber_id,
        new_subscriber.email.as_ref(),
        new_subscriber.name.as_ref(),
        Utc::now()
    )
    .execute(pool)
    .await?;
    Ok(subscriber_id)
}

pub fn error_chain_fmt(
    e: &impl std::error::Error,
    f: &mut std::fmt::Formatter<'_>,
) -> std::fmt::Result {
    writeln!(f, "{}\n", e)?;
    let mut current = e.source();
    while let Some(cause) = current {
        writeln!(f, "Caused by:\n\t{}", cause)?;
        current = cause.source();
    }
    Ok(())
}

pub struct StoreTokenError(sqlx::Error);

impl std::fmt::Debug for StoreTokenError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        error_chain_fmt(self, f)
    }
}

impl std::fmt::Display for StoreTokenError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "A database error occurred while storing a subscription token"
        )
    }
}

impl std::error::Error for StoreTokenError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        Some(&self.0)
    }
}

async fn store_token(
    pool: &mut Transaction<'_, Postgres>,
    subscriber_id: &Uuid,
    token: &str,
) -> Result<(), StoreTokenError> {
    sqlx::query!(
        r#"
    INSERT INTO subscription_tokens (subscription_token, subscription_id)
    VALUES ($1, $2)
    "#,
        token,
        subscriber_id
    )
    .execute(pool)
    .await
    .map_err(StoreTokenError)?;
    Ok(())
}

async fn send_confirmation_email(
    new_subscriber: NewSubscriber,
    email_client: &EmailClient,
    base_url: &str,
    token: &str,
) -> Result<(), reqwest::Error> {
    let confirmation_link = format!(
        "{}/subscriptions/confirm?subscription_token={}",
        base_url, token
    );
    println!("Confirmation link: {}", confirmation_link);
    email_client
        .send_email(
            &new_subscriber.email,
            "Welcome!",
            &format!(
                "<html>Welcome to my new Newsletter!<br/>
                Visit <a href=\"{}\">here</a> to confirm your subscription</html>",
                confirmation_link
            ),
            &format!(
                "Welcome to my new Newsletter!\nVisit {} to confirm your subscription",
                confirmation_link
            ),
        )
        .await?;

    Ok(())
}

fn generate_subscription_token() -> String {
    let mut rng = thread_rng();
    std::iter::repeat(())
        .map(|()| rng.sample(Alphanumeric))
        .map(char::from)
        .take(25)
        .collect()
}
