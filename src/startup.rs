use actix_session::{storage::RedisSessionStore, SessionMiddleware};
use actix_web::{cookie::Key, dev::Server, web, App, HttpServer};
use actix_web_flash_messages::{storage::CookieMessageStore, FlashMessagesFramework};
use secrecy::{ExposeSecret, Secret};
use sqlx::{postgres::PgPoolOptions, PgPool};
use std::{net::TcpListener, time::Duration};
use tracing_actix_web::TracingLogger;

use crate::{
    configuration::DatabaseSettings, configuration::Settings, email_client::EmailClient, routes,
};

pub struct HmacSecret(pub Secret<String>);

pub async fn run(
    listener: TcpListener,
    connection: PgPool,
    email_client: EmailClient,
    base_url: String,
    hmac_secret: Secret<String>,
    redis_url: Secret<String>,
) -> Result<Server, anyhow::Error> {
    let db_connection = web::Data::new(connection);
    let email_client = web::Data::new(email_client);
    let base_url = web::Data::new(base_url);
    let hmac_secret_data = web::Data::new(HmacSecret(hmac_secret.clone()));

    let cookie_storage =
        CookieMessageStore::builder(Key::from(hmac_secret.expose_secret().as_bytes())).build();
    let message_framework = FlashMessagesFramework::builder(cookie_storage).build();

    let redis_session_store = RedisSessionStore::new(redis_url.expose_secret()).await?;

    let server = HttpServer::new(move || {
        App::new()
            .wrap(message_framework.clone())
            .wrap(SessionMiddleware::new(
                redis_session_store.clone(),
                Key::from(hmac_secret.expose_secret().as_bytes()),
            ))
            .wrap(TracingLogger::default())
            .route("/", web::get().to(routes::home))
            .route("/login", web::get().to(routes::login_form))
            .route("/login", web::post().to(routes::login))
            .route("/health_check", web::get().to(routes::health_check))
            .route("/subscriptions", web::post().to(routes::subscribe))
            .route("/subscriptions/confirm", web::get().to(routes::confirm))
            .route("/newsletters", web::post().to(routes::newsletter))
            .app_data(db_connection.clone())
            .app_data(email_client.clone())
            .app_data(base_url.clone())
            .app_data(hmac_secret_data.clone())
    })
    .listen(listener)?
    .run();
    Ok(server)
}

pub struct Application {
    pub port: u16,
    pub server: Server,
}

impl Application {
    pub async fn build(configuration: Settings) -> Result<Self, anyhow::Error> {
        let connection_pool = get_connection_pool(&configuration.database);
        let sender_email = configuration
            .email_configuration
            .sender()
            .expect("Invalid sender email");

        let timeout = configuration.email_configuration.timeout();
        let email_client = EmailClient::new(
            configuration.email_configuration.base_url,
            sender_email,
            configuration.email_configuration.authorization_token,
            timeout,
        );

        let address = format!(
            "{}:{}",
            configuration.application_settings.host, configuration.application_settings.port
        );

        let listener = TcpListener::bind(address)?;
        let port = listener.local_addr().unwrap().port();
        let server = run(
            listener,
            connection_pool,
            email_client,
            configuration.application_settings.base_url,
            configuration.application_settings.hmac_secret,
            configuration.redis_url,
        )
        .await?;
        Ok(Self { port, server })
    }

    pub fn port(&self) -> u16 {
        self.port
    }

    pub async fn run_until_stopped(self) -> Result<(), std::io::Error> {
        self.server.await
    }
}

pub fn get_connection_pool(configuration: &DatabaseSettings) -> PgPool {
    PgPoolOptions::new()
        .acquire_timeout(Duration::from_secs(2))
        .connect_lazy_with(configuration.with_db())
}
