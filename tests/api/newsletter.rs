use crate::helpers::{assert_is_redirect_to, spawn_app, ConfirmationLinks, TestApp};
use wiremock::matchers::{any, method, path};
use wiremock::{Mock, ResponseTemplate};

#[tokio::test]
async fn newsletter_are_not_delivered_to_unconfirmed_subscribers() {
    let app = spawn_app().await;

    create_unconfirmed_subscriber(&app).await;

    Mock::given(any())
        .respond_with(ResponseTemplate::new(200))
        .expect(0)
        .mount(&app.email_server)
        .await;

    let newsletter_request_body = serde_json::json!({
        "title": "Newsletter title",
        "html_content": "Newsletter content",
        "content": "Newsletter content",
    });

    let _response = app
        .post_login_form(&serde_json::json!({
            "username": app.test_user.name,
            "password": app.test_user.password,
        }))
        .await;

    let response = app.post_newsletters(newsletter_request_body).await;

    assert_is_redirect_to(&response, "/admin/dashboard");
}

#[tokio::test]
async fn newsletters_are_delivered_to_confirmed_subscribers() {
    let app = spawn_app().await;

    create_confirmed_subscriber(&app).await;

    let newsletter_request_body = serde_json::json!({
        "title": "Newsletter title",
        "html_content": "Newsletter content",
        "content": "Newsletter content",
    });

    Mock::given(any())
        .respond_with(ResponseTemplate::new(200))
        .expect(1)
        .mount(&app.email_server)
        .await;

    let response = app.post_newsletters(newsletter_request_body).await;

    assert_eq!(response.status().as_u16(), 303);
}

#[tokio::test]
async fn newsletter_returns_400_for_invalid_data() {
    let app = spawn_app().await;

    let test_cases = vec![
        (
            serde_json::json!({
                "html_content": "Newsletter content",
                "content": "Newsletter content",
            }),
            "missing title",
        ),
        (
            serde_json::json!({
                "title": "Newsletter title",
            }),
            "missing content",
        ),
    ];
    let _response = app
        .post_login_form(&serde_json::json!({
            "username": app.test_user.name,
            "password": app.test_user.password,
        }))
        .await;

    for (invalid_body, message) in test_cases {
        let response = app.post_newsletters(invalid_body).await;

        assert_eq!(
            response.status().as_u16(),
            400,
            "The API did not fail with 400 Bad Request when the payload was {}.",
            message
        );
    }
}

#[tokio::test]
async fn not_logged_in_cannot_send_newsletter() {
    let app = spawn_app().await;

    let newsletter_request_body = serde_json::json!({
        "title": "Newsletter title",
        "html_content": "Newsletter content",
        "content": "Newsletter content",
    });

    let response = app.post_newsletters(newsletter_request_body).await;

    assert_is_redirect_to(&response, "/login");
}

async fn create_unconfirmed_subscriber(app: &TestApp) -> ConfirmationLinks {
    let body = "name=le%20guin&email=ursula_le_guin%40gmail.com";

    let _mock_guard = Mock::given(path("/email"))
        .and(method("POST"))
        .respond_with(ResponseTemplate::new(200))
        .expect(1)
        .mount_as_scoped(&app.email_server)
        .await;

    let _response = app
        .post_login_form(&serde_json::json!({
            "username": app.test_user.name,
            "password": app.test_user.password,
        }))
        .await;

    app.post_subscriptions(body.into())
        .await
        .error_for_status()
        .unwrap();

    let email_request = &app.email_server.received_requests().await.unwrap()[0];

    app.get_confirmation_links(&email_request)
}

async fn create_confirmed_subscriber(app: &TestApp) {
    let confirmation_link = create_unconfirmed_subscriber(app).await;

    reqwest::get(confirmation_link.html)
        .await
        .unwrap()
        .error_for_status()
        .unwrap();
}
