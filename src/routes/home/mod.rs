use actix_web::{HttpResponse, Responder};

pub async fn home() -> impl Responder {
    HttpResponse::Ok()
        .content_type("text/html; charset=utf-8")
        .body(include_str!("home.html"))
}
