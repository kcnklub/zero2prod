use actix_web::web;
use actix_web::HttpResponse;
use actix_web::Responder;
use chrono::Utc;
use log::error;
use log::info;
use sqlx::PgPool;
use uuid::Uuid;

#[derive(serde::Deserialize)]
pub struct FormData {
    name: String,
    email: String,
}

pub async fn subscribe(form: web::Form<FormData>, connection: web::Data<PgPool>) -> impl Responder {
    let request_id = Uuid::new_v4();
    info!(
        "request id {} Adding '{}' '{}' as a new subscriber",
        request_id, form.email, form.name
    );
    info!("Saving new subscriber deatils in teh database");
    match sqlx::query!(
        r#"
    INSERT INTO subscriptions (id, email, name, subscribed_at)
    VALUES ($1, $2, $3, $4)
    "#,
        Uuid::new_v4(),
        form.email,
        form.name,
        Utc::now()
    )
    .execute(connection.get_ref())
    .await
    {
        Ok(_) => {
            info!("request id {} adding new subscriber", request_id);
            HttpResponse::Ok().finish()
        }
        Err(err) => {
            error!(
                "request id {} to create Subscription: {:?}",
                request_id, err
            );
            HttpResponse::InternalServerError().finish()
        }
    }
}
