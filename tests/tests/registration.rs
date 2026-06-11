use serde_json::json;
use integration_tests::{spawn_test_app, TestApp};

#[tokio::test]
async fn register_user_success() {
    let app = spawn_test_app().await;
    let username = TestApp::unique_username();
    let email = TestApp::unique_email();
    let passwd = TestApp::valid_password();

    let response = app
        .post_json(
            "/user/register",
            &json!({
                "username": username,
                "email": email,
                "password": passwd,
            }),
        )
        .await;

    // The endpoint should answer with 200
    let status = response.status();
    assert_eq!(status, 200, "Expected 200, got {status}");

    // Verify the user exists in the database
    let user_id = app.get_user_id_by_email(&email).await;
    assert!(user_id.is_some(), "User should exist in the database");

    // Verify the authenticator exists and is not verified yet
    let is_verified = app.is_user_verified(&user_id.unwrap()).await;
    assert_eq!(is_verified, Some(false));

    // Cleanup
    app.delete_user_by_email(&email).await;
}

#[tokio::test]
async fn register_user_duplicate_username() {
    let app = spawn_test_app().await;
    let username = TestApp::unique_username();
    let email1 = TestApp::unique_email();
    let email2 = TestApp::unique_email();
    let passwd = TestApp::valid_password();

    // Seed the first user directly in DB (bypasses email requirement)
    app.seed_unverified_user(&username, &email1).await;

    // Try registering a second user with the same username
    let resp2 = app
        .post_json(
            "/user/register",
            &json!({
                "username": username,
                "email": email2,
                "password": passwd,
            }),
        )
        .await;

    assert_eq!(
        resp2.status(),
        409,
        "Expected 409 Conflict for duplicate username"
    );

    // Cleanup
    app.delete_user_by_email(&email1).await;
    app.delete_user_by_email(&email2).await;
}

#[tokio::test]
async fn register_user_duplicate_email() {
    let app = spawn_test_app().await;
    let username1 = TestApp::unique_username();
    let username2 = TestApp::unique_username();
    let email = TestApp::unique_email();
    let passwd = TestApp::valid_password();

    // Seed the first user directly in DB (bypasses email requirement)
    app.seed_unverified_user(&username1, &email).await;

    // Try registering a second user with the same email
    let resp2 = app
        .post_json(
            "/user/register",
            &json!({
                "username": username2,
                "email": email,
                "password": passwd,
            }),
        )
        .await;

    assert_eq!(
        resp2.status(),
        409,
        "Expected 409 Conflict for duplicate email"
    );

    // Cleanup
    app.delete_user_by_email(&email).await;
}

#[tokio::test]
async fn register_user_invalid_username() {
    let app = spawn_test_app().await;
    let email = TestApp::unique_email();
    let passwd = TestApp::valid_password();

    let response = app
        .post_json(
            "/user/register",
            &json!({
                "username": "invalid username with spaces!",
                "email": email,
                "password": passwd,
            }),
        )
        .await;

    assert_eq!(response.status(), 422);

    // Ensure no user was persisted
    let user_id = app.get_user_id_by_email(&email).await;
    assert!(user_id.is_none(), "User should NOT exist in the database");
}

#[tokio::test]
async fn register_user_invalid_password() {
    let app = spawn_test_app().await;
    let username = TestApp::unique_username();
    let email = TestApp::unique_email();

    let response = app
        .post_json(
            "/user/register",
            &json!({
                "username": username,
                "email": email,
                "password": "short",
            }),
        )
        .await;

    assert_eq!(response.status(), 422);

    // Ensure no user was persisted
    let user_id = app.get_user_id_by_email(&email).await;
    assert!(user_id.is_none(), "User should NOT exist in the database");
}