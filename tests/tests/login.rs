use integration_tests::{spawn_test_app, TestApp};
use serde_json::json;

#[tokio::test]
async fn login_user_not_verified() {
    let app = spawn_test_app().await;
    let username = TestApp::unique_username();
    let email = TestApp::unique_email();

    // Seed an unverified user with a known password hash
    app.seed_unverified_user_with_password(&username, &email).await;

    // Try to login. Should fail because email is not verified
    let login_resp = app
        .post_json(
            "/auth/local/login",
            &json!({
                "identity": email,
                "password": "TestPass123!@#",
            }),
        )
        .await;

    assert_eq!(login_resp.status(), 401);

    // Cleanup
    app.delete_user_by_email(&email).await;
}

#[tokio::test]
async fn login_user_not_found() {
    let app = spawn_test_app().await;

    let login_resp = app
        .post_json(
            "/auth/local/login",
            &json!({
                "identity": "nonexistent_user",
                "password": "SomePass123!@#",
            }),
        )
        .await;

    assert_eq!(login_resp.status(), 401);
}

#[tokio::test]
async fn login_user_invalid_password() {
    let app = spawn_test_app().await;
    let username = TestApp::unique_username();
    let email = TestApp::unique_email();

    // Seed a verified user with a known password
    app.seed_verified_user(&username, &email).await;

    // Try to login with wrong password
    let login_resp = app
        .post_json(
            "/auth/local/login",
            &json!({
                "identity": email,
                "password": "WrongPass123!@#",
            }),
        )
        .await;

    assert_eq!(login_resp.status(), 401);

    // Cleanup
    app.delete_user_by_email(&email).await;
}