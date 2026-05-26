use integration_tests::spawn_test_app;

#[tokio::test]
async fn health_check_returns_ok() {
    let app = spawn_test_app().await;

    let response = app.get("/business-health").await;
    assert!(
        response.status().is_success(),
        "Expected 2xx, got {}",
        response.status()
    );
}
