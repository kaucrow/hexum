use async_trait::async_trait;
use thiserror::Error;
use uuid::Uuid;

use super::{User, EmailAddress, UserAuthenticator, AuthProvider};

#[cfg_attr(test, mockall::automock)]
#[async_trait]
pub trait Repository: Send + Sync + 'static {
    // --- Getters ---
    async fn get_user_by_id(&self, id: &Uuid) -> Option<User>;
    async fn get_user_by_username(&self, username: &str) -> Option<User>;
    async fn get_user_by_email(&self, email: &EmailAddress) -> Option<User>;

    // --- User modification ---
    async fn add_new_user(&self, user: User) -> Result<(), RepositoryError>;
    async fn delete_user_by_id(&self, id: &Uuid) -> Result<(), RepositoryError>;

    // --- Authentication ---
    async fn get_authenticator(
        &self,
        user_id: &Uuid,
        auth_provider: AuthProvider,
    ) -> Result<Option<UserAuthenticator>, RepositoryError>;

    async fn verify_local_auth_by_user_id(&self, id: &Uuid) -> Result<(), RepositoryError>;

    async fn add_authenticator(
        &self,
        user_authenticator: UserAuthenticator
    ) -> Result<(), RepositoryError>;
}

#[derive(Error, Debug, Clone, PartialEq)]
pub enum ConflictError {
    #[error("The username provided is already in use.")]
    UsernameInUse,
    #[error("The email provided is already in use.")]
    EmailInUse,
}

#[derive(Error, Debug)]
pub enum RepositoryError {
    #[error(transparent)]
    Conflict(#[from] ConflictError),
    #[error("User repository: {0}")]
    Internal(String),
}