use chrono::{DateTime, Utc};
use tokio::fs;
use tracing::{error, info, warn};

use crate::api_types::{
    AuthStatusResponse, BackupInfo, CommandReport, ConfigResponse, RawConfigResponse,
    RestoreBackupResponse, SaveResponse, ServiceStatus,
};
use crate::config::model::{ConfigLine, DnsRecords, ValidationLevel};
use crate::config::parser::parse_config;
use crate::config::records::{collect_records_from_config, replace_managed_records};
use crate::config::render::render_config;
use crate::config::validate::{has_errors, validate_records};
use crate::error::{AppError, AppResult};
use crate::i18n::Locale;
use crate::server::auth;
use crate::server::config_apply::{self, ConfigApplyRequest, ConfigApplyResult};
use crate::server::state::{AppState, CreatedSession};
use crate::storage::backup;

pub async fn auth_status(
    state: &AppState,
    token: Option<&str>,
    locale: Locale,
) -> AuthStatusResponse {
    let authenticated = match token {
        Some(token) => state.verify_session(token).await,
        None => false,
    };

    AuthStatusResponse {
        configured: state.is_password_configured().await,
        authenticated,
        locale,
    }
}

pub async fn setup_password(
    state: &AppState,
    password: String,
    password_confirmation: String,
) -> AppResult<CreatedSession> {
    auth::configure_password(state, password, password_confirmation).await
}

pub async fn login(state: &AppState, password: String) -> AppResult<CreatedSession> {
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
    let content = fs::read_to_string(&state.inner.paths.config_file).await?;
    let parsed = parse_config(&content)?;
    let next = replace_managed_records(&parsed, records)?;
    let rendered = render_config(&next);
    let result = persist_config(state, &rendered, apply, "structured").await?;

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
        warn!("raw config save rejected by managed record validation");
        return Err(AppError::InvalidConfig(String::from(
            "raw config contains invalid managed records",
        )));
    }

    let _operation = state.inner.config_operations.lock().await;
    let result = persist_config(state, &content, apply, "raw").await?;
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
    let path = backup::checked_backup_path(&state.inner.paths.backup_dir, &id)?;

    let _operation = state.inner.config_operations.lock().await;
    let content = fs::read_to_string(&path).await?;
    let result = apply_config_transaction(state, &content, true, "restore").await?;
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
) -> AppResult<SaveResponse> {
    let result = apply_config_transaction(state, content, apply, source).await?;

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
) -> AppResult<ConfigApplyResult> {
    match config_apply::apply_config(
        ConfigApplyRequest {
            config_file: &state.inner.paths.config_file,
            backup_dir: &state.inner.paths.backup_dir,
            content,
            apply,
            source,
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
