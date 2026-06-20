use std::path::PathBuf;
use std::sync::Arc;

use chrono::{DateTime, Utc};
use tokio::sync::RwLock;

use crate::dnsmasq::command::DnsmasqCommand;
use crate::dnsmasq::systemd::Systemd;
use crate::server::auth::new_session;
use crate::storage::paths::StoragePaths;

#[derive(Clone)]
pub struct AppState {
    pub inner: Arc<AppStateInner>,
}

pub struct AppStateInner {
    pub paths: StoragePaths,
    pub dnsmasq: DnsmasqCommand,
    pub systemd: Systemd,
    pub auth: RwLock<AuthState>,
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

impl AppState {
    pub fn new(
        config_file: PathBuf,
        backup_dir: PathBuf,
        dnsmasq_bin: String,
        service_name: String,
    ) -> Self {
        Self {
            inner: Arc::new(AppStateInner {
                paths: StoragePaths::new(config_file, backup_dir),
                dnsmasq: DnsmasqCommand::new(dnsmasq_bin),
                systemd: Systemd::new(service_name),
                auth: RwLock::new(AuthState::default()),
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
