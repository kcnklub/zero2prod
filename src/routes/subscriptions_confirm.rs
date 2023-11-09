use actix_web::{
    web::{Data, Query},
    HttpResponse,
};
use serde::Deserialize;
use sqlx::PgPool;
use uuid::Uuid;

#[derive(Deserialize, Debug)]
pub struct Parameter {
    subscription_token: String,
}

#[tracing::instrument(name = "Confirm a pending subscriber", skip(parameters, pool))]
pub async fn confirm(Query(parameters): Query<Parameter>, pool: Data<PgPool>) -> HttpResponse {
    let id = match get_subscriber_id(&pool, &parameters.subscription_token).await {
        Ok(id) => id,
        Err(_) => return HttpResponse::InternalServerError().finish(),
    };

    let id = match id {
        Some(id) => id,
        None => return HttpResponse::Unauthorized().finish(),
    };

    if confirm_subscriber(&pool, id).await.is_err() {
        return HttpResponse::InternalServerError().finish();
    }

    HttpResponse::Ok().finish()
}

async fn get_subscriber_id(
    pool: &PgPool,
    subscription_token: &str,
) -> Result<Option<Uuid>, sqlx::Error> {
    let rec = sqlx::query!(
        r#"SELECT subscription_id FROM subscription_tokens WHERE subscription_token = $1"#,
        subscription_token
    )
    .fetch_optional(pool)
    .await?;

    Ok(rec.map(|r| r.subscription_id))
}

async fn confirm_subscriber(pool: &PgPool, id: Uuid) -> Result<(), sqlx::Error> {
    sqlx::query!(
        r#"UPDATE subscriptions SET status = 'confirmed' WHERE id = $1"#,
        id
    )
    .execute(pool)
    .await?;

    delete_token(pool, id).await
}

async fn delete_token(pool: &sqlx::Pool<sqlx::Postgres>, id: Uuid) -> Result<(), sqlx::Error> {
    sqlx::query!(
        r#"DELETE FROM subscription_tokens WHERE subscription_id = $1"#,
        id
    )
    .execute(pool)
    .await?;

    Ok(())
}
