use integration_tests::{spawn_test_app, TestApp};
use serde_json::json;

#[tokio::test]
async fn logout_without_session_succeeds() {
    let app = spawn_test_app().await;

    // Logout without any cookies. Should still return 200
    let response = app
        .post_json("/auth/logout", &json!({}))
        .await;

    assert!(
        response.status().is_success(),
        "Expected 2xx, got {}",
        response.status()
    );
}

#[tokio::test]
async fn full_login_then_logout_flow() {
    let app = spawn_test_app().await;
    let username = TestApp::unique_username();
    let email = TestApp::unique_email();

    // Seed a verified user directly in the database
    app.seed_verified_user(&username, &email).await;

    // Login: reqwest cookie jar captures the session cookies automatically
    let login_resp = app
        .post_json(
            "/auth/local/login",
            &json!({
                "identity": email,
                "password": "TestPass123!@#",
            }),
        )
        .await;
    assert!(
        login_resp.status().is_success(),
        "Login should succeed for verified user, got {}",
        login_resp.status()
    );

    // Logout: cookies are sent automatically by the client
    let logout_resp = app
        .post_json("/auth/logout", &json!({}))
        .await;
    assert!(
        logout_resp.status().is_success(),
        "Expected 2xx, got {}",
        logout_resp.status()
    );

    // Cleanup
    app.delete_user_by_email(&email).await;
}
