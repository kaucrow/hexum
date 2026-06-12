use async_trait::async_trait;

use crate::{
    Config,
    prelude::*,
};
use super::*;

#[derive(Clone)]
pub struct Service {
    port: Arc<dyn Port>,
    config: Arc<Config>,
}

impl Service {
    pub fn new(port: Arc<dyn Port>, config: Arc<Config>) -> Self {
        Self { port, config }
    }
}

#[async_trait]
impl UseCase for Service {
    async fn check_ip_limit(&self, ip: &str, endpoint: &str) -> Result<(), UseCaseError> {
        match self
            .port
            .check_rate_limit(
                &format!("ratelimit:ip:{}:{}", ip, endpoint),
                self.config.ratelimit.ip_max_per_minute,
                60,
            )
            .await
        {
            Ok(Some(ttl)) => Err(UseCaseError::TooManyRequests { retry_after_secs: ttl }),
            Ok(None) => Ok(()),
            Err(e) => Err(e.into()),
        }
    }

    async fn check_custom_limit(
        &self,
        key: &str,
        max_requests: u64,
        window_secs: u64,
    ) -> Result<(), UseCaseError> {
        match self
            .port
            .check_rate_limit(key, max_requests, window_secs)
            .await
        {
            Ok(Some(ttl)) => Err(UseCaseError::TooManyRequests { retry_after_secs: ttl }),
            Ok(None) => Ok(()),
            Err(e) => Err(e.into()),
        }
    }

    async fn check_login_lockout(&self, identity: &str) -> Result<(), UseCaseError> {
        match self.port.is_locked_out(identity).await {
            Ok(Some(ttl)) => Err(UseCaseError::LockedOut { retry_after_secs: ttl }),
            Ok(None) => Ok(()),
            Err(e) => Err(e.into()),
        }
    }

    async fn record_login_failure(&self, identity: &str) -> Result<FailedAttemptStatus, UseCaseError> {
        let status = self.port
            .record_failed_attempt(
                identity,
                self.config.ratelimit.login_max_attempts,
                self.config.ratelimit.login_window_secs,
                self.config.ratelimit.login_lockout_secs,
            )
            .await
            .map_err(UseCaseError::from)?;

        // Progressive delay: base_delay * 2^(attempts-1), capped at 30s
        let delay_ms = (self.config.ratelimit.login_base_delay_ms as u128)
            .saturating_mul(2u128.saturating_pow(status.attempts.saturating_sub(1) as u32));
        let delay_ms = delay_ms.min(30_000);
        tokio::time::sleep(tokio::time::Duration::from_millis(delay_ms as u64)).await;

        Ok(status)
    }

    async fn clear_login_failures(&self, identity: &str) -> Result<(), UseCaseError> {
        self.port
            .clear_failed_attempts(identity)
            .await
            .map_err(Into::into)
    }
}

impl From<PortError> for UseCaseError {
    fn from(e: PortError) -> Self {
        match e {
            PortError::Internal(s) => UseCaseError::Internal(s),
        }
    }
}