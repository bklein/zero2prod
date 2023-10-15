use crate::helpers::{assert_is_redirect_to_, spawn_app, spawn_app_logged_in};
use zero2prod::paths::{self, Path};

#[tokio::test]
async fn you_must_be_logged_in_to_access_the_admin_dashboard() {
    let app = spawn_app().await;
    let response = app.get_admin_dashboard().await;

    assert_is_redirect_to_(&response, "/login");
}

#[tokio::test]
async fn dashboard_has_a_link_to_create_newsletter() {
    let app = spawn_app().await;
    app.login_test_user().await;

    let html = app.get_admin_dashboard_html().await;
    assert!(html.contains(&format!(
        r#"href="{}""#,
        paths::path_uri(Path::AdminNewsletters)
    )));
}

#[tokio::test]
async fn new_newsletter_must_have_title() {
    let app = spawn_app_logged_in().await;

    let invalid_body = serde_json::json!({
        "title": "",
        "text": "Newsletter plain text body.",
        "html": "<p>Newsletter html body.",
        "idempotency_key": uuid::Uuid::new_v4().to_string(),
    });
    let response = app.post_newsletters(&invalid_body).await;
    assert_is_redirect_to_(&response, "/admin/newsletters");

    let html_page = app.get_newsletters_html().await;
    assert!(html_page.contains("<p><i>The newsletter must have a title.</i></p>"));
}

#[tokio::test]
async fn new_newsletter_must_have_text_content() {
    let app = spawn_app_logged_in().await;

    let invalid_body = serde_json::json!({
        "title": "Newsletter title",
        "text": "",
        "html": "<p>Newsletter html body.",
        "idempotency_key": uuid::Uuid::new_v4().to_string(),
    });
    let response = app.post_newsletters(&invalid_body).await;
    assert_is_redirect_to_(&response, "/admin/newsletters");

    let html_page = app.get_newsletters_html().await;
    assert!(html_page.contains("<p><i>The newsletter must have text content.</i></p>"));
}

#[tokio::test]
async fn new_newsletter_must_have_html_content() {
    let app = spawn_app_logged_in().await;

    let invalid_body = serde_json::json!({
        "title": "Newsletter title",
        "text": "Newsletter plain text body.",
        "html": "",
        "idempotency_key": uuid::Uuid::new_v4().to_string(),
    });
    let response = app.post_newsletters(&invalid_body).await;
    assert_is_redirect_to_(&response, "/admin/newsletters");

    let html_page = app.get_newsletters_html().await;
    assert!(html_page.contains("<p><i>The newsletter must have HTML content.</i></p>"));
}

#[tokio::test]
async fn logout_clears_session_state() {
    let app = spawn_app().await;

    let login_body = serde_json::json!({
        "username": &app.test_user.username,
        "password": &app.test_user.password,
    });

    let response = app.post_login(&login_body).await;
    assert_is_redirect_to_(&response, "/admin/dashboard");

    let html_page = app.get_admin_dashboard_html().await;
    assert!(html_page.contains(&format!("Welcome {}", &app.test_user.username)));

    let response = app.post_logout().await;
    assert_is_redirect_to_(&response, "/login");

    let html_page = app.get_login_html().await;
    assert!(html_page.contains(r#"<p><i>You have successfully logged out.</i></p>"#));

    let response = app.get_admin_dashboard().await;
    assert_is_redirect_to_(&response, "/login");
}
