use actix_web::{web, Error, HttpResponse};
use actix_web_flash_messages::FlashMessage;
use secrecy::{ExposeSecret, Secret};
use sqlx::PgPool;

use crate::{
    authentification::{self, validate_credentials, AuthError, Credentials},
    routes::get_username,
    session_state::TypedSession,
    utils::{e500, see_other},
};

#[derive(serde::Deserialize)]
pub struct ChangePasswordData {
    password: Secret<String>,
    new_password: Secret<String>,
    confirm_password: Secret<String>,
}

pub async fn change_password(
    form: web::Form<ChangePasswordData>,
    pool: web::Data<PgPool>,
    session: TypedSession,
) -> Result<HttpResponse, Error> {
    let user_id = session.get_user_id().map_err(e500)?;
    if user_id.is_none() {
        return Ok(see_other("/login"));
    }
    let user_id = user_id.unwrap();

    if form.new_password.expose_secret() != form.confirm_password.expose_secret() {
        FlashMessage::error("Passwords do not match").send();
        return Ok(see_other("/admin/password"));
    }

    let username = get_username(user_id, &pool).await.map_err(e500)?;
    let credentials = Credentials {
        username,
        password: form.0.password,
    };

    if let Err(e) = validate_credentials(&pool, credentials).await {
        return match e {
            AuthError::InvalidCredentials(_) => {
                FlashMessage::error("The Current Password is incorrect").send();
                Ok(see_other("/admin/password"))
            }
            AuthError::UnexpectedError(e) => Err(e500(e)),
        };
    }

    authentification::change_password(user_id, form.0.new_password, &pool)
        .await
        .map_err(e500)?;

    FlashMessage::error("You have successfully changed your password").send();
    Ok(see_other("/admin/dashboard"))
}
