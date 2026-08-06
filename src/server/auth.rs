use std::time::Duration;

use bcrypt::{DEFAULT_COST, HashParts, hash, verify};
use chrono::{DateTime, Utc};
use tokio::task;
use tokio::time;
use uuid::Uuid;

use crate::error::AppError;
use crate::server::state::{AppState, CreatedSession};
use crate::storage::credentials;

pub const SESSION_TTL: Duration = Duration::from_secs(24 * 60 * 60);
pub const SESSION_COOKIE: &str = "dnsmasqweb_session";

#[derive(Clone, Debug)]
pub struct RequestAuth {
    pub token: Option<String>,
    pub configured: bool,
    pub authenticated: bool,
}

pub async fn resolve_request_auth(state: &AppState, token: Option<String>) -> RequestAuth {
    let configured = state.is_password_configured().await;
    let authenticated = match token.as_deref() {
        Some(token) => state.verify_session(token).await,
        None => false,
    };
    RequestAuth {
        token,
        configured,
        authenticated,
    }
}

pub async fn load_persisted_password(state: &AppState) -> Result<(), AppError> {
    let Some(password_hash) = credentials::load(&state.inner.paths.credentials_file).await? else {
        return Ok(());
    };
    password_hash
        .parse::<HashParts>()
        .map_err(|error| AppError::Auth(format!("invalid persisted password hash: {error}")))?;

    let mut auth = state.inner.auth.write().await;
    auth.password_hash = Some(password_hash);
    auth.sessions.clear();
    Ok(())
}

pub async fn configure_password(
    state: &AppState,
    password: String,
    password_confirmation: String,
) -> Result<CreatedSession, AppError> {
    if password != password_confirmation {
        return Err(AppError::InvalidConfig(String::from(
            "passwords do not match",
        )));
    }

    let password = normalize_password(password)?;
    let _operation = state.inner.auth_operations.lock().await;

    {
        let auth = state.inner.auth.read().await;
        if auth.password_hash.is_some() {
            return Err(AppError::InvalidConfig(String::from(
                "password is already configured",
            )));
        }
    }

    let password_hash = hash_password(password).await?;
    credentials::store(&state.inner.paths.credentials_file, &password_hash).await?;
    let mut auth = state.inner.auth.write().await;
    auth.password_hash = Some(password_hash);
    auth.sessions.clear();
    drop(auth);

    Ok(state.create_session().await)
}

pub async fn change_password(
    state: &AppState,
    current_password: String,
    new_password: String,
    new_password_confirmation: String,
) -> Result<CreatedSession, AppError> {
    if new_password != new_password_confirmation {
        return Err(AppError::InvalidConfig(String::from(
            "passwords do not match",
        )));
    }

    let current_password = normalize_password(current_password)?;
    let new_password = normalize_password(new_password)?;
    let _operation = state.inner.auth_operations.lock().await;
    let current_hash = {
        let auth = state.inner.auth.read().await;
        auth.password_hash
            .clone()
            .ok_or_else(|| AppError::InvalidConfig(String::from("password is not configured")))?
    };

    if !verify_password(current_password, current_hash).await? {
        return Err(AppError::InvalidConfig(String::from(
            "current password is incorrect",
        )));
    }

    let password_hash = hash_password(new_password).await?;
    credentials::store(&state.inner.paths.credentials_file, &password_hash).await?;
    let mut auth = state.inner.auth.write().await;
    auth.password_hash = Some(password_hash);
    auth.sessions.clear();
    drop(auth);

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

    let valid = verify_password(password, password_hash).await?;

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

async fn hash_password(password: String) -> Result<String, AppError> {
    task::spawn_blocking(move || hash(password, DEFAULT_COST))
        .await
        .map_err(|error| AppError::Auth(format!("failed to hash password: {error}")))?
        .map_err(|error| AppError::Auth(format!("failed to hash password: {error}")))
}

async fn verify_password(password: String, password_hash: String) -> Result<bool, AppError> {
    task::spawn_blocking(move || verify(password, &password_hash))
        .await
        .map_err(|error| AppError::Auth(format!("failed to verify password: {error}")))?
        .map_err(|error| AppError::Auth(format!("failed to verify password: {error}")))
}

pub fn new_session() -> (String, DateTime<Utc>) {
    let token = Uuid::new_v4().to_string();
    let expires_at = Utc::now() + chrono::Duration::seconds(SESSION_TTL.as_secs() as i64);
    (token, expires_at)
}

#[cfg(test)]
mod tests {
    use std::ops::Deref;
    use std::os::unix::fs::PermissionsExt;
    use std::path::PathBuf;
    use std::time::{SystemTime, UNIX_EPOCH};

    use chrono::Utc;

    use super::{change_password, configure_password, load_persisted_password, login, logout};
    use crate::server::state::{AppState, AuthSession, RuntimeSettings};

    struct TestState {
        app: AppState,
        root: PathBuf,
    }

    impl TestState {
        fn new(name: &str) -> Self {
            let nanos = SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .map(|duration| duration.as_nanos())
                .unwrap_or_default();
            let root = std::env::temp_dir().join(format!(
                "dnsmasqweb-auth-{name}-{}-{nanos}",
                std::process::id()
            ));
            let app = test_app_state(&root);
            Self { app, root }
        }
    }

    impl Deref for TestState {
        type Target = AppState;

        fn deref(&self) -> &Self::Target {
            &self.app
        }
    }

    impl Drop for TestState {
        fn drop(&mut self) {
            let _ = std::fs::remove_dir_all(&self.root);
        }
    }

    fn test_app_state(root: &std::path::Path) -> AppState {
        AppState::new(
            leptos::config::LeptosOptions::builder()
                .output_name("dnsmasqweb")
                .build(),
            root.join("dnsmasq.conf"),
            root.join("backups"),
            root.join("state/password.hash"),
            String::from("dnsmasq"),
            String::from("dnsmasq"),
            RuntimeSettings::default(),
        )
    }

    #[tokio::test]
    async fn setup_hashes_password_and_creates_session() {
        let state = TestState::new("setup");

        let response = configure_password(&state, String::from("secret"), String::from("secret"))
            .await
            .expect("password setup should succeed");

        assert!(state.is_password_configured().await);
        assert!(state.verify_session(&response.token).await);
        let metadata =
            std::fs::metadata(&state.inner.paths.credentials_file).expect("credentials metadata");
        assert_eq!(metadata.permissions().mode() & 0o777, 0o600);
    }

    #[tokio::test]
    async fn login_rejects_wrong_password_and_accepts_correct_password() {
        let state = TestState::new("login");
        configure_password(&state, String::from("secret"), String::from("secret"))
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
        let state = TestState::new("logout");
        let response = configure_password(&state, String::from("secret"), String::from("secret"))
            .await
            .expect("password setup should succeed");

        logout(&state, Some(&response.token)).await;

        assert!(!state.verify_session(&response.token).await);
    }

    #[tokio::test]
    async fn expired_sessions_are_rejected() {
        let state = TestState::new("expired");
        {
            let mut auth = state.inner.auth.write().await;
            auth.sessions.push(AuthSession {
                token: String::from("expired"),
                expires_at: Utc::now() - chrono::Duration::seconds(1),
            });
        }

        assert!(!state.verify_session("expired").await);
    }

    #[tokio::test]
    async fn setup_rejects_mismatched_passwords() {
        let state = TestState::new("mismatch");

        let result =
            configure_password(&state, String::from("secret"), String::from("different")).await;

        assert!(matches!(
            result,
            Err(crate::error::AppError::InvalidConfig(_))
        ));
        assert!(!state.is_password_configured().await);
    }

    #[tokio::test]
    async fn persisted_password_loads_after_restart() {
        let state = TestState::new("persisted");
        configure_password(&state, String::from("secret"), String::from("secret"))
            .await
            .expect("password setup should succeed");

        let restarted = test_app_state(&state.root);
        load_persisted_password(&restarted)
            .await
            .expect("persisted password should load");

        assert!(restarted.is_password_configured().await);
        assert!(login(&restarted, String::from("secret")).await.is_ok());
    }

    #[tokio::test]
    async fn password_change_revokes_sessions_and_persists_new_hash() {
        let state = TestState::new("change");
        let old_session = configure_password(
            &state,
            String::from("old-secret"),
            String::from("old-secret"),
        )
        .await
        .expect("password setup should succeed");

        let new_session = change_password(
            &state,
            String::from("old-secret"),
            String::from("new-secret"),
            String::from("new-secret"),
        )
        .await
        .expect("password change should succeed");

        assert!(!state.verify_session(&old_session.token).await);
        assert!(state.verify_session(&new_session.token).await);
        assert!(login(&state, String::from("old-secret")).await.is_err());
        assert!(login(&state, String::from("new-secret")).await.is_ok());

        let restarted = test_app_state(&state.root);
        load_persisted_password(&restarted)
            .await
            .expect("changed password should load");
        assert!(login(&restarted, String::from("new-secret")).await.is_ok());
    }
}
