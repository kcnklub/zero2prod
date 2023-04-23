use std::net::TcpListener;

use sqlx::{Connection, PgConnection};
use zero2prod::configuration::{self, get_configuration};

#[tokio::test]
async fn health_check_works() {
    let host = spawn_app();

    let client = reqwest::Client::new();

    let response = client
        .get(&format!("{}/health_check", &host))
        .send()
        .await
        .expect("Failed to execute request");

    assert!(response.status().is_success());
    assert_eq!(Some(0), response.content_length());
}

#[tokio::test]
async fn subscribe_returns_a_200_for_valid_form_data() {
    let host = spawn_app();
    let configuration = get_configuration().expect("Unable to read configuration");
    let connection_string = configuration.database.connection_string();
    let mut connection = PgConnection::connect(&connection_string)
        .await
        .expect("Failed to create connection to postgres");
    let client = reqwest::Client::new();

    let body = "name=kcnklub&email=test@test.com";
    let response = client
        .post(&format!("{}/subscriptions", host))
        .header("Content-Type", "application/x-www-form-urlencoded")
        .body(body)
        .send()
        .await
        .expect("Failed to execute request");

    let saved = sqlx::query!("Select email, name from subscriptions",)
        .fetch_one(&mut connection)
        .await
        .expect("Failed to fetch data from the database");

    assert!(response.status().is_success());
    assert_eq!(saved.email, "test@test.com");
    assert_eq!(saved.name, "kcnklub");
}

#[tokio::test]
async fn subscribe_return_a_400_when_data_is_missing() {
    let host = spawn_app();
    let client = reqwest::Client::new();
    let test_cases = vec![
        ("name=kcnklub", "missing the email!"),
        ("email=test@test.com", "missing the name!"),
        ("", "missing both inputs"),
    ];

    for (invalid_body, error_message) in test_cases {
        let response = client
            .post(&format!("{}/subscriptions", host))
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

fn spawn_app() -> String {
    let listener = TcpListener::bind("127.0.0.1:0").expect("Unable to bind to socket");
    let port = listener.local_addr().unwrap().port();
    let server = zero2prod::startup::run(listener).expect("Unable to bind to port");
    let _ = tokio::spawn(server);
    format!("http://localhost:{}", port)
}
