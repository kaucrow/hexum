use serde_json::json;
use integration_tests::{spawn_test_app, TestApp};

#[tokio::test]
async fn verify_user_with_valid_code() {
    let app = spawn_test_app().await;
    let username = TestApp::unique_username();
    let email = TestApp::unique_email();

    // Seed an unverified user directly in the database (with authenticator)
    let user_id = app.seed_unverified_user_with_password(&username, &email).await;

    // Store a verification code in Redis (same format as the app)
    let code = "042739";
    app.store_verification_token(&user_id, code).await;

    // Call the verify endpoint
    let verify_resp = app
        .post_json(
            "/user/verify-account",
            &json!({ "code": code }),
        )
        .await;
    assert!(
        verify_resp.status().is_success(),
        "Expected 2xx, got {}",
        verify_resp.status()
    );

    // Confirm the authenticator is now verified
    let is_verified = app.is_user_verified(&user_id).await;
    assert_eq!(is_verified, Some(true));

    // Cleanup Redis + Postgres
    app.delete_verification_token(code).await;
    app.delete_user_by_email(&email).await;
}

#[tokio::test]
async fn verify_user_invalid_code_returns_401() {
    let app = spawn_test_app().await;

    let response = app
        .post_json(
            "/user/verify-account",
            &json!({ "code": "999999" }),
        )
        .await;

    assert_eq!(response.status(), 401);
}