use ::redis::{
    AsyncCommands,
    aio::ConnectionManager,
};
use async_trait::async_trait;
use thiserror::Error;

use super::*;

#[derive(Clone)]
pub struct RedisAdapter {
    pub conn: ConnectionManager,
}

impl RedisAdapter {
    pub async fn new(conn: ConnectionManager) -> Self {
        Self { conn }
    }

    fn login_attempts_key(identity: &str) -> String {
        format!("ratelimit:login:{}", identity)
    }

    fn lockout_key(identity: &str) -> String {
        format!("ratelimit:lockout:{}", identity)
    }
}

#[async_trait]
impl Port for RedisAdapter {
    async fn check_rate_limit(
        &self,
        key: &str,
        max_requests: u64,
        window_secs: u64,
    ) -> Result<Option<u64>, PortError> {
        let res: Result<_, LocalError> = async {
            let mut conn = self.conn.clone();

            let count: u64 = conn.incr(key, 1).await?;

            if count == 1 {
                let _: () = conn.expire(key, window_secs as i64).await?;
            }

            if count > max_requests {
                let ttl: i64 = conn.ttl(key).await?;
                let retry_after = if ttl > 0 { ttl as u64 } else { window_secs };
                return Ok(Some(retry_after));
            }

            Ok(None)
        }.await;

        res.map_err(Into::into)
    }

    async fn record_failed_attempt(
        &self,
        identity: &str,
        max_attempts: u64,
        window_secs: u64,
        lockout_secs: u64,
    ) -> Result<FailedAttemptStatus, PortError> {
        let res: Result<_, LocalError> = async {
            let key = Self::login_attempts_key(identity);
            let count: u64 = self.conn.clone().incr(&key, 1).await?;

            if count == 1 {
                let _: () = self.conn.clone().expire(&key, window_secs as i64).await?;
            }

            let is_locked = count >= max_attempts;
            let mut lockout_remaining: Option<u64> = None;

            if is_locked {
                let lock_key = Self::lockout_key(identity);
                let set_result: Option<String> = self.conn.clone()
                    .set_nx(&lock_key, "1")
                    .await?;

                if set_result.is_some() {
                    let _: () = self.conn.clone()
                        .expire(&lock_key, lockout_secs as i64)
                        .await?;
                    lockout_remaining = Some(lockout_secs);
                } else {
                    let ttl: i64 = self.conn.clone().ttl(&lock_key).await?;
                    lockout_remaining = if ttl > 0 { Some(ttl as u64) } else { Some(lockout_secs) };
                }
            }

            Ok(FailedAttemptStatus {
                attempts: count,
                is_locked,
                lockout_remaining_secs: lockout_remaining,
            })
        }.await;

        res.map_err(Into::into)
    }

    async fn clear_failed_attempts(&self, identity: &str) -> Result<(), PortError> {
        let res: Result<_, LocalError> = async {
            let attempts_key = Self::login_attempts_key(identity);
            let lockout_key = Self::lockout_key(identity);

            let _: () = self.conn.clone().del(&[&attempts_key, &lockout_key]).await?;

            Ok(())
        }.await;

        res.map_err(Into::into)
    }

    async fn is_locked_out(&self, identity: &str) -> Result<Option<u64>, PortError> {
        let res: Result<_, LocalError> = async {
            let lock_key = Self::lockout_key(identity);
            let exists: bool = self.conn.clone().exists(&lock_key).await?;

            if exists {
                let ttl: i64 = self.conn.clone().ttl(&lock_key).await?;
                Ok(if ttl > 0 { Some(ttl as u64) } else { Some(0) })
            } else {
                Ok(None)
            }
        }.await;

        res.map_err(Into::into)
    }
}

#[derive(Error, Debug)]
pub enum LocalError {
    #[error(transparent)]
    Redis(#[from] ::redis::RedisError),
}

impl From<LocalError> for PortError {
    fn from(e: LocalError) -> Self {
        PortError::Internal(e.to_string())
    }
}
