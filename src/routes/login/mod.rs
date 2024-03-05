use actix_web::{web, HttpResponse, Responder};
use hmac::{Hmac, Mac};
use htmlescape;

mod post;
pub use post::*;
use secrecy::ExposeSecret;

use crate::startup::HmacSecret;

#[derive(serde::Deserialize)]
pub struct QueryData {
    error: String,
    tag: String,
}

impl QueryData {
    fn verify(self, secret: &HmacSecret) -> Result<String, anyhow::Error> {
        let tag = hex::decode(self.tag)?;
        let query_string = format!("error={}", urlencoding::encode(&self.error));

        let mut mac = Hmac::<sha2::Sha256>::new_from_slice(secret.0.expose_secret().as_bytes())?;
        mac.update(query_string.as_bytes());
        mac.verify_slice(&tag)?;

        Ok(self.error)
    }
}

pub async fn login_form(
    query: Option<web::Query<QueryData>>,
    secret: web::Data<HmacSecret>,
) -> impl Responder {
    let error_message = match query {
        None => "".into(),
        Some(query) => match query.0.verify(&secret) {
            Ok(error) => format!(
                r##"
            <p><i>{}</i></p>
                "##,
                htmlescape::encode_minimal(&error)
            ),
            Err(e) => {
                tracing::warn!(
                    error.message = %e,
                    error.cause_chain = ?e,
                    "Failed to verify query string."
                );
                "".into()
            }
        },
    };
    HttpResponse::Ok()
        .content_type("text/html; charset=utf-8")
        .body(format!(
            r##"
            <!doctype html>
            <html lang="en">
                <head>
                    <meta charset="UTF-8" />
                    <title>Login</title>
                    <meta http-equiv="content-type" content="text/html; charset=utf-8" />
                </head>
                <body>
                    {error_message}
                    <form action="/login" method="post">
                        <label>
                            Username
                            <input type="text" name="username" />
                        </label>
                        <label>
                            Password
                            <input type="password" name="password" />
                        </label>
                        <input type="submit" value="Login" />
                    </form>
                </body>
            </html
            "##
        ))
}
