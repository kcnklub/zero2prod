use actix_web::{web::Query, HttpResponse};
use serde::Deserialize;

#[derive(Deserialize, Debug)]
pub struct Parameter {
    subscription_token: String,
}

#[tracing::instrument(name = "Confirm a pending subscriber", skip(_parameters))]
pub async fn confirm(Query(_parameters): Query<Parameter>) -> HttpResponse {
    HttpResponse::Ok().finish()
}
