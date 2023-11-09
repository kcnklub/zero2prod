use actix_web::{
    web::{Data, Form},
    HttpResponse, Responder,
};
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
) -> impl Responder {
    println!("Adding a new subscriber");
    let new_subscriber = match form.try_into() {
        Ok(new_sub) => new_sub,
        Err(_) => return HttpResponse::BadRequest().finish(),
    };

    let mut transaction = match pool.begin().await {
        Ok(t) => t,
        Err(_) => return HttpResponse::InternalServerError().finish(),
    };
    let subscriber_id = match insert_subscriber(&new_subscriber, &mut transaction).await {
        Ok(id) => id,
        Err(_) => return HttpResponse::InternalServerError().finish(),
    };
    let token = generate_subscription_token();

    if store_token(&mut transaction, &subscriber_id, &token)
        .await
        .is_err()
    {
        println!("Failed to store subscription token");
        return HttpResponse::InternalServerError().finish();
    }

    if send_confirmation_email(new_subscriber, &email_client, &base_url, &token)
        .await
        .is_err()
    {
        println!("Failed to send confirmation email");
        return HttpResponse::InternalServerError().finish();
    }

    if transaction.commit().await.is_err() {
        println!("Failed to commit transaction");
        return HttpResponse::InternalServerError().finish();
    }

    HttpResponse::Ok().finish()
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
    .await
    .map_err(|error| {
        tracing::error!("Failed to execute query: {:?}", error);
        error
    })?;
    Ok(subscriber_id)
}

async fn store_token(
    pool: &mut Transaction<'_, Postgres>,
    subscriber_id: &Uuid,
    token: &str,
) -> Result<(), sqlx::Error> {
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
    .map_err(|error| {
        tracing::error!("Failed to execute query: {:?}", error);
        error
    })?;
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
    let result = email_client
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
        .await;

    if let Err(error) = result {
        tracing::info!("Failed to send confirmation email: {:?}", error);
        return Err(error);
    }

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
