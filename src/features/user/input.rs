use async_trait::async_trait;
use thiserror::Error;

use super::User;

#[async_trait]
pub trait UseCase: Send + Sync + 'static {
    async fn register_user(&self, user: User, passwd: &str) -> Result<(), UseCaseError>;
    async fn verify_user_account(&self, token: &str) -> Result<(), UseCaseError>;
}

#[derive(Error, Debug)]
pub enum UseCaseError {
    #[error("The username provided is already in use.")]
    UsernameInUse,
    #[error("The email provided is already in use.")]
    EmailInUse,
    #[error("UserUseCase: {0}.")]
    Internal(String),
}