use sqlx::postgres::PgPoolOptions;
use std::net::TcpListener;
use zero2prod::startup::run;
use zero2prod::telemetry::{get_subscriber, init_subscriber};

#[tokio::main]
async fn main() -> Result<(), std::io::Error> {
    let subscriber = get_subscriber("zero2prod".into(), "info".into(), std::io::stdout);
    init_subscriber(subscriber);

    let configuration =
        zero2prod::configuration::get_configuration().expect("Unable to get configuration");
    let address = format!(
        "{}:{}",
        configuration.application_settings.host, configuration.application_settings.port
    );

    let connection_pool = PgPoolOptions::new()
        .acquire_timeout(std::time::Duration::from_secs(2))
        .connect_lazy_with(configuration.database.with_db());
    let listener = TcpListener::bind(address).expect("Unable to bind to socket");
    run(listener, connection_pool)?.await
}
