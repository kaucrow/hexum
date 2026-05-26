use integration_tests::spawn_test_app;
use serde_json::json;

#[tokio::test]
async fn refresh_session_missing_cookie_returns_401() {
    let app = spawn_test_app().await;

    let response = app
        .post_json("/auth/refresh-session", &json!({}))
        .await;

    assert_eq!(response.status(), 401);
}

#[tokio::test]
async fn refresh_session_invalid_token_returns_401() {
    let app = spawn_test_app().await;

    // Manually set an invalid refresh_token cookie
    let response = app
        .client
        .post(app.url("/auth/refresh-session"))
        .header(
            "Cookie",
            "refresh_token=invalid_refresh_token_that_does_not_exist",
        )
        .send()
        .await
        .expect("Request failed");

    assert_eq!(response.status(), 401);
}