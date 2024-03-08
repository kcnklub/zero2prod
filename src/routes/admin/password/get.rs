use actix_web::{web, Error, HttpResponse};
use actix_web_flash_messages::IncomingFlashMessages;

use crate::{authentication::UserId, session_state::TypedSession};

pub async fn change_password_form(
    flash_messages: IncomingFlashMessages,
    _user_id: web::ReqData<UserId>,
) -> Result<HttpResponse, Error> {
    let mut message_html = String::new();
    for message in flash_messages.iter() {
        message_html.push_str(&format!("<p>{}</p>", message.content()));
    }

    Ok(HttpResponse::Ok().body(
        format!(r#"
    <!DOCTYPE html>
    <html>
        <head>
        </head>
        <body>
            {}
            <form action="/admin/password" method="post">
                <label>Current Password<input type="password" name="password" /></label><br/>
                <label>New Password<input type="password" name="new_passord" /></label><br/>
                <label>Confirm Password<input type="password" name="confirm_password" /></label><br/>
                <button type="submit" value="Change Password">Change Password</button>
            </form>
        </body>
    </html>
    "#, message_html)))
}
