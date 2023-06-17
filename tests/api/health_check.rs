use crate::helpers::spawn_app;

#[tokio::test]
async fn health_check_succeeds() {
    let app = spawn_app().await;
    let client = reqwest::Client::new();

    let response = client
        .get(&format!("{}/health_check", &app.address))
        .send()
        .await
        .expect("Fail to send");
    assert!(response.status().is_success());
    assert_eq!(Some(0), response.content_length());
}

