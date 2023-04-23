use actix_web::{dev::Server, web, App, HttpServer};
use sqlx::PgPool;
use std::net::TcpListener;

use crate::routes;

pub fn run(listener: TcpListener, connection: PgPool) -> Result<Server, std::io::Error> {
    let connection_pool = web::Data::new(connection);
    let server = HttpServer::new(move || {
        App::new()
            .route("/health_check", web::get().to(routes::health_check))
            .route("/subscriptions", web::post().to(routes::subscribe))
            .app_data(connection_pool.clone())
    })
    .listen(listener)?
    .run();

    Ok(server)
}
