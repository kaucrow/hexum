use integration_tests::{spawn_test_app, TestApp};
use serde_json::json;
use uuid::Uuid;

/// End-to-end test covering the full authentication lifecycle:
/// seed user -> verify -> login -> refresh → logout -> stale refresh rejected
#[tokio::test]
async fn full_auth_lifecycle() {
    let app = spawn_test_app().await;
    let username = TestApp::unique_username();
    let email = TestApp::unique_email();

    // ── Seed an unverified user & store verification token in Redis ──
    let user_id = app.seed_unverified_user_with_password(&username, &email).await;

    let token = Uuid::new_v4().to_string();
    app.store_verification_token(&user_id, &token).await;

    // ── Verify ───────────────────────────────────────────────
    let verify_resp = app.get(&format!("/user/verify?token={token}")).await;
    assert!(
        verify_resp.status().is_success(),
        "Verification failed: {}",
        verify_resp.status()
    );

    // Confirm DB state
    assert_eq!(app.is_user_verified(&user_id).await, Some(true));

    // ── Login ────────────────────────────────────────────────
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
        "Login failed: {}",
        login_resp.status()
    );

    // The cookie jar has stored the cookies automatically.
    // Verify we got Set-Cookie headers.
    let set_cookies = login_resp.headers().get_all("set-cookie");
    assert!(
        set_cookies.iter().count() >= 2,
        "Expected at least 2 Set-Cookie headers (access_token + refresh_token)"
    );

    // ─── Refresh session ──────────────────────────────────────
    let refresh_resp = app.post_json("/auth/refresh-session", &json!({})).await;
    assert!(
        refresh_resp.status().is_success(),
        "Refresh failed: {}",
        refresh_resp.status()
    );

    // ── Logout ───────────────────────────────────────────────
    let logout_resp = app.post_json("/auth/logout", &json!({})).await;
    assert!(
        logout_resp.status().is_success(),
        "Logout failed: {}",
        logout_resp.status()
    );

    // ── Try to refresh again — should fail (session invalidated) ──
    let stale_refresh_resp = app.post_json("/auth/refresh-session", &json!({})).await;
    assert_eq!(
        stale_refresh_resp.status(),
        401,
        "Stale refresh should return 401"
    );

    // Cleanup
    app.delete_user_by_email(&email).await;
}
