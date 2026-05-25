use uuid::Uuid;

use crate::prelude::*;
use crate::features::*;
use crate::features::user::{
    User, Username, EmailAddress, Role, *
};

fn make_test_user(id: Uuid) -> User {
    User {
        id,
        username: Username::new("testuser".into()).unwrap(),
        email: EmailAddress::new("test@example.com".into()).unwrap(),
        roles: vec![Role::BasicUser],
        is_active: true,
    }
}

#[tokio::test]
async fn test_register_user_success() {
    let mut user_repo = user::MockRepository::new();
    user_repo.expect_add_new_user()
        .returning(|_| Ok(()));
    user_repo.expect_add_authenticator()
        .returning(|_| Ok(()));

    let mut security = security::MockPort::new();
    security.expect_hash_password()
        .returning(|s: &user::Password| Ok(format!("hashed:{}", s.as_str())));
    security.expect_generate_verification_token()
        .returning(|| "verify-token-123".to_string());

    let mut email = email::MockPort::new();
    email.expect_send_verification_email()
        .returning(|_, _| Ok(()));

    let mut verification = verification::MockPort::new();
    verification.expect_store_verification_token()
        .returning(|_, _, _| Ok(()));

    let service = user::Service::new(
        Arc::new(user_repo),
        Arc::new(verification),
        Arc::new(security),
        Arc::new(email),
    );

    let user_id = Uuid::new_v4();
    let user = make_test_user(user_id);

    let result = service.register_user(user, "MyP@ssword123").await;
    assert!(result.is_ok());
}

#[tokio::test]
async fn test_register_user_username_in_use() {
    let mut user_repo = user::MockRepository::new();
    user_repo.expect_add_new_user()
        .returning(|_| Err(user::RepositoryError::Conflict(user::ConflictError::UsernameInUse)));

    let mut security = security::MockPort::new();
    security.expect_hash_password()
        .returning(|s: &user::Password| Ok(format!("hashed:{}", s.as_str())));

    let email = email::MockPort::new();
    let verification = verification::MockPort::new();

    let service = user::Service::new(
        Arc::new(user_repo),
        Arc::new(verification),
        Arc::new(security),
        Arc::new(email),
    );

    let user = make_test_user(Uuid::new_v4());
    let result = service.register_user(user, "MyP@ssword123").await;
    assert!(matches!(result, Err(user::UseCaseError::Conflict(user::ConflictError::UsernameInUse))));
}

#[tokio::test]
async fn test_register_user_email_in_use() {
    let mut user_repo = user::MockRepository::new();
    user_repo.expect_add_new_user()
        .returning(|_| Err(user::RepositoryError::Conflict(user::ConflictError::EmailInUse)));

    let mut security = security::MockPort::new();
    security.expect_hash_password()
        .returning(|s: &user::Password| Ok(format!("hashed:{}", s.as_str())));

    let email = email::MockPort::new();
    let verification = verification::MockPort::new();

    let service = user::Service::new(
        Arc::new(user_repo),
        Arc::new(verification),
        Arc::new(security),
        Arc::new(email),
    );

    let user = make_test_user(Uuid::new_v4());
    let result = service.register_user(user, "MyP@ssword123").await;
    assert!(matches!(result, Err(user::UseCaseError::Conflict(user::ConflictError::EmailInUse))));
}

#[tokio::test]
async fn test_register_user_email_failure_deletes_user() {
    let mut user_repo = user::MockRepository::new();
    user_repo.expect_add_new_user()
        .returning(|_| Ok(()));
    user_repo.expect_add_authenticator()
        .returning(|_| Ok(()));
    user_repo.expect_delete_user_by_id()
        .withf(|_id: &Uuid| true)
        .returning(|_| Ok(()));

    let mut security = security::MockPort::new();
    security.expect_hash_password()
        .returning(|s: &user::Password| Ok(format!("hashed:{}", s.as_str())));
    security.expect_generate_verification_token()
        .returning(|| "verify-token-123".to_string());

    let mut email = email::MockPort::new();
    email.expect_send_verification_email()
        .returning(|_, _| {
            Err(email::PortError::Internal("email failed".into()))
        });

    let mut verification = verification::MockPort::new();
    verification.expect_store_verification_token()
        .returning(|_, _, _| Ok(()));

    let service = user::Service::new(
        Arc::new(user_repo),
        Arc::new(verification),
        Arc::new(security),
        Arc::new(email),
    );

    let user_id = Uuid::new_v4();
    let user = make_test_user(user_id);
    let result = service.register_user(user, "MyP@ssword123").await;

    assert!(result.is_err());
    assert!(matches!(result, Err(user::UseCaseError::Internal(_))));
}

#[tokio::test]
async fn test_verify_user_account_success() {
    let user_id = Uuid::new_v4();

    let mut user_repo = user::MockRepository::new();
    user_repo.expect_verify_local_auth_by_user_id()
        .returning(|_| Ok(()));

    let security = security::MockPort::new();
    let email = email::MockPort::new();

    let mut verification = verification::MockPort::new();
    verification.expect_consume_verification_token()
        .returning(move |_| Ok(user_id));

    let service = user::Service::new(
        Arc::new(user_repo),
        Arc::new(verification),
        Arc::new(security),
        Arc::new(email),
    );

    let result = service.verify_user_account("valid-token").await;
    assert!(result.is_ok());
}

#[tokio::test]
async fn test_verify_user_account_invalid_token() {
    let user_repo = user::MockRepository::new();
    let security = security::MockPort::new();
    let email = email::MockPort::new();

    let mut verification = verification::MockPort::new();
    verification.expect_consume_verification_token()
        .returning(|_| {
            Err(verification::PortError::VerificationTokenInvalid)
        });

    let service = user::Service::new(
        Arc::new(user_repo),
        Arc::new(verification),
        Arc::new(security),
        Arc::new(email),
    );

    let result = service.verify_user_account("bad-token").await;
    assert!(result.is_err());
}