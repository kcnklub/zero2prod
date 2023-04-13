use std::net::TcpListener;

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
    let client = reqwest::Client::new();

    let body = "name=kcnklub&email=test@test.com";
    let response = client
        .post(&format!("{}/subscriptions", host))
        .header("Content-Type", "application/x-www-form-urlencoded")
        .body(body)
        .send()
        .await
        .expect("Failed to execute request");

    assert!(response.status().is_success());
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
    let server = zero2prod::run(listener).expect("Unable to bind to port");
    let _ = tokio::spawn(server);
    format!("http://localhost:{}", port)
}
