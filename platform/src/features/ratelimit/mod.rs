mod input;
mod output;
mod redis;
mod service;

pub use input::*;
pub use output::*;
pub use redis::RedisAdapter;
pub use service::Service;

/// Status returned after recording a failed login attempt.
#[derive(Debug, Clone)]
pub struct FailedAttemptStatus {
    /// Number of failed attempts recorded so far in the current window.
    pub attempts: u64,
    /// Whether the identity is now locked out.
    pub is_locked: bool,
    /// Seconds until the lockout expires (only meaningful if `is_locked` is true).
    pub lockout_remaining_secs: Option<u64>,
}