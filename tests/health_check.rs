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

fn spawn_app() -> String {
    let listener = TcpListener::bind("127.0.0.1:0").expect("Unable to bind to socket");
    let port = listener.local_addr().unwrap().port();
    let server = zero2prod::run(listener).expect("Unable to bind to port");
    let _ = tokio::spawn(server);
    format!("http://localhost:{}", port)
}
