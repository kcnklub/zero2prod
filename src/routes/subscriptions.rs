use actix_web::web::{Data, Form};
use actix_web::{HttpResponse, Responder};
use chrono::Utc;
use sqlx::PgPool;
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

#[tracing::instrument(
    name = "Adding a new subscriber",
    skip(form, pool, email_client),
    fields(
        subscriber_email = %form.email,
        subscriber_name = %form.name,
    )
)]
pub async fn subscribe(
    Form(form): Form<FormData>,
    pool: Data<PgPool>,
    email_client: Data<EmailClient>,
) -> impl Responder {
    println!("Adding a new subscriber");
    let new_subscriber = match form.try_into() {
        Ok(new_sub) => new_sub,
        Err(_) => return HttpResponse::BadRequest().finish(),
    };

    if insert_subscriber(&new_subscriber, &pool).await.is_err() {
        return HttpResponse::InternalServerError().finish();
    }

    if send_confirmation_email(&email_client, new_subscriber)
        .await
        .is_err()
    {
        return HttpResponse::InternalServerError().finish();
    }

    HttpResponse::Ok().finish()
}

#[tracing::instrument(
    name = "Saving new subscriber details to the database",
    skip(new_subscriber, pool)
)]
pub async fn insert_subscriber(
    new_subscriber: &NewSubscriber,
    pool: &PgPool,
) -> Result<(), sqlx::Error> {
    sqlx::query!(
        r#"
    INSERT INTO subscriptions (id, email, name, subscribed_at, status)
    VALUES ($1, $2, $3, $4, 'pending_confirmation')
    "#,
        Uuid::new_v4(),
        new_subscriber.email.as_ref(),
        new_subscriber.name.as_ref(),
        Utc::now()
    )
    .execute(pool)
    .await
    .map_err(|error| {
        tracing::error!("Failed to execute query: {:?}", error);
        error
    })?;
    Ok(())
}

#[tracing::instrument(
    name = "Send a confirmation email to a new subscriber",
    skip(email_client, new_subscriber)
)]
async fn send_confirmation_email(
    email_client: &EmailClient,
    new_subscriber: NewSubscriber,
) -> Result<(), reqwest::Error> {
    let confirmation_link = "https://there-is-no-link-yet.com/subscriptions/confirm";
    email_client
        .send_email(
            new_subscriber.email,
            "Welcome!",
            &format!(
                "Welcome to our newsletter!<br/>\
                Visit <a href=\"{}\"></a> to confirm your subscription",
                confirmation_link
            ),
            &format!(
                "Welcome to our newsletter!\nVisit {} to confirm your subscription",
                confirmation_link
            ),
        )
        .await?;

    Ok(())
}
