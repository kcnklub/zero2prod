use crate::helpers::spawn_app;

#[tokio::test]
async fn subscribe_returns_a_200_for_valid_form_data() {
    let test_app = spawn_app().await;

    let body = "name=kcnklub&email=test@test.com";
    let response = test_app.post_subscriptions(body.into()).await;

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
    let test_cases = vec![
        ("name=kcnklub", "missing the email!"),
        ("email=test@test.com", "missing the name!"),
        ("", "missing both inputs"),
    ];

    for (invalid_body, error_message) in test_cases {
        let response = test_app.post_subscriptions(invalid_body.into()).await;

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
    let test_app = spawn_app().await;
    let test_cases = vec![
        ("name=&email=test@test.com", "empty name"),
        ("name=test&email=", "empty email"),
        ("name=test&email=definitely-not-an-email", "invalid email"),
    ];

    for (body, message) in test_cases {
        let response = test_app.post_subscriptions(body.into()).await;

        assert_eq!(
            400,
            response.status().as_u16(),
            "The api did not return a 400 bad request when the payload was {}.",
            message
        );
    }
}
