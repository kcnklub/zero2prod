use env_logger::Env;
use sqlx::PgPool;
use std::net::TcpListener;
use zero2prod::startup::run;

#[tokio::main]
async fn main() -> Result<(), std::io::Error> {
    env_logger::Builder::from_env(Env::default().default_filter_or("info")).init();
    let configuration =
        zero2prod::configuration::get_configuration().expect("Unable to get configuration");
    let address = format!("127.0.0.1:{}", configuration.application_port);
    let database_connection_string = configuration.database.connection_string();
    let connection_pool = PgPool::connect(&database_connection_string)
        .await
        .expect("Uable to create db connection");
    let listener = TcpListener::bind(address).expect("Unable to bind to socket");
    run(listener, connection_pool)?.await
}
