use std::collections::HashMap;
use std::net::IpAddr;
use std::time::{Duration, Instant};

use tokio::sync::Mutex;

use crate::error::{AppError, AppResult};

const LOGIN_ATTEMPTS_PER_WINDOW: usize = 10;
const LOGIN_WINDOW: Duration = Duration::from_secs(60);

#[derive(Default)]
pub struct LoginRateLimiter {
    attempts: Mutex<HashMap<Option<IpAddr>, AttemptWindow>>,
}

#[derive(Clone, Copy)]
struct AttemptWindow {
    started_at: Instant,
    attempts: usize,
}

impl LoginRateLimiter {
    pub fn new() -> Self {
        Self::default()
    }

    pub async fn check(&self, peer_ip: Option<IpAddr>) -> AppResult<()> {
        self.check_at(peer_ip, Instant::now()).await
    }

    pub async fn reset(&self, peer_ip: Option<IpAddr>) {
        self.attempts.lock().await.remove(&peer_ip);
    }

    async fn check_at(&self, peer_ip: Option<IpAddr>, now: Instant) -> AppResult<()> {
        let mut attempts = self.attempts.lock().await;
        attempts.retain(|_, entry| now.duration_since(entry.started_at) < LOGIN_WINDOW);

        let entry = attempts.entry(peer_ip).or_insert(AttemptWindow {
            started_at: now,
            attempts: 0,
        });
        if entry.attempts >= LOGIN_ATTEMPTS_PER_WINDOW {
            let remaining = LOGIN_WINDOW.saturating_sub(now.duration_since(entry.started_at));
            return Err(AppError::RateLimited {
                retry_after_seconds: remaining.as_secs() + u64::from(remaining.subsec_nanos() > 0),
            });
        }

        entry.attempts += 1;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use std::net::{IpAddr, Ipv4Addr};
    use std::time::{Duration, Instant};

    use super::{LOGIN_WINDOW, LoginRateLimiter};
    use crate::error::AppError;

    #[tokio::test]
    async fn limits_each_ip_and_resets_after_window() {
        let limiter = LoginRateLimiter::new();
        let peer = Some(IpAddr::V4(Ipv4Addr::LOCALHOST));
        let started_at = Instant::now();

        for _ in 0..10 {
            limiter
                .check_at(peer, started_at)
                .await
                .expect("attempt should be allowed");
        }
        assert!(matches!(
            limiter.check_at(peer, started_at).await,
            Err(AppError::RateLimited { .. })
        ));
        limiter
            .check_at(peer, started_at + LOGIN_WINDOW + Duration::from_millis(1))
            .await
            .expect("new window should be allowed");
    }

    #[tokio::test]
    async fn successful_login_reset_clears_attempts() {
        let limiter = LoginRateLimiter::new();
        let started_at = Instant::now();
        for _ in 0..10 {
            limiter
                .check_at(None, started_at)
                .await
                .expect("attempt should be allowed");
        }

        limiter.reset(None).await;
        limiter
            .check_at(None, started_at)
            .await
            .expect("attempt after reset should be allowed");
    }
}
