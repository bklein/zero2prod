use crate::helpers::{assert_is_redirect_to_, spawn_app};

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
    assert!(html.contains(r#"href="/admin/newsletters""#));
}
