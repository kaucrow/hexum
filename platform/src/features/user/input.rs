use async_trait::async_trait;
use thiserror::Error;

use crate::prelude::*;
use super::{User, UserError, ConflictError};

#[async_trait]
pub trait UseCase: Send + Sync + 'static {
    // ─── Getters ───
    async fn get_user_by_id(&self, id: &Uuid) -> Result<Option<User>, UseCaseError>;

    // ─── User modification ───
    async fn change_user_email(&self, new_email: &str) -> Result<(), UseCaseError>;
    async fn verify_user_email_change(&self, user_id: &Uuid, code: &str) -> Result<(), UseCaseError>;
    async fn delete_user(&self, user_id: &Uuid) -> Result<Option<Uuid>, UseCaseError>;

    // ─── Registration ───
    async fn register_user(&self, user: User, passwd: &str) -> Result<(), UseCaseError>;
    async fn verify_user_account(&self, code: &str) -> Result<(), UseCaseError>;
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