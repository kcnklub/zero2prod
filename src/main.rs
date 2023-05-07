use secrecy::ExposeSecret;
use sqlx::PgPool;
use std::net::TcpListener;
use zero2prod::startup::run;
use zero2prod::telemetry::{get_subscriber, init_subscriber};

#[tokio::main]
async fn main() -> Result<(), std::io::Error> {
    let subscriber = get_subscriber("zero2prod".into(), "info".into(), std::io::stdout);
    init_subscriber(subscriber);

    let configuration =
        zero2prod::configuration::get_configuration().expect("Unable to get configuration");
    let address = format!("0.0.0.0:{}", configuration.application_port);
    let connection_pool =
        PgPool::connect_lazy(&configuration.database.connection_string().expose_secret())
            .expect("Uable to create db connection");
    let listener = TcpListener::bind(address).expect("Unable to bind to socket");
    run(listener, connection_pool)?.await
}
