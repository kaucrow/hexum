use async_trait::async_trait;
use thiserror::Error;

use super::*;

#[cfg_attr(test, mockall::automock)]
#[async_trait]
pub trait UseCase: Send + Sync + 'static {
    /// IP-based rate limit check for a named endpoint.
    /// Uses the configured `ip_max_per_minute` with a 60s window.
    /// Returns `UseCaseError::TooManyRequests` if rate limited,
    /// or `()` if the request is allowed.
    async fn check_ip_limit(&self, ip: &str, endpoint: &str) -> Result<(), UseCaseError>;

    /// Rate limit check with explicit max requests and window.
    async fn check_custom_limit(
        &self,
        key: &str,
        max_requests: u64,
        window_secs: u64,
    ) -> Result<(), UseCaseError>;

    /// Check whether an identity is currently locked out.
    /// Returns `UseCaseError::LockedOut` if locked out, or `()`.
    async fn check_login_lockout(&self, identity: &str) -> Result<(), UseCaseError>;

    /// Record a failed login attempt and apply progressive delay.
    /// Uses configured `login_max_attempts`, `login_window_secs`,
    /// `login_lockout_secs`, and `login_base_delay_ms`.
    async fn record_login_failure(&self, identity: &str) -> Result<FailedAttemptStatus, UseCaseError>;

    /// Clear failed-attempt counters and lockout after a successful login.
    async fn clear_login_failures(&self, identity: &str) -> Result<(), UseCaseError>;
}

#[derive(Error, Debug)]
pub enum UseCaseError {
    #[error("Too many requests. Retry after {retry_after_secs}s.")]
    TooManyRequests { retry_after_secs: u64 },

    #[error("Account is temporarily locked. Retry after {retry_after_secs}s.")]
    LockedOut { retry_after_secs: u64 },

    #[error("Rate limit service: {0}")]
    Internal(String),
}