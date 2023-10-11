use crate::helpers::{
    assert_is_redirect_to_, spawn_app, spawn_app_logged_in, ConfirmationLinks, TestApp,
};
use fake::{
    faker::{internet::en::SafeEmail, name::en::Name},
    Fake,
};
use std::time::Duration;
use wiremock::{
    matchers::{any, method, path},
    Mock, MockBuilder, ResponseTemplate,
};
use zero2prod::paths::{self, Path};

#[tokio::test]
async fn newsletters_are_not_delivered_to_unconfirmed_subscribers() {
    let app = spawn_app_logged_in().await;
    create_unconfirmed_subscriber(&app).await;

    Mock::given(any())
        .respond_with(ResponseTemplate::new(200))
        .expect(0)
        .mount(&app.email_server)
        .await;

    let newsletter_request_body = serde_json::json!({
        "title": "Newsletter title",
        "text": "Newsletter plain text content",
        "html": "<p>Newsletter HTML content.</p>",
        "idempotency_key": uuid::Uuid::new_v4().to_string(),
    });

    let response = app.post_newsletters(&newsletter_request_body).await;
    assert_is_redirect_to_(&response, "/admin/dashboard");

    let html_page = app.get_admin_dashboard_html().await;
    assert!(html_page.contains("Sent newsletter successfully."));
}

#[tokio::test]
async fn newsletters_are_delivered_to_confirmed_subscribers() {
    let app = spawn_app_logged_in().await;
    create_confirmed_subscriber(&app).await;

    Mock::given(any())
        .respond_with(ResponseTemplate::new(200))
        .expect(1)
        .mount(&app.email_server)
        .await;

    let newsletter_request_body = serde_json::json!({
        "title": "Newsletter title",
        "text": "Newsletter plain text content",
        "html": "<p>Newsletter HTML content.</p>",
        "idempotency_key": uuid::Uuid::new_v4().to_string(),
    });

    let response = app.post_newsletters(&newsletter_request_body).await;
    assert_is_redirect_to_(&response, "/admin/dashboard");

    let html_page = app.get_admin_dashboard_html().await;
    assert!(html_page.contains("Sent newsletter successfully."));
}

async fn create_unconfirmed_subscriber(app: &TestApp) -> ConfirmationLinks {
    let name: String = Name().fake();
    let email: String = SafeEmail().fake();
    let body = serde_urlencoded::to_string(&serde_json::json!({
        "name": name,
        "email": email
    }))
    .unwrap();

    let _mock_guard = Mock::given(path("/email"))
        .and(method("POST"))
        .respond_with(ResponseTemplate::new(200))
        .named("Create unconfirmed subscriber")
        .expect(1)
        .mount_as_scoped(&app.email_server)
        .await;

    app.post_subscriptions(body.into())
        .await
        .error_for_status()
        .unwrap();

    let email_request = &app
        .email_server
        .received_requests()
        .await
        .unwrap()
        .pop()
        .unwrap();
    app.get_confirmation_links(email_request)
}

async fn create_confirmed_subscriber(app: &TestApp) {
    let confirmation_links = create_unconfirmed_subscriber(app).await;
    reqwest::get(confirmation_links.html)
        .await
        .unwrap()
        .error_for_status()
        .unwrap();
}

#[tokio::test]
async fn must_be_logged_in_to_get_newsletter_form() {
    let app = spawn_app().await;
    let response = app.get_newsletters().await;

    assert_is_redirect_to_(&response, "/login");
}

#[tokio::test]
async fn must_be_logged_in_to_create_newsletter() {
    let app = spawn_app().await;

    let body = serde_json::json!({
        "title": "Newsletter title",
        "text": "Newsletter plain text content",
        "html": "<p>Newsletter HTML content.</p>",
        "idempotency_key": uuid::Uuid::new_v4().to_string(),
    });
    let response = app.post_newsletters(&body).await;
    assert_is_redirect_to_(&response, "/login");
}

#[tokio::test]
async fn new_newsletter_page_has_form() {
    let app = spawn_app_logged_in().await;

    let html = app.get_newsletters_html().await;
    assert!(html.contains("<form"));
}

#[tokio::test]
async fn newsletter_create_is_idempotent() {
    let app = spawn_app_logged_in().await;
    create_confirmed_subscriber(&app).await;

    Mock::given(any())
        .respond_with(ResponseTemplate::new(200))
        .expect(1)
        .mount(&app.email_server)
        .await;

    let newsletter_request_body = serde_json::json!({
        "title": "Newsletter title",
        "text": "Newsletter plain text content",
        "html": "<p>Newsletter HTML content.</p>",
        "idempotency_key": uuid::Uuid::new_v4().to_string(),
    });

    let response = app.post_newsletters(&newsletter_request_body).await;
    assert_is_redirect_to_(&response, paths::path_uri(Path::AdminDashboard));

    let html_page = app.get_admin_dashboard_html().await;
    assert!(html_page.contains("Sent newsletter successfully."));

    let response = app.post_newsletters(&newsletter_request_body).await;
    assert_is_redirect_to_(&response, paths::path_uri(Path::AdminDashboard));

    let html_page = app.get_admin_dashboard_html().await;
    assert!(html_page.contains("Sent newsletter successfully."));
}

#[tokio::test]
async fn concurrent_form_submission_is_handled_gracefully() {
    let app = spawn_app_logged_in().await;
    create_confirmed_subscriber(&app).await;

    Mock::given(any())
        .respond_with(ResponseTemplate::new(200).set_delay(Duration::from_secs(2)))
        .expect(1)
        .mount(&app.email_server)
        .await;

    let newsletter_request_body = serde_json::json!({
        "title": "Newsletter title",
        "text": "Newsletter plain text content",
        "html": "<p>Newsletter HTML content.</p>",
        "idempotency_key": uuid::Uuid::new_v4().to_string(),
    });

    let response1 = app.post_newsletters(&newsletter_request_body);
    let response2 = app.post_newsletters(&newsletter_request_body);
    let (response1, response2) = tokio::join!(response1, response2);

    assert_eq!(response1.status(), response1.status());
    assert_eq!(
        response1.text().await.unwrap(),
        response2.text().await.unwrap()
    );
}

fn when_sending_an_email() -> MockBuilder {
    Mock::given(path("/email")).and(method("POST"))
}

#[tokio::test]
async fn transient_errors_do_not_cause_duplicate_deliveries_on_retries() {
    let app = spawn_app_logged_in().await;
    create_confirmed_subscriber(&app).await;
    create_confirmed_subscriber(&app).await;

    Mock::given(any())
        .respond_with(ResponseTemplate::new(200))
        .expect(1)
        .mount(&app.email_server)
        .await;

    let newsletter_request_body = serde_json::json!({
        "title": "Newsletter title",
        "text": "Newsletter plain text content",
        "html": "<p>Newsletter HTML content.</p>",
        "idempotency_key": uuid::Uuid::new_v4().to_string(),
    });

    when_sending_an_email()
        .respond_with(ResponseTemplate::new(200))
        .up_to_n_times(1)
        .expect(1)
        .mount(&app.email_server)
        .await;
    when_sending_an_email()
        .respond_with(ResponseTemplate::new(500))
        .up_to_n_times(1)
        .expect(1)
        .mount(&app.email_server)
        .await;

    let response = app.post_newsletters(&newsletter_request_body).await;
    assert_eq!(response.status().as_u16(), 500);

    when_sending_an_email()
        .respond_with(ResponseTemplate::new(200))
        .expect(1)
        .named("Delivery retry")
        .mount(&app.email_server)
        .await;

    let response = app.post_newsletters(&newsletter_request_body).await;
    assert_eq!(response.status().as_u16(), 303);
}
