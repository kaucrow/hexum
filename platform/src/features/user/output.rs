use async_trait::async_trait;
use thiserror::Error;
use uuid::Uuid;

use super::{
    NewUserData,
    User,
    EmailAddress,
    UserAuthenticator,
    AuthProvider,
};

#[cfg_attr(test, mockall::automock)]
#[async_trait]
pub trait Repository: Send + Sync + 'static {
    // ─── Getters ───
    async fn get_user_by_id(&self, id: &Uuid) -> Result<Option<User>, RepositoryError>;
    async fn get_user_by_username(&self, username: &str) -> Result<Option<User>, RepositoryError>;
    async fn get_user_by_email(&self, email: &EmailAddress) -> Result<Option<User>, RepositoryError>;

    // ─── User modification ───
    async fn add_new_user(&self, user: User) -> Result<(), RepositoryError>;
    async fn delete_user_by_id(&self, id: &Uuid) -> Result<Option<Uuid>, RepositoryError>;
    async fn update_user_email(&self, user_id: &Uuid, new_email: &EmailAddress) -> Result<(), RepositoryError>;
    async fn update_user_data(&self, user_id: &Uuid, new_data: NewUserData) -> Result<(), RepositoryError>;

    // ─── Authentication ───
    async fn get_authenticator(
        &self,
        user_id: &Uuid,
        auth_provider: AuthProvider,
    ) -> Result<Option<UserAuthenticator>, RepositoryError>;

    async fn add_authenticator(
        &self,
        user_authenticator: UserAuthenticator
    ) -> Result<(), RepositoryError>;

    async fn verify_local_auth_by_user_id(&self, id: &Uuid) -> Result<(), RepositoryError>;
}

/// Port for persisting uploaded image files to storage.
/// The filename is always `{user_id}.{ext}`, so subsequent uploads
/// overwrite the previous picture for the same user.
#[cfg_attr(test, mockall::automock)]
#[async_trait]
pub trait FileStorage: Send + Sync + 'static {
    /// Saves an image to disk and returns the public URL path
    /// (e.g. `"/uploads/05639468-710b-44fe-9fc7-372514e95c37.png"`).
    async fn save_image(
        &self,
        user_id: &Uuid,
        bytes: &[u8],
        content_type: &str,
    ) -> Result<String, FileStorageError>;
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

#[derive(Error, Debug)]
pub enum FileStorageError {
    #[error("Unsupported image MIME type: {0}")]
    UnsupportedMimeType(String),
    #[error("File storage I/O error: {0}")]
    Io(String),
}