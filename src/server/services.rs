use std::net::IpAddr;

use chrono::{DateTime, Utc};
use tokio::fs;
use tracing::{error, info, warn};

use crate::api_types::{
    AuthStatusResponse, BackupInfo, BootstrapResponse, CommandReport, ConfigResponse,
    ConfigRevision, DashboardBootstrap, RawConfigResponse, RestoreBackupResponse, SaveResponse,
    ServiceStatus,
};
use crate::config::model::{DnsRecords, ParsedConfig, ValidationLevel};
use crate::config::parser::parse_config;
use crate::config::records::{
    collect_records_from_config, managed_record_count, replace_managed_records,
};
use crate::config::render::render_config;
use crate::config::validate::{has_errors, validate_records};
use crate::error::{AppError, AppResult};
use crate::i18n::Locale;
use crate::server::auth;
use crate::server::auth::RequestAuth;
use crate::server::config_apply::{self, ConfigApplyRequest, ConfigApplyResult};
use crate::server::config_revision;
use crate::server::state::{AppState, CreatedSession};
use crate::storage::backup;

pub async fn auth_status(request_auth: &RequestAuth, locale: Locale) -> AuthStatusResponse {
    AuthStatusResponse {
        configured: request_auth.configured,
        authenticated: request_auth.authenticated,
        locale,
    }
}

pub async fn bootstrap(
    state: &AppState,
    request_auth: &RequestAuth,
    locale: Locale,
) -> BootstrapResponse {
    if !request_auth.configured {
        return BootstrapResponse::Setup { locale };
    }
    if !request_auth.authenticated {
        return BootstrapResponse::Login { locale };
    }

    let (config_pair, backups) = tokio::join!(get_dashboard_config(state), list_backups(state));
    let (config, raw) = match config_pair {
        Ok((config, raw)) => (Ok(config), Ok(raw)),
        Err(error) => {
            let message = error.to_string();
            (Err(message.clone()), Err(message))
        }
    };
    BootstrapResponse::Authenticated {
        locale,
        dashboard: Box::new(DashboardBootstrap {
            config,
            raw,
            backups: backups.map_err(|error| error.to_string()),
        }),
    }
}

pub async fn setup_password(
    state: &AppState,
    password: String,
    password_confirmation: String,
) -> AppResult<CreatedSession> {
    auth::configure_password(state, password, password_confirmation).await
}

pub async fn login(
    state: &AppState,
    password: String,
    peer_ip: Option<IpAddr>,
) -> AppResult<CreatedSession> {
    state.inner.login_rate_limiter.check(peer_ip).await?;
    let result = auth::login(state, password).await;
    if result.is_ok() {
        state.inner.login_rate_limiter.reset(peer_ip).await;
    }
    result
}

pub async fn change_password(
    state: &AppState,
    current_password: String,
    new_password: String,
    new_password_confirmation: String,
) -> AppResult<CreatedSession> {
    auth::change_password(
        state,
        current_password,
        new_password,
        new_password_confirmation,
    )
    .await
}

pub async fn logout(state: &AppState, token: Option<&str>) {
    auth::logout(state, token).await;
}

pub async fn get_config(state: &AppState) -> AppResult<ConfigResponse> {
    let snapshot = read_config_snapshot(state).await?;
    Ok(config_response(
        &snapshot,
        state.inner.systemd.status().await,
    ))
}

async fn get_dashboard_config(state: &AppState) -> AppResult<(ConfigResponse, RawConfigResponse)> {
    let snapshot = read_config_snapshot(state).await?;
    let config = config_response(&snapshot, state.inner.systemd.status().await);
    let raw = raw_config_response(&snapshot);
    Ok((config, raw))
}

fn config_response(snapshot: &ConfigSnapshot, service: ServiceStatus) -> ConfigResponse {
    let records = collect_records_from_config(&snapshot.parsed);
    let warnings = validate_records(&records)
        .into_iter()
        .filter(|issue| matches!(issue.level, ValidationLevel::Warning))
        .collect();
    let unmanaged_line_count = snapshot.parsed.lines.len() - managed_record_count(&snapshot.parsed);

    ConfigResponse {
        records,
        revision: snapshot.revision.clone(),
        unmanaged_line_count,
        warnings,
        last_modified: snapshot.last_modified,
        service,
    }
}

pub async fn save_records(
    state: &AppState,
    records: DnsRecords,
    apply: bool,
    expected_revision: ConfigRevision,
) -> AppResult<SaveResponse> {
    let issues = validate_records(&records);
    if has_errors(&issues) {
        warn!("structured config save rejected by validation");
        return Err(AppError::InvalidConfig(format!(
            "validation failed: {}",
            issues
                .iter()
                .filter(|issue| matches!(issue.level, ValidationLevel::Error))
                .map(|issue| issue.message.as_str())
                .collect::<Vec<_>>()
                .join("; ")
        )));
    }

    let _operation = state.inner.config_operations.lock().await;
    let (current_content, current_revision) = read_config_content(state).await?;
    ensure_revision(&current_revision, &expected_revision)?;
    let current_config = parse_config(&current_content)?;
    let next = replace_managed_records(&current_config, records)?;
    let rendered = render_config(&next);
    let result = persist_config(state, &rendered, apply, "structured", &expected_revision).await?;

    Ok(SaveResponse {
        warnings: issues
            .into_iter()
            .filter(|issue| matches!(issue.level, ValidationLevel::Warning))
            .collect(),
        ..result
    })
}

pub async fn get_raw_config(state: &AppState) -> AppResult<RawConfigResponse> {
    let snapshot = read_config_snapshot(state).await?;
    Ok(raw_config_response(&snapshot))
}

pub async fn save_raw_config(
    state: &AppState,
    content: String,
    apply: bool,
    expected_revision: ConfigRevision,
) -> AppResult<SaveResponse> {
    let parsed = parse_config(&content)?;
    let records = collect_records_from_config(&parsed);
    let issues = validate_records(&records);
    if has_errors(&issues) {
        warn!("raw config save rejected by managed record validation");
        return Err(AppError::InvalidConfig(String::from(
            "raw config contains invalid managed records",
        )));
    }

    let _operation = state.inner.config_operations.lock().await;
    let (_, current_revision) = read_config_content(state).await?;
    ensure_revision(&current_revision, &expected_revision)?;
    let result = persist_config(state, &content, apply, "raw", &expected_revision).await?;
    Ok(SaveResponse {
        warnings: issues
            .into_iter()
            .filter(|issue| matches!(issue.level, ValidationLevel::Warning))
            .collect(),
        ..result
    })
}

pub async fn test_config(state: &AppState, content: Option<String>) -> AppResult<CommandReport> {
    let content = match content {
        Some(content) => content,
        None => fs::read_to_string(&state.inner.paths.config_file).await?,
    };
    let report = config_apply::test_content(
        &state.inner.paths.config_file,
        &content,
        &state.inner.dnsmasq,
    )
    .await;
    if let Err(error) = &report {
        warn!(%error, "dnsmasq config test failed");
    }
    report
}

pub async fn reload_dnsmasq(state: &AppState) -> AppResult<CommandReport> {
    let _operation = state.inner.config_operations.lock().await;
    let report = state.inner.systemd.restart().await;
    match &report {
        Ok(_) => info!("dnsmasq service restarted"),
        Err(error) => error!(%error, "dnsmasq service restart failed"),
    }
    report
}

pub async fn status(state: &AppState) -> ServiceStatus {
    state.inner.systemd.status().await
}

pub async fn list_backups(state: &AppState) -> AppResult<Vec<BackupInfo>> {
    backup::list_backups(&state.inner.paths.backup_dir).await
}

pub async fn delete_backup(state: &AppState, id: String) -> AppResult<()> {
    let _operation = state.inner.config_operations.lock().await;
    let result = backup::delete_backup(&state.inner.paths.backup_dir, &id).await;
    match &result {
        Ok(()) => info!(backup_id = %id, "backup deleted"),
        Err(error) => warn!(backup_id = %id, %error, "backup delete failed"),
    }
    result
}

pub async fn restore_backup(state: &AppState, id: String) -> AppResult<RestoreBackupResponse> {
    info!(backup_id = %id, "backup restore requested");
    let _operation = state.inner.config_operations.lock().await;
    let path = backup::checked_backup_file(&state.inner.paths.backup_dir, &id).await?;
    let content = fs::read_to_string(&path).await?;
    let (_, current_revision) = read_config_content(state).await?;
    let result =
        apply_config_transaction(state, &content, true, "restore", &current_revision).await?;
    info!(
        backup_id = %id,
        rollback_backup = %result.backup.path,
        "backup restored and dnsmasq restarted"
    );

    Ok(RestoreBackupResponse {
        created_backup: result.backup,
        test: result.test,
        reload: result.reload,
    })
}

async fn persist_config(
    state: &AppState,
    content: &str,
    apply: bool,
    source: &'static str,
    expected_revision: &ConfigRevision,
) -> AppResult<SaveResponse> {
    let result = apply_config_transaction(state, content, apply, source, expected_revision).await?;

    Ok(SaveResponse {
        applied: apply,
        backup: Some(result.backup),
        test: result.test,
        reload: result.reload,
        warnings: Vec::new(),
    })
}

async fn apply_config_transaction(
    state: &AppState,
    content: &str,
    apply: bool,
    source: &'static str,
    expected_revision: &ConfigRevision,
) -> AppResult<ConfigApplyResult> {
    match config_apply::apply_config(
        ConfigApplyRequest {
            config_file: &state.inner.paths.config_file,
            backup_dir: &state.inner.paths.backup_dir,
            content,
            apply,
            source,
            expected_revision,
            max_backups: state.inner.max_backups,
        },
        &state.inner.dnsmasq,
        &state.inner.systemd,
    )
    .await
    {
        Ok(result) => Ok(result),
        Err(error) => {
            warn!(source, %error, "config replacement failed");
            Err(error)
        }
    }
}

struct ConfigSnapshot {
    content: String,
    parsed: ParsedConfig,
    revision: ConfigRevision,
    last_modified: Option<DateTime<Utc>>,
}

async fn read_config_snapshot(state: &AppState) -> AppResult<ConfigSnapshot> {
    let (content, revision) = read_config_content(state).await?;
    let parsed = parse_config(&content)?;
    let last_modified = fs::metadata(&state.inner.paths.config_file)
        .await
        .ok()
        .and_then(|metadata| metadata.modified().ok())
        .map(DateTime::<Utc>::from);

    Ok(ConfigSnapshot {
        content,
        parsed,
        revision,
        last_modified,
    })
}

async fn read_config_content(state: &AppState) -> AppResult<(String, ConfigRevision)> {
    let content = fs::read_to_string(&state.inner.paths.config_file).await?;
    let revision = config_revision::calculate(&content);
    Ok((content, revision))
}

fn raw_config_response(snapshot: &ConfigSnapshot) -> RawConfigResponse {
    RawConfigResponse {
        content: snapshot.content.clone(),
        revision: snapshot.revision.clone(),
        last_modified: snapshot.last_modified,
    }
}

fn ensure_revision(actual: &ConfigRevision, expected: &ConfigRevision) -> AppResult<()> {
    if actual == expected {
        Ok(())
    } else {
        Err(AppError::ConfigConflict)
    }
}
