use async_trait::async_trait;
use thiserror::Error;

#[cfg_attr(test, mockall::automock)]
#[async_trait]
pub trait Port: Send + Sync + 'static {
    /// Stores a verification token mapped to a payload
    /// (e.g., User ID, Email, Password, etc.) with an expiry.
    async fn store_verification_token(
        &self,
        payload: &str,
        token: &str,
        expires_in_secs: u64
    ) -> Result<(), PortError>;

    /// Retrieves the payload from a verification token.
    async fn consume_verification_token(
        &self,
        token: &str
    ) -> Result<String, PortError>;
}

#[derive(Error, Debug)]
pub enum PortError {
    #[error("The verification token is invalid or expired.")]
    VerificationTokenInvalid,

    #[error("The verification code is already in use. Please try again.")]
    CodeInUse,

    #[error("Verification: {0}")]
    Internal(String),
}