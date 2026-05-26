use integration_tests::{spawn_test_app, TestApp};
use uuid::Uuid;

#[tokio::test]
async fn verify_user_with_valid_token() {
    let app = spawn_test_app().await;
    let username = TestApp::unique_username();
    let email = TestApp::unique_email();

    // Seed an unverified user directly in the database (with authenticator)
    let user_id = app.seed_unverified_user_with_password(&username, &email).await;

    // Store a verification token in Redis (same format as the app)
    let token = Uuid::new_v4().to_string();
    app.store_verification_token(&user_id, &token).await;

    // Call the verify endpoint
    let verify_resp = app.get(&format!("/user/verify?token={token}")).await;
    assert!(
        verify_resp.status().is_success(),
        "Expected 2xx, got {}",
        verify_resp.status()
    );

    // Confirm the authenticator is now verified
    let is_verified = app.is_user_verified(&user_id).await;
    assert_eq!(is_verified, Some(true));

    // Cleanup Redis + Postgres
    app.delete_verification_token(&token).await;
    app.delete_user_by_email(&email).await;
}

#[tokio::test]
async fn verify_user_invalid_token_returns_401() {
    let app = spawn_test_app().await;

    let response = app
        .get("/user/verify?token=invalid_token_that_does_not_exist")
        .await;

    assert_eq!(response.status(), 401);
}
