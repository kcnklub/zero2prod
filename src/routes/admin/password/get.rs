use actix_web::{Error, HttpResponse};
use actix_web_flash_messages::IncomingFlashMessages;

use crate::{
    session_state::TypedSession,
    utils::{e500, see_other},
};

pub async fn change_password_form(
    session: TypedSession,
    flash_messages: IncomingFlashMessages,
) -> Result<HttpResponse, Error> {
    if session.get_user_id().map_err(e500)?.is_none() {
        return Ok(see_other("/login"));
    }

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
