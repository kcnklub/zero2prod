use sqlx::{Connection, Executor, PgConnection, PgPool};
use std::net::TcpListener;
use zero2prod::configuration::{get_configuration, DatabaseSettings};

pub struct TestApp {
    pub address: String,
    pub db_pool: PgPool,
}

#[tokio::test]
async fn health_check_works() {
    let test_app = spawn_app().await;

    let client = reqwest::Client::new();

    let response = client
        .get(&format!("{}/health_check", &test_app.address))
        .send()
        .await
        .expect("Failed to execute request");

    assert!(response.status().is_success());
    assert_eq!(Some(0), response.content_length());
}

#[tokio::test]
async fn subscribe_returns_a_200_for_valid_form_data() {
    let test_app = spawn_app().await;
    let client = reqwest::Client::new();

    let body = "name=kcnklub&email=test@test.com";
    let response = client
        .post(&format!("{}/subscriptions", test_app.address))
        .header("Content-Type", "application/x-www-form-urlencoded")
        .body(body)
        .send()
        .await
        .expect("Failed to execute request");

    let saved = sqlx::query!("Select email, name from subscriptions",)
        .fetch_one(&test_app.db_pool)
        .await
        .expect("Failed to fetch data from the database");

    assert!(response.status().is_success());
    assert_eq!(saved.email, "test@test.com");
    assert_eq!(saved.name, "kcnklub");
}

#[tokio::test]
async fn subscribe_return_a_400_when_data_is_missing() {
    let test_app = spawn_app().await;
    let client = reqwest::Client::new();
    let test_cases = vec![
        ("name=kcnklub", "missing the email!"),
        ("email=test@test.com", "missing the name!"),
        ("", "missing both inputs"),
    ];

    for (invalid_body, error_message) in test_cases {
        let response = client
            .post(&format!("{}/subscriptions", test_app.address))
            .header("Content-Type", "application/x-www-form-urlencoded")
            .body(invalid_body)
            .send()
            .await
            .expect("Failed to execute request");

        assert_eq!(
            400,
            response.status().as_u16(),
            "Test failed! {}",
            error_message
        );
    }
}

async fn spawn_app() -> TestApp {
    let listener = TcpListener::bind("127.0.0.1:0").expect("Unable to bind to socket");
    let port = listener.local_addr().unwrap().port();

    let mut configuration = get_configuration().expect("Failed to read configuration");
    configuration.database.database_name = uuid::Uuid::new_v4().to_string();
    let pool = configure_database(&configuration.database).await;
    let server = zero2prod::startup::run(listener, pool.clone()).expect("Unable to bind to port");
    let _ = tokio::spawn(server);
    TestApp {
        address: format!("http://localhost:{}", port),
        db_pool: pool,
    }
}

async fn configure_database(config: &DatabaseSettings) -> PgPool {
    let mut connection = PgConnection::connect(&config.connection_string_without_database_name())
        .await
        .expect("Unable to connect to db");

    connection
        .execute(format!(r#"CREATE DATABASE "{}";"#, config.database_name).as_str())
        .await
        .expect("Unable to create db");

    let connection_pool = PgPool::connect(&config.connection_string())
        .await
        .expect("Cannot connect with connection pool");

    sqlx::migrate!("./migrations")
        .run(&connection_pool)
        .await
        .expect("unable to migrate db");

    connection_pool
}
