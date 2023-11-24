use actix_web::{HttpResponse, Responder};

mod post;
pub use post::*;

pub async fn login_form() -> impl Responder {
    HttpResponse::Ok()
        .content_type("text/html; charset=utf-8")
        .body(include_str!("login.html"))
}
