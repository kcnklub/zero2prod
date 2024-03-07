use crate::helpers::{assert_is_redirect_to, spawn_app};

#[tokio::test]
async fn you_must_be_logged_in_to_see_the_change_password_form() {
    let app = spawn_app().await;

    let response = app.get_change_password().await;

    assert_is_redirect_to(&response, "/login");
}

#[tokio::test]
async fn you_must_be_logged_in_to_change_your_password() {
    let app = spawn_app().await;

    let response = app
        .post_change_password(&serde_json::json!({
            "password": "password",
            "new_password": "new_password",
            "confirm_password": "new_password"
        }))
        .await;

    assert_is_redirect_to(&response, "/login");
}

#[tokio::test]
async fn new_passwords_must_match() {
    let app = spawn_app().await;

    // login
    app.post_login_form(&serde_json::json!({
        "username": app.test_user.name,
        "password": app.test_user.password
    }))
    .await;

    // attempt to change password
    let response = app
        .post_change_password(&serde_json::json!({
            "password": "password",
            "new_password": "new_password",
            "confirm_password": "new_password2"
        }))
        .await;

    assert_is_redirect_to(&response, "/admin/password");

    // follow redirect
    let html_page = app.get_change_password_html().await;
    assert!(html_page.contains("Passwords do not match"));
}

#[tokio::test]
async fn current_password_must_be_valid() {
    let app = spawn_app().await;

    // login
    app.post_login_form(&serde_json::json!({
        "username": app.test_user.name,
        "password": app.test_user.password
    }))
    .await;

    // attempt to change password
    let response = app
        .post_change_password(&serde_json::json!({
            "password": "password",
            "new_password": "new_password",
            "confirm_password": "new_password"
        }))
        .await;

    assert_is_redirect_to(&response, "/admin/password");

    // follow redirect
    let html_page = app.get_change_password_html().await;
    assert!(html_page.contains("The Current Password is incorrect"));
}

#[tokio::test]
async fn successful_password_change() {
    let app = spawn_app().await;

    // login
    app.post_login_form(&serde_json::json!({
        "username": app.test_user.name,
        "password": app.test_user.password
    }))
    .await;

    // attempt to change password
    let response = app
        .post_change_password(&serde_json::json!({
            "password": app.test_user.password,
            "new_password": "new_password",
            "confirm_password": "new_password"
        }))
        .await;

    assert_is_redirect_to(&response, "/admin/dashboard");

    // follow redirect
    let html_page = app.get_admin_dashboard_html().await;
    assert!(html_page.contains("You have successfully changed your password"));

    // logout
    let response = app.post_logout().await;
    assert_is_redirect_to(&response, "/login");

    // login with new password
    let response = app
        .post_login_form(&serde_json::json!({
            "username": app.test_user.name,
            "password": "new_password"
        }))
        .await;
    assert_is_redirect_to(&response, "/admin/dashboard");

    let html_page = app.get_admin_dashboard_html().await;
    assert!(html_page.contains(&format!("Welcome, {}!", app.test_user.name)));
}
