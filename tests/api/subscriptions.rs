use wiremock::{
    matchers::{method, path},
    Mock, ResponseTemplate,
};

use crate::helpers::spawn_app;

#[tokio::test]
async fn subscribe_returns_a_200_for_valid_form_data() {
    let app = spawn_app().await;

    let body = "name=kcnklub&email=test@test.com";

    Mock::given(path("/email"))
        .and(method("POST"))
        .respond_with(ResponseTemplate::new(200))
        .expect(1)
        .mount(&app.email_server)
        .await;

    let response = app.post_subscriptions(body.into()).await;

    assert_eq!(200, response.status().as_u16());
}

#[tokio::test]
async fn subscribe_persists_the_new_subscriber() {
    let app = spawn_app().await;

    let body = "name=kcnklub&email=test@test.com";

    Mock::given(path("/email"))
        .and(method("POST"))
        .respond_with(ResponseTemplate::new(200))
        .expect(1)
        .mount(&app.email_server)
        .await;

    app.post_subscriptions(body.into()).await;

    let saved = sqlx::query!("Select email, name, status from subscriptions",)
        .fetch_one(&app.db_pool)
        .await
        .expect("Failed to fetch data from the database");

    assert_eq!(saved.email, "test@test.com");
    assert_eq!(saved.name, "kcnklub");
    assert_eq!(saved.status, "pending_confirmation");
}

#[tokio::test]
async fn subscribe_return_a_400_when_data_is_missing() {
    let app = spawn_app().await;
    let test_cases = vec![
        ("name=kcnklub", "missing the email!"),
        ("email=test@test.com", "missing the name!"),
        ("", "missing both inputs"),
    ];

    for (invalid_body, error_message) in test_cases {
        let response = app.post_subscriptions(invalid_body.into()).await;

        assert_eq!(
            400,
            response.status().as_u16(),
            "Test failed! {}",
            error_message
        );
    }
}

#[tokio::test]
async fn subscribe_return_a_200_when_fields_are_present_but_empty() {
    let app = spawn_app().await;
    let test_cases = vec![
        ("name=&email=test@test.com", "empty name"),
        ("name=test&email=", "empty email"),
        ("name=test&email=definitely-not-an-email", "invalid email"),
    ];

    for (body, message) in test_cases {
        let response = app.post_subscriptions(body.into()).await;

        assert_eq!(
            400,
            response.status().as_u16(),
            "The api did not return a 400 bad request when the payload was {}.",
            message
        );
    }
}

#[tokio::test]
async fn subscribe_sends_a_confirmation_email_for_valid_data() {
    let app = spawn_app().await;
    let body = "name=le%20guin&email=ursula_le_guin%40gmail.com";

    Mock::given(path("/email"))
        .and(method("POST"))
        .respond_with(ResponseTemplate::new(200))
        .expect(1)
        .mount(&app.email_server)
        .await;

    let response = app.post_subscriptions(body.into()).await;

    let saved = sqlx::query!("Select email, name from subscriptions",)
        .fetch_one(&app.db_pool)
        .await
        .expect("Failed to fetch data from the database");

    assert!(response.status().is_success());
    assert_eq!(saved.email, "ursula_le_guin@gmail.com");
    assert_eq!(saved.name, "le guin");
}

#[tokio::test]
async fn subscribe_sends_a_confirmation_email_with_a_link() {
    let app = spawn_app().await;
    let body = "name=le%20guin&email=ursula_le_guin%40gmail.com";

    Mock::given(path("/email"))
        .and(method("POST"))
        .respond_with(ResponseTemplate::new(200))
        .expect(1)
        .mount(&app.email_server)
        .await;

    let response = app.post_subscriptions(body.into()).await;

    assert!(response.status().is_success());

    let email_request = &app.email_server.received_requests().await.unwrap()[0];
    let body: serde_json::Value = serde_json::from_slice(&email_request.body).unwrap();

    let get_link = |s: &str| {
        let links: Vec<_> = linkify::LinkFinder::new()
            .links(s)
            .filter(|l| *l.kind() == linkify::LinkKind::Url)
            .collect();

        assert_eq!(links.len(), 1);
        links[0].as_str().to_owned()
    };

    let html_body = get_link(&body["HtmlBody"].as_str().unwrap());
    let text_body = get_link(&body["TextBody"].as_str().unwrap());

    assert_eq!(html_body, text_body);
}
