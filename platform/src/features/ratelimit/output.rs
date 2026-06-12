use async_trait::async_trait;
use thiserror::Error;

use super::*;

#[cfg_attr(test, mockall::automock)]
#[async_trait]
pub trait Port: Send + Sync + 'static {
    /// Check whether `key` has exceeded `max_requests` within `window_secs`.
    /// Returns `Ok(Some(ttl))` with the remaining TTL in seconds if rate limited,
    /// or `Ok(None)` if the request is allowed.
    async fn check_rate_limit(
        &self,
        key: &str,
        max_requests: u64,
        window_secs: u64,
    ) -> Result<Option<u64>, PortError>;

    /// Record a failed login attempt for `identity`.
    /// If the number of attempts exceeds `max_attempts` within `window_secs`,
    /// a lockout key is set with TTL `lockout_secs`.
    async fn record_failed_attempt(
        &self,
        identity: &str,
        max_attempts: u64,
        window_secs: u64,
        lockout_secs: u64,
    ) -> Result<FailedAttemptStatus, PortError>;

    /// Clear all failed-attempt counters and any lockout for `identity`.
    async fn clear_failed_attempts(&self, identity: &str) -> Result<(), PortError>;

    /// Check whether `identity` is currently locked out.
    /// Returns `Ok(Some(ttl))` with the remaining TTL in seconds if locked,
    /// or `Ok(None)` if not locked.
    async fn is_locked_out(&self, identity: &str) -> Result<Option<u64>, PortError>;
}

#[derive(Error, Debug)]
pub enum PortError {
    #[error("Rate limit: {0}")]
    Internal(String),
}