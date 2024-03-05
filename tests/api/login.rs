use crate::helpers::{assert_is_redirect_to, spawn_app};

#[tokio::test]
async fn an_error_flash_message_is_set_on_failure() {
    let app = spawn_app().await;

    let login_body = serde_json::json!({
        "username": "username",
        "password": "password"
    });
    let response = app.post_login_form(&login_body).await;

    assert_is_redirect_to(&response, "/login");

    let login_text = app.get_login_html().await;
    assert!(login_text.contains("Authentication failed"));

    let login_text = app.get_login_html().await;
    assert!(!login_text.contains("Authentication failed"));
}
