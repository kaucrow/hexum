use async_trait::async_trait;
use thiserror::Error;

use crate::features::user;

/// Describes the operation that triggered the verification email,
/// allowing the email adapter to customize the subject and body content.
#[derive(Debug, Clone, Copy)]
pub enum VerificationContext {
    AccountRegistration,
    UsernameChange,
    PasswordReset,
}

impl VerificationContext {
    pub fn subject(&self) -> &'static str {
        match self {
            Self::AccountRegistration => "Verify your NativEat account",
            Self::UsernameChange => "Confirm your username change",
            Self::PasswordReset => "Reset your password",
        }
    }

    pub fn action_name(&self) -> &'static str {
        match self {
            Self::AccountRegistration => "Verify Account",
            Self::UsernameChange => "Change Username",
            Self::PasswordReset => "Reset Password",
        }
    }

    pub fn description(&self) -> &'static str {
        match self {
            Self::AccountRegistration => "to activate your account",
            Self::UsernameChange => "to confirm your new username",
            Self::PasswordReset => "to reset your password",
        }
    }
}

#[cfg_attr(test, mockall::automock)]
#[async_trait]
pub trait Port: Send + Sync + 'static {
    async fn send_verification_email(
        &self,
        to: &user::EmailAddress,
        code: &str,
        context: VerificationContext,
    ) -> Result<(), PortError>;
}

#[derive(Error, Debug)]
pub enum PortError {
    #[error("Email: {0}")]
    Internal(String)
}