use std::num::NonZeroUsize;
use std::path::PathBuf;
use std::sync::Arc;
use std::time::Duration;

use axum::extract::FromRef;
use chrono::{DateTime, Utc};
use leptos::config::LeptosOptions;
use tokio::sync::{Mutex, RwLock};

use crate::dnsmasq::command::DnsmasqCommand;
use crate::dnsmasq::systemd::Systemd;
use crate::server::auth::new_session;
use crate::server::rate_limit::LoginRateLimiter;
use crate::storage::paths::StoragePaths;

#[derive(Clone)]
pub struct AppState {
    pub inner: Arc<AppStateInner>,
}

pub struct AppStateInner {
    pub leptos_options: LeptosOptions,
    pub paths: StoragePaths,
    pub dnsmasq: DnsmasqCommand,
    pub systemd: Systemd,
    pub max_backups: Option<NonZeroUsize>,
    pub login_rate_limiter: LoginRateLimiter,
    pub auth: RwLock<AuthState>,
    pub auth_operations: Mutex<()>,
    pub config_operations: Mutex<()>,
}

#[derive(Debug, Default)]
pub struct AuthState {
    pub password_hash: Option<String>,
    pub sessions: Vec<AuthSession>,
}

#[derive(Debug)]
pub struct AuthSession {
    pub token: String,
    pub expires_at: DateTime<Utc>,
}

#[derive(Debug)]
pub struct CreatedSession {
    pub token: String,
    pub expires_at: DateTime<Utc>,
}

#[derive(Clone, Copy, Debug)]
pub struct RuntimeSettings {
    pub dnsmasq_test_timeout: Duration,
    pub systemctl_timeout: Duration,
    pub max_backups: Option<NonZeroUsize>,
}

impl Default for RuntimeSettings {
    fn default() -> Self {
        Self {
            dnsmasq_test_timeout: Duration::from_secs(10),
            systemctl_timeout: Duration::from_secs(30),
            max_backups: NonZeroUsize::new(50),
        }
    }
}

impl AppState {
    pub fn new(
        leptos_options: LeptosOptions,
        config_file: PathBuf,
        backup_dir: PathBuf,
        credentials_file: PathBuf,
        dnsmasq_bin: String,
        service_name: String,
        settings: RuntimeSettings,
    ) -> Self {
        Self {
            inner: Arc::new(AppStateInner {
                leptos_options,
                paths: StoragePaths::new(config_file, backup_dir, credentials_file),
                dnsmasq: DnsmasqCommand::new(dnsmasq_bin, settings.dnsmasq_test_timeout),
                systemd: Systemd::new(service_name, settings.systemctl_timeout),
                max_backups: settings.max_backups,
                login_rate_limiter: LoginRateLimiter::new(),
                auth: RwLock::new(AuthState::default()),
                auth_operations: Mutex::new(()),
                config_operations: Mutex::new(()),
            }),
        }
    }

    pub async fn is_password_configured(&self) -> bool {
        self.inner.auth.read().await.password_hash.is_some()
    }

    pub async fn create_session(&self) -> CreatedSession {
        let (token, expires_at) = new_session();
        let mut auth = self.inner.auth.write().await;
        auth.sessions.push(AuthSession {
            token: token.clone(),
            expires_at,
        });
        CreatedSession { token, expires_at }
    }

    pub async fn verify_session(&self, token: &str) -> bool {
        self.prune_expired_sessions().await;
        let auth = self.inner.auth.read().await;
        auth.sessions
            .iter()
            .any(|session| session.token == token && session.expires_at > Utc::now())
    }

    pub async fn prune_expired_sessions(&self) {
        let mut auth = self.inner.auth.write().await;
        let now = Utc::now();
        auth.sessions.retain(|session| session.expires_at > now);
    }
}

impl FromRef<AppState> for LeptosOptions {
    fn from_ref(state: &AppState) -> Self {
        state.inner.leptos_options.clone()
    }
}
