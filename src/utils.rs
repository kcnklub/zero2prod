use actix_web::{error::ErrorInternalServerError, http::header::LOCATION, Error, HttpResponse};

pub fn e500<T>(e: T) -> Error
where
    T: std::fmt::Debug + std::fmt::Display + 'static,
{
    ErrorInternalServerError(e)
}

pub fn see_other(location: &str) -> HttpResponse {
    HttpResponse::SeeOther()
        .insert_header((LOCATION, location))
        .finish()
}
