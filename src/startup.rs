use actix_web::dev::Server;
use actix_web::web;
use actix_web::App;
use actix_web::HttpServer;
use std::net::TcpListener;

use crate::routes;

pub fn run(listener: TcpListener) -> Result<Server, std::io::Error> {
    let server = HttpServer::new(|| {
        App::new()
            .route("/health_check", web::get().to(routes::health_check))
            .route("/subscriptions", web::post().to(routes::subscribe))
    })
    .listen(listener)?
    .run();

    Ok(server)
}
