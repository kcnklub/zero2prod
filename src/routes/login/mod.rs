use actix_web::{
    cookie::{time, Cookie},
    HttpResponse, Responder,
};
use std::fmt::Write;

mod post;
use actix_web_flash_messages::{IncomingFlashMessages, Level};
pub use post::*;

pub async fn login_form(flash_message: IncomingFlashMessages) -> impl Responder {
    let mut error_message = String::new();
    for m in flash_message.iter().filter(|m| m.level() == Level::Error) {
        writeln!(error_message, "<p><i>{}</i></p>", m.content()).unwrap();
    }
    HttpResponse::Ok()
        .content_type("text/html; charset=utf-8")
        .cookie(
            Cookie::build("_flash", "")
                .max_age(time::Duration::ZERO)
                .finish(),
        )
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
