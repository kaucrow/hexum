use uuid::Uuid;

use crate::prelude::*;
use crate::features::*;
use crate::features::auth::*;

// ==================================================================
// Helpers
// ==================================================================

fn make_test_user(id: Uuid) -> user::User {
    user::User {
        id,
        username: user::Username::new("testuser".into()).unwrap(),
        email: user::EmailAddress::new("test@example.com".into()).unwrap(),
        roles: vec![user::Role::BasicUser],
        is_active: true,
    }
}

fn make_service(
    user_repo: user::MockRepository,
    session: session::MockPort,
    security: security::MockPort,
    oauth: oauth::MockPort,
) -> auth::Service {
    auth::Service::new(
        Arc::new(user_repo),
        Arc::new(session),
        Arc::new(security),
        Arc::new(oauth),
    )
}

/// Helper: sets up mocks for a successful login flow and returns the service.
/// The passed `password` is what `verify_password` will check against.
fn setup_successful_login(
    password: &'static str,
    user_id: Uuid,
) -> auth::Service {
    let test_user = make_test_user(user_id);

    let mut user_repo = user::MockRepository::new();
    user_repo.expect_get_user_by_username()
        .returning(move |_| Ok(Some(test_user.clone())));
    {
        let uid = user_id;
        user_repo.expect_get_authenticator()
            .returning(move |_, _| {
                let mut auth = user::UserAuthenticator::new_local(uid, format!("hashed:{password}"));
                auth.is_verified = Some(true);
                Ok(Some(auth))
            });
    }

    let mut session = session::MockPort::new();
    session.expect_store_session()
        .returning(|_, _, _| Ok(()));

    let mut security = security::MockPort::new();
    let pwd = password;
    security.expect_verify_password()
        .returning(move |pass, _| pass == pwd);
    security.expect_generate_access_token()
        .returning(|id| Ok(format!("access_token:{id}")));
    security.expect_generate_refresh_token()
        .returning(|| "refresh_token_123".to_string());

    let oauth = oauth::MockPort::new();

    make_service(user_repo, session, security, oauth)
}

// ==================================================================
// login_user — Core Flows
// ==================================================================

#[tokio::test]
async fn test_login_user_by_username_success() {
    let user_id = Uuid::new_v4();
    let test_user = make_test_user(user_id);

    let mut user_repo = user::MockRepository::new();
    user_repo.expect_get_user_by_username()
        .returning(move |_| Ok(Some(test_user.clone())));
    {
        let uid = user_id;
        user_repo.expect_get_authenticator()
            .returning(move |_, _| {
                let mut auth = user::UserAuthenticator::new_local(uid, "hashed:pass".into());
                auth.is_verified = Some(true);
                Ok(Some(auth))
            });
    }

    let mut session = session::MockPort::new();
    session.expect_store_session()
        .returning(|_, _, _| Ok(()));

    let mut security = security::MockPort::new();
    security.expect_verify_password()
        .returning(|_, _| true);
    security.expect_generate_access_token()
        .returning(|id| Ok(format!("access_token:{id}")));
    security.expect_generate_refresh_token()
        .returning(|| "refresh_token_123".to_string());

    let oauth = oauth::MockPort::new();

    let service = auth::Service::new(
        Arc::new(user_repo),
        Arc::new(session),
        Arc::new(security),
        Arc::new(oauth),
    );

    let result = service.login_user("testuser", "password123").await;
    assert!(result.is_ok());

    let tokens = result.unwrap();
    assert!(tokens.access_token.starts_with("access_token:"));
    assert_eq!(tokens.refresh_token, "refresh_token_123");
}

#[tokio::test]
async fn test_login_user_by_email_success() {
    let user_id = Uuid::new_v4();
    let test_user = make_test_user(user_id);

    let mut user_repo = user::MockRepository::new();
    user_repo.expect_get_user_by_username()
        .returning(|_| Ok(None));
    user_repo.expect_get_user_by_email()
        .returning(move |_| Ok(Some(test_user.clone())));
    {
        let uid = user_id;
        user_repo.expect_get_authenticator()
            .returning(move |_, _| {
                let mut auth = user::UserAuthenticator::new_local(uid, "hashed:pass".into());
                auth.is_verified = Some(true);
                Ok(Some(auth))
            });
    }

    let mut session = session::MockPort::new();
    session.expect_store_session()
        .returning(|_, _, _| Ok(()));

    let mut security = security::MockPort::new();
    security.expect_verify_password()
        .returning(|_, _| true);
    security.expect_generate_access_token()
        .returning(|id| Ok(format!("access_token:{id}")));
    security.expect_generate_refresh_token()
        .returning(|| "refresh_token_123".to_string());

    let oauth = oauth::MockPort::new();

    let service = auth::Service::new(
        Arc::new(user_repo),
        Arc::new(session),
        Arc::new(security),
        Arc::new(oauth),
    );

    let result = service.login_user("test@example.com", "password123").await;
    assert!(result.is_ok());
}

#[tokio::test]
async fn test_login_user_not_found() {
    let mut user_repo = user::MockRepository::new();
    user_repo.expect_get_user_by_username()
        .returning(|_| Ok(None));
    user_repo.expect_get_user_by_email()
        .returning(|_| Ok(None));

    let session = session::MockPort::new();
    let security = security::MockPort::new();
    let oauth = oauth::MockPort::new();

    let service = auth::Service::new(
        Arc::new(user_repo),
        Arc::new(session),
        Arc::new(security),
        Arc::new(oauth),
    );

    let result = service.login_user("unknown", "password123").await;
    assert!(matches!(result, Err(auth::UseCaseError::UserNotFound)));
}

#[tokio::test]
async fn test_login_user_inactive() {
    let mut test_user = make_test_user(Uuid::new_v4());
    test_user.is_active = false;

    let mut user_repo = user::MockRepository::new();
    user_repo.expect_get_user_by_username()
        .returning(move |_| Ok(Some(test_user.clone())));

    let session = session::MockPort::new();
    let security = security::MockPort::new();
    let oauth = oauth::MockPort::new();

    let service = auth::Service::new(
        Arc::new(user_repo),
        Arc::new(session),
        Arc::new(security),
        Arc::new(oauth),
    );

    let result = service.login_user("testuser", "password123").await;
    assert!(matches!(result, Err(auth::UseCaseError::UserInactive)));
}

#[tokio::test]
async fn test_login_user_not_verified() {
    let user_id = Uuid::new_v4();
    let test_user = make_test_user(user_id);

    let mut user_repo = user::MockRepository::new();
    user_repo.expect_get_user_by_username()
        .returning(move |_| Ok(Some(test_user.clone())));
    {
        let uid = user_id;
        user_repo.expect_get_authenticator()
            .returning(move |_, _| {
                let mut auth = user::UserAuthenticator::new_local(uid, "hashed:pass".into());
                auth.is_verified = Some(false);
                Ok(Some(auth))
            });
    }

    let session = session::MockPort::new();
    let security = security::MockPort::new();
    let oauth = oauth::MockPort::new();

    let service = auth::Service::new(
        Arc::new(user_repo),
        Arc::new(session),
        Arc::new(security),
        Arc::new(oauth),
    );

    let result = service.login_user("testuser", "password123").await;
    assert!(matches!(result, Err(auth::UseCaseError::UserNotVerified)));
}

#[tokio::test]
async fn test_login_user_invalid_password() {
    let user_id = Uuid::new_v4();
    let test_user = make_test_user(user_id);

    let mut user_repo = user::MockRepository::new();
    user_repo.expect_get_user_by_username()
        .returning(move |_| Ok(Some(test_user.clone())));
    {
        let uid = user_id;
        user_repo.expect_get_authenticator()
            .returning(move |_, _| {
                let mut auth = user::UserAuthenticator::new_local(uid, "hashed:pass".into());
                auth.is_verified = Some(true);
                Ok(Some(auth))
            });
    }

    let session = session::MockPort::new();
    let mut security = security::MockPort::new();
    security.expect_verify_password()
        .returning(|_, _| false);

    let oauth = oauth::MockPort::new();

    let service = auth::Service::new(
        Arc::new(user_repo),
        Arc::new(session),
        Arc::new(security),
        Arc::new(oauth),
    );

    let result = service.login_user("testuser", "wrongpassword").await;
    assert!(matches!(result, Err(auth::UseCaseError::InvalidPassword)));
}

// ==================================================================
// login_user — Password Edge Cases
// ==================================================================

// ------------------------------------------------------------------
// Valid-Format Passwords
// ------------------------------------------------------------------

#[tokio::test]
async fn test_login_with_valid_format_password_success() {
    let user_id = Uuid::new_v4();
    let password = "C0rrect-Horse!";
    let service = setup_successful_login(password, user_id);

    let result = service.login_user("testuser", password).await;
    assert!(result.is_ok(), "Login with valid-format password should succeed");
}

#[tokio::test]
async fn test_login_with_valid_format_password_mismatch() {
    let user_id = Uuid::new_v4();
    let service = setup_successful_login("C0rrect-Horse!", user_id);

    let result = service.login_user("testuser", "Wrong-Horse1!").await;
    assert!(matches!(result, Err(auth::UseCaseError::InvalidPassword)),
        "Mismatched valid-format password should fail");
}

#[tokio::test]
async fn test_login_with_punctuation_mixed_password_success() {
    let user_id = Uuid::new_v4();
    let password = "MyP@ssword#2024!";
    let service = setup_successful_login(password, user_id);

    let result = service.login_user("testuser", password).await;
    assert!(result.is_ok(), "Login with mixed-punctuation password should succeed");
}

// ------------------------------------------------------------------
// Empty and Edge Identity Values
// ------------------------------------------------------------------

#[tokio::test]
async fn test_login_with_empty_identity_not_found() {
    let mut user_repo = user::MockRepository::new();
    user_repo.expect_get_user_by_username()
        .returning(|_| Ok(None));
    user_repo.expect_get_user_by_email()
        .returning(|_| Ok(None));

    let session = session::MockPort::new();
    let security = security::MockPort::new();
    let oauth = oauth::MockPort::new();

    let service = make_service(user_repo, session, security, oauth);

    let result = service.login_user("", "password123").await;
    assert!(matches!(result, Err(auth::UseCaseError::UserNotFound)));
}

// ==================================================================
// verify_user
// ==================================================================

#[tokio::test]
async fn test_verify_user_success() {
    let user_id = Uuid::new_v4();
    let test_user = make_test_user(user_id);

    let mut user_repo = user::MockRepository::new();
    user_repo.expect_get_user_by_id()
        .returning(move |_| Ok(Some(test_user.clone())));

    let mut security = security::MockPort::new();
    security.expect_verify_access_token()
        .returning(move |_| Ok(user_id));

    let session = session::MockPort::new();
    let oauth = oauth::MockPort::new();

    let service = auth::Service::new(
        Arc::new(user_repo),
        Arc::new(session),
        Arc::new(security),
        Arc::new(oauth),
    );

    let result = service.verify_user("valid-access-token").await;
    assert!(result.is_ok());

    let user = result.unwrap();
    assert_eq!(user.username.as_str(), "testuser");
}

#[tokio::test]
async fn test_verify_user_inactive() {
    let mut test_user = make_test_user(Uuid::new_v4());
    test_user.is_active = false;

    let mut user_repo = user::MockRepository::new();
    user_repo.expect_get_user_by_id()
        .returning(move |_| Ok(Some(test_user.clone())));

    let mut security = security::MockPort::new();
    security.expect_verify_access_token()
        .returning(|_| Ok(Uuid::new_v4()));

    let session = session::MockPort::new();
    let oauth = oauth::MockPort::new();

    let service = auth::Service::new(
        Arc::new(user_repo),
        Arc::new(session),
        Arc::new(security),
        Arc::new(oauth),
    );

    let result = service.verify_user("some-token").await;
    assert!(matches!(result, Err(auth::UseCaseError::UserInactive)));
}

#[tokio::test]
async fn test_verify_user_not_found() {
    let mut user_repo = user::MockRepository::new();
    user_repo.expect_get_user_by_id()
        .returning(|_| Ok(None));

    let mut security = security::MockPort::new();
    security.expect_verify_access_token()
        .returning(|_| Ok(Uuid::new_v4()));

    let session = session::MockPort::new();
    let oauth = oauth::MockPort::new();

    let service = auth::Service::new(
        Arc::new(user_repo),
        Arc::new(session),
        Arc::new(security),
        Arc::new(oauth),
    );

    let result = service.verify_user("some-token").await;
    assert!(matches!(result, Err(auth::UseCaseError::UserNotFound)));
}

// ==================================================================
// refresh_session
// ==================================================================

#[tokio::test]
async fn test_refresh_session_success() {
    let user_id = Uuid::new_v4();
    let test_user = make_test_user(user_id);

    let mut user_repo = user::MockRepository::new();
    user_repo.expect_get_user_by_id()
        .returning(move |_| Ok(Some(test_user.clone())));

    let mut session = session::MockPort::new();
    session.expect_consume_session()
        .returning(move |_| Ok(Some(user_id)));
    session.expect_store_session()
        .returning(|_, _, _| Ok(()));

    let mut security = security::MockPort::new();
    security.expect_generate_access_token()
        .returning(|id| Ok(format!("access_token:{id}")));
    security.expect_generate_refresh_token()
        .returning(|| "refresh_token_123".to_string());

    let oauth = oauth::MockPort::new();

    let service = auth::Service::new(
        Arc::new(user_repo),
        Arc::new(session),
        Arc::new(security),
        Arc::new(oauth),
    );

    let result = service.refresh_session("refresh-token").await;
    assert!(result.is_ok());
}

#[tokio::test]
async fn test_refresh_session_invalid_token() {
    let user_repo = user::MockRepository::new();

    let mut session = session::MockPort::new();
    session.expect_consume_session()
        .returning(|_| Ok(None));

    let security = security::MockPort::new();
    let oauth = oauth::MockPort::new();

    let service = auth::Service::new(
        Arc::new(user_repo),
        Arc::new(session),
        Arc::new(security),
        Arc::new(oauth),
    );

    let result = service.refresh_session("invalid-token").await;
    assert!(matches!(result, Err(auth::UseCaseError::InvalidRefreshToken)));
}

#[tokio::test]
async fn test_refresh_session_inactive_user() {
    let mut test_user = make_test_user(Uuid::new_v4());
    test_user.is_active = false;

    let mut user_repo = user::MockRepository::new();
    user_repo.expect_get_user_by_id()
        .returning(move |_| Ok(Some(test_user.clone())));

    let mut session = session::MockPort::new();
    session.expect_consume_session()
        .returning(|_| Ok(Some(Uuid::new_v4())));

    let security = security::MockPort::new();
    let oauth = oauth::MockPort::new();

    let service = auth::Service::new(
        Arc::new(user_repo),
        Arc::new(session),
        Arc::new(security),
        Arc::new(oauth),
    );

    let result = service.refresh_session("some-token").await;
    assert!(matches!(result, Err(auth::UseCaseError::UserInactive)));
}

// ==================================================================
// logout_user
// ==================================================================

#[tokio::test]
async fn test_logout_user_success() {
    let user_repo = user::MockRepository::new();

    let mut session = session::MockPort::new();
    session.expect_consume_session()
        .returning(|_| Ok(Some(Uuid::new_v4())));

    let security = security::MockPort::new();
    let oauth = oauth::MockPort::new();

    let service = auth::Service::new(
        Arc::new(user_repo),
        Arc::new(session),
        Arc::new(security),
        Arc::new(oauth),
    );

    let result = service.logout_user("refresh-token").await;
    assert!(result.is_ok());
}

#[tokio::test]
async fn test_logout_user_invalid_token() {
    let user_repo = user::MockRepository::new();

    let mut session = session::MockPort::new();
    session.expect_consume_session()
        .returning(|_| Ok(None));

    let security = security::MockPort::new();
    let oauth = oauth::MockPort::new();

    let service = auth::Service::new(
        Arc::new(user_repo),
        Arc::new(session),
        Arc::new(security),
        Arc::new(oauth),
    );

    let result = service.logout_user("invalid-token").await;
    assert!(matches!(result, Err(auth::UseCaseError::InvalidRefreshToken)));
}