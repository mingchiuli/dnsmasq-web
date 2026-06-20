use std::time::Duration;

use bcrypt::{DEFAULT_COST, hash, verify};
use chrono::{DateTime, Utc};
use tokio::task;
use tokio::time;
use uuid::Uuid;

use crate::error::AppError;
use crate::server::state::{AppState, CreatedSession};

pub const SESSION_TTL: Duration = Duration::from_secs(24 * 60 * 60);

pub async fn configure_password(
    state: &AppState,
    password: String,
) -> Result<CreatedSession, AppError> {
    let password = normalize_password(password)?;
    let password_hash = task::spawn_blocking(move || hash(password, DEFAULT_COST))
        .await
        .map_err(|error| AppError::Auth(format!("failed to hash password: {error}")))?
        .map_err(|error| AppError::Auth(format!("failed to hash password: {error}")))?;

    {
        let mut auth = state.inner.auth.write().await;
        if auth.password_hash.is_some() {
            return Err(AppError::InvalidConfig(String::from(
                "password is already configured",
            )));
        }
        auth.password_hash = Some(password_hash);
    }

    Ok(state.create_session().await)
}

pub async fn login(state: &AppState, password: String) -> Result<CreatedSession, AppError> {
    let password = normalize_password(password)?;
    let password_hash = {
        let auth = state.inner.auth.read().await;
        auth.password_hash
            .clone()
            .ok_or_else(|| AppError::InvalidConfig(String::from("password is not configured")))?
    };

    let valid = task::spawn_blocking(move || verify(password, &password_hash))
        .await
        .map_err(|error| AppError::Auth(format!("failed to verify password: {error}")))?
        .map_err(|error| AppError::Auth(format!("failed to verify password: {error}")))?;

    if valid {
        Ok(state.create_session().await)
    } else {
        Err(AppError::Unauthorized)
    }
}

pub async fn logout(state: &AppState, token: Option<&str>) {
    if let Some(token) = token {
        let mut auth = state.inner.auth.write().await;
        auth.sessions.retain(|session| session.token != token);
    }
}

pub async fn cleanup_expired_sessions(state: AppState) {
    let mut interval = time::interval(Duration::from_secs(60));
    loop {
        interval.tick().await;
        state.prune_expired_sessions().await;
    }
}

fn normalize_password(password: String) -> Result<String, AppError> {
    let password = password.trim().to_string();
    if password.is_empty() {
        Err(AppError::InvalidConfig(String::from(
            "password cannot be empty",
        )))
    } else {
        Ok(password)
    }
}

pub fn new_session() -> (String, DateTime<Utc>) {
    let token = Uuid::new_v4().to_string();
    let expires_at = Utc::now()
        + chrono::Duration::from_std(SESSION_TTL).expect("session ttl must fit in chrono duration");
    (token, expires_at)
}

#[cfg(test)]
mod tests {
    use std::path::PathBuf;

    use chrono::Utc;

    use super::{configure_password, login, logout};
    use crate::server::state::{AppState, AuthSession};

    fn state() -> AppState {
        AppState::new(
            PathBuf::from("/tmp/dnsmasq.conf"),
            PathBuf::from("/tmp/dnsmasqweb-backups"),
            String::from("dnsmasq"),
            String::from("dnsmasq"),
        )
    }

    #[tokio::test]
    async fn setup_hashes_password_and_creates_session() {
        let state = state();

        let response = configure_password(&state, String::from("secret"))
            .await
            .expect("password setup should succeed");

        assert!(state.is_password_configured().await);
        assert!(state.verify_session(&response.token).await);
    }

    #[tokio::test]
    async fn login_rejects_wrong_password_and_accepts_correct_password() {
        let state = state();
        configure_password(&state, String::from("secret"))
            .await
            .expect("password setup should succeed");

        assert!(login(&state, String::from("wrong")).await.is_err());
        let response = login(&state, String::from("secret"))
            .await
            .expect("login should succeed");
        assert!(state.verify_session(&response.token).await);
    }

    #[tokio::test]
    async fn logout_removes_session() {
        let state = state();
        let response = configure_password(&state, String::from("secret"))
            .await
            .expect("password setup should succeed");

        logout(&state, Some(&response.token)).await;

        assert!(!state.verify_session(&response.token).await);
    }

    #[tokio::test]
    async fn expired_sessions_are_rejected() {
        let state = state();
        {
            let mut auth = state.inner.auth.write().await;
            auth.sessions.push(AuthSession {
                token: String::from("expired"),
                expires_at: Utc::now() - chrono::Duration::seconds(1),
            });
        }

        assert!(!state.verify_session("expired").await);
    }
}
