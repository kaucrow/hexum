use async_trait::async_trait;
use thiserror::Error;
use uuid::Uuid;

#[async_trait]
pub trait Port: Send + Sync + 'static {
    fn verify_password(&self, password: &str, hash: &str) -> bool;
    fn hash(&self, s: &str) -> Result<String, PortError>;
    fn verify_access_token(&self, token: &str) -> Result<Uuid, PortError>;
    fn generate_access_token(&self, user_id: &Uuid) -> Result<String, PortError>;
    fn generate_refresh_token(&self) -> String;
    fn generate_verification_token(&self) -> String;
}

#[derive(Error, Debug)]
pub enum PortError {
    #[error("The token provided is invalid or expired.")]
    TokenVerificationFailed,

    #[error("Security: {0}")]
    Internal(String),
}