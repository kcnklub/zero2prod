use actix_web::{web, HttpResponse};
use actix_web_flash_messages::IncomingFlashMessages;
use anyhow::Context;
use sqlx::PgPool;
use uuid::Uuid;

use crate::{authentication::UserId, utils::e500};

pub async fn admin_dashboard(
    pool: web::Data<PgPool>,
    flash_messages: IncomingFlashMessages,
    user_id: web::ReqData<UserId>,
) -> Result<HttpResponse, actix_web::Error> {
    let user_id = user_id.into_inner();
    let username = get_username(*user_id, &pool).await.map_err(e500)?;
    let mut message_html = String::new();
    for message in flash_messages.iter() {
        message_html.push_str(&format!("<p>{}</p>", message.content()));
    }

    Ok(HttpResponse::Ok().body(format!("Welcome, {}!<br />{}", username, message_html)))
}

#[tracing::instrument(name = "fetching username from database", skip(pool))]
pub async fn get_username(user_id: Uuid, pool: &PgPool) -> Result<String, anyhow::Error> {
    let row = sqlx::query!("SELECT name FROM users WHERE user_id = $1", user_id)
        .fetch_one(pool)
        .await
        .context("Failed to fetch username")?;

    Ok(row.name)
}
