use integration_tests::spawn_test_app;
use serde_json::json;

#[tokio::test]
async fn google_login_with_invalid_code_returns_error() {
    let app = spawn_test_app().await;

    // Calling the Google OAuth endpoint with a garbage code should
    // return a 400 Bad Request (invalid code).
    let response = app
        .client
        .post(app.url("/auth/oauth/google/login"))
        .json(&json!({ "code": "invalid_authorization_code" }))
        .send()
        .await
        .expect("Request failed");

    // With real OAuth credentials configured, the adapter will attempt
    // to call Google's token endpoint and fail with an invalid code.
    // This should result in a 400 status.
    assert!(
        response.status().is_client_error(),
        "Expected 4xx, got {}",
        response.status()
    );
}

#[tokio::test]
async fn github_login_with_invalid_code_returns_error() {
    let app = spawn_test_app().await;

    let response = app
        .client
        .post(app.url("/auth/oauth/github/login"))
        .json(&json!({ "code": "invalid_authorization_code" }))
        .send()
        .await
        .expect("Request failed");

    assert!(
        response.status().is_client_error(),
        "Expected 4xx, got {}",
        response.status()
    );
}

#[tokio::test]
async fn oauth_login_ui_endpoint_returns_html() {
    let app = spawn_test_app().await;

    let response = app.get("/auth/oauth/login-ui").await;

    assert!(
        response.status().is_success(),
        "Expected 2xx, got {}",
        response.status()
    );
}

#[tokio::test]
async fn oauth_callback_ui_endpoint_returns_html() {
    let app = spawn_test_app().await;

    let response = app
        .get("/auth/oauth/callback-ui?code=some_code&state=some_state")
        .await;

    assert!(
        response.status().is_success(),
        "Expected 2xx, got {}",
        response.status()
    );
}