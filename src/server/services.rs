use chrono::{DateTime, Utc};
use tokio::fs;

use crate::api_types::{
    AuthResponse, AuthStatusResponse, BackupInfo, CommandReport, ConfigResponse, RawConfigResponse,
    RestoreBackupResponse, SaveResponse, ServiceStatus,
};
use crate::config::model::{ConfigLine, DnsRecords, ValidationLevel};
use crate::config::parser::parse_config;
use crate::config::records::{collect_records_from_config, replace_managed_records};
use crate::config::render::render_config;
use crate::config::validate::{has_errors, validate_records};
use crate::error::{AppError, AppResult};
use crate::server::auth;
use crate::server::state::AppState;
use crate::storage::{atomic_write, backup};

pub async fn auth_status(state: &AppState) -> AuthStatusResponse {
    AuthStatusResponse {
        configured: state.is_password_configured().await,
    }
}

pub async fn setup_password(state: &AppState, password: String) -> AppResult<AuthResponse> {
    auth::configure_password(state, password).await
}

pub async fn login(state: &AppState, password: String) -> AppResult<AuthResponse> {
    auth::login(state, password).await
}

pub async fn logout(state: &AppState, token: Option<&str>) {
    auth::logout(state, token).await;
}

pub async fn get_config(state: &AppState) -> AppResult<ConfigResponse> {
    let content = fs::read_to_string(&state.inner.paths.config_file).await?;
    let parsed = parse_config(&content)?;
    let records = collect_records_from_config(&parsed);
    let warnings = validate_records(&records)
        .into_iter()
        .filter(|issue| matches!(issue.level, ValidationLevel::Warning))
        .collect();
    let metadata = fs::metadata(&state.inner.paths.config_file).await.ok();
    let last_modified = metadata
        .and_then(|metadata| metadata.modified().ok())
        .map(DateTime::<Utc>::from);
    let unmanaged_line_count = parsed
        .lines
        .iter()
        .filter(|line| !matches!(line, ConfigLine::Managed(_)))
        .count();

    Ok(ConfigResponse {
        records,
        unmanaged_line_count,
        warnings,
        last_modified,
        service: state.inner.systemd.status().await,
    })
}

pub async fn save_records(
    state: &AppState,
    records: DnsRecords,
    apply: bool,
) -> AppResult<SaveResponse> {
    let issues = validate_records(&records);
    if has_errors(&issues) {
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

    let content = fs::read_to_string(&state.inner.paths.config_file).await?;
    let parsed = parse_config(&content)?;
    let next = replace_managed_records(&parsed, records)?;
    let rendered = render_config(&next);
    let result = persist_config(state, &rendered, apply).await?;

    Ok(SaveResponse {
        warnings: issues
            .into_iter()
            .filter(|issue| matches!(issue.level, ValidationLevel::Warning))
            .collect(),
        ..result
    })
}

pub async fn get_raw_config(state: &AppState) -> AppResult<RawConfigResponse> {
    let content = fs::read_to_string(&state.inner.paths.config_file).await?;
    let metadata = fs::metadata(&state.inner.paths.config_file).await.ok();
    let last_modified = metadata
        .and_then(|metadata| metadata.modified().ok())
        .map(DateTime::<Utc>::from);

    Ok(RawConfigResponse {
        content,
        last_modified,
    })
}

pub async fn save_raw_config(
    state: &AppState,
    content: String,
    apply: bool,
) -> AppResult<SaveResponse> {
    let parsed = parse_config(&content)?;
    let records = collect_records_from_config(&parsed);
    let issues = validate_records(&records);
    if has_errors(&issues) {
        return Err(AppError::InvalidConfig(String::from(
            "raw config contains invalid managed records",
        )));
    }

    let result = persist_config(state, &content, apply).await?;
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
    let temp_path = atomic_write::write_temp_near(&state.inner.paths.config_file, &content).await?;
    let report = state.inner.dnsmasq.test_config(&temp_path).await;
    let _ = fs::remove_file(&temp_path).await;
    report
}

pub async fn reload_dnsmasq(state: &AppState) -> AppResult<CommandReport> {
    state.inner.systemd.reload_or_restart().await
}

pub async fn status(state: &AppState) -> ServiceStatus {
    state.inner.systemd.status().await
}

pub async fn list_backups(state: &AppState) -> AppResult<Vec<BackupInfo>> {
    backup::list_backups(&state.inner.paths.backup_dir).await
}

pub async fn restore_backup(state: &AppState, id: String) -> AppResult<RestoreBackupResponse> {
    let path = backup::backup_path(&state.inner.paths.backup_dir, &id);
    let content = fs::read_to_string(&path).await?;
    let temp_path = atomic_write::write_temp_near(&state.inner.paths.config_file, &content).await?;
    let test = state.inner.dnsmasq.test_config(&temp_path).await;
    let _ = fs::remove_file(&temp_path).await;
    let test = test?;

    let created_backup = backup::create_backup(
        &state.inner.paths.config_file,
        &state.inner.paths.backup_dir,
    )
    .await?;
    atomic_write::replace(&state.inner.paths.config_file, &content).await?;
    let reload = Some(state.inner.systemd.reload_or_restart().await?);

    Ok(RestoreBackupResponse {
        created_backup,
        test,
        reload,
    })
}

async fn persist_config(state: &AppState, content: &str, apply: bool) -> AppResult<SaveResponse> {
    let temp_path = atomic_write::write_temp_near(&state.inner.paths.config_file, content).await?;
    let test = state.inner.dnsmasq.test_config(&temp_path).await;
    let _ = fs::remove_file(&temp_path).await;
    let test = test?;

    let backup = backup::create_backup(
        &state.inner.paths.config_file,
        &state.inner.paths.backup_dir,
    )
    .await?;
    atomic_write::replace(&state.inner.paths.config_file, content).await?;

    let reload = if apply {
        Some(state.inner.systemd.reload_or_restart().await?)
    } else {
        None
    };

    Ok(SaveResponse {
        applied: apply,
        backup: Some(backup),
        test,
        reload,
        warnings: Vec::new(),
    })
}
