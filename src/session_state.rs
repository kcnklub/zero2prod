use actix_session::{Session, SessionExt};
use actix_web::{dev::Payload, FromRequest, HttpRequest};
use std::future::{ready, Ready};
use uuid::Uuid;

pub struct TypedSession(Session);

impl TypedSession {
    const USER_ID: &'static str = "user_id";

    pub fn renew(&self) {
        self.0.renew();
    }

    pub fn insert_user_id(&self, value: Uuid) -> Result<(), actix_session::SessionInsertError> {
        self.0.insert(Self::USER_ID, value)?;
        Ok(())
    }

    pub fn get_user_id(&self) -> Result<Option<Uuid>, actix_session::SessionGetError> {
        self.0.get(Self::USER_ID)
    }

    pub fn log_out(&self) {
        self.0.purge();
    }
}

impl FromRequest for TypedSession {
    type Error = <Session as FromRequest>::Error;
    type Future = Ready<Result<TypedSession, Self::Error>>;

    fn from_request(req: &HttpRequest, _payload: &mut Payload) -> Self::Future {
        ready(Ok(TypedSession(req.get_session())))
    }
}
