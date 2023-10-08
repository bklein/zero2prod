use crate::helpers::{assert_is_redirect_to_, spawn_app};
use uuid::Uuid;
use zero2prod::paths::{path_uri, Path};

#[tokio::test]
async fn you_must_be_logged_in_to_access_the_change_password_form() {
    let app = spawn_app().await;

    let response = app.get_change_password().await;

    assert_is_redirect_to_(&response, path_uri(Path::Login));
}

#[tokio::test]
async fn you_must_be_logged_in_to_change_your_password() {
    let app = spawn_app().await;
    let new_password = Uuid::new_v4().to_string();

    let response = app
        .post_change_password(&serde_json::json!({
            "current_password": Uuid::new_v4().to_string(),
            "new_password": &new_password,
            "new_password_confirmation": &new_password,
        }))
        .await;

    assert_is_redirect_to_(&response, "/login");
}

#[tokio::test]
async fn new_password_fields_must_match() {
    let app = spawn_app().await;
    let new_password = Uuid::new_v4().to_string();
    let new_password_confirmation = Uuid::new_v4().to_string();

    app.post_login(&serde_json::json!({
        "username": &app.test_user.username,
        "password": &app.test_user.password,
    }))
    .await;

    let response = app
        .post_change_password(&serde_json::json!({
            "current_password": &app.test_user.password,
            "new_password": &new_password,
            "new_password_confirmation": &new_password_confirmation,
        }))
        .await;
    assert_is_redirect_to_(&response, "/admin/password");

    let html_page = app.get_change_password_html().await;
    assert!(html_page.contains("<p><i>Password does not match confirmation.</i></p>"));
}

#[tokio::test]
async fn current_password_must_be_valid() {
    let app = spawn_app().await;
    let new_password = Uuid::new_v4().to_string();
    let wrong_password = Uuid::new_v4().to_string();

    app.post_login(&serde_json::json!({
        "username": &app.test_user.username,
        "password": &app.test_user.password,
    }))
    .await;

    let response = app
        .post_change_password(&serde_json::json!({
            "current_password": &wrong_password,
            "new_password": &new_password,
            "new_password_confirmation": &new_password,
        }))
        .await;
    assert_is_redirect_to_(&response, "/admin/password");

    let html_page = app.get_change_password_html().await;
    assert!(html_page.contains("<p><i>The current password is incorrect.</i></p>"));
}

#[tokio::test]
async fn new_password_must_be_at_least_minimum_length() {
    let app = spawn_app().await;
    let new_password: String = "x".repeat(12);

    app.post_login(&serde_json::json!({
        "username": &app.test_user.username,
        "password": &app.test_user.password,
    }))
    .await;

    let response = app
        .post_change_password(&serde_json::json!({
            "current_password": &app.test_user.password,
            "new_password": &new_password,
            "new_password_confirmation": &new_password,
        }))
        .await;
    assert_is_redirect_to_(&response, "/admin/password");

    let html_page = app.get_change_password_html().await;
    assert!(html_page.contains("<p><i>The new password is too short.</i></p>"));
}

#[tokio::test]
async fn new_password_must_not_be_more_than_max_length() {
    let app = spawn_app().await;
    let new_password: String = "x".repeat(129);

    app.post_login(&serde_json::json!({
        "username": &app.test_user.username,
        "password": &app.test_user.password,
    }))
    .await;

    let response = app
        .post_change_password(&serde_json::json!({
            "current_password": &app.test_user.password,
            "new_password": &new_password,
            "new_password_confirmation": &new_password,
        }))
        .await;
    assert_is_redirect_to_(&response, "/admin/password");

    let html_page = app.get_change_password_html().await;
    assert!(html_page.contains("<p><i>The new password is too long.</i></p>"));
}

#[tokio::test]
async fn changing_password_works() {
    let app = spawn_app().await;

    let response = app
        .post_login(&serde_json::json!({
            "username": &app.test_user.username,
            "password": &app.test_user.password,
        }))
        .await;
    assert_is_redirect_to_(&response, "/admin/dashboard");

    let new_password = Uuid::new_v4().to_string();
    let response = app
        .post_change_password(&serde_json::json!({
            "current_password": &app.test_user.password,
            "new_password": &new_password,
            "new_password_confirmation": &new_password,
        }))
        .await;
    assert_is_redirect_to_(&response, "/admin/password");

    let html_page = app.get_change_password_html().await;
    assert!(html_page.contains("Your password has been changed."));

    let response = app.post_logout().await;
    assert_is_redirect_to_(&response, "/login");

    let response = app
        .post_login(&serde_json::json!({
            "username": &app.test_user.username,
            "password": &new_password,
        }))
        .await;
    assert_is_redirect_to_(&response, "/admin/dashboard");
}
