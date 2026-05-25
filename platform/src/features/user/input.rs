use async_trait::async_trait;
use thiserror::Error;

use super::{User, UserError, ConflictError};

#[async_trait]
pub trait UseCase: Send + Sync + 'static {
    async fn register_user(&self, user: User, passwd: &str) -> Result<(), UseCaseError>;
    async fn verify_user_account(&self, token: &str) -> Result<(), UseCaseError>;
}

#[derive(Error, Debug)]
pub enum UseCaseError {
    /// Resource conflict (e.g., duplicate username/email).
    #[error(transparent)]
    Conflict(#[from] ConflictError),

    /// Domain validation failure (e.g., invalid password format).
    #[error(transparent)]
    Validation(#[from] UserError),

    /// User account verification token is invalid/expired.
    #[error("The verification token is invalid or expired.")]
    VerificationTokenInvalid,

    /// Unexpected internal error.
    #[error("User service: {0}.")]
    Internal(String),
}