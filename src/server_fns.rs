use leptos::prelude::*;

use crate::api_types::{
    AuthResponse, AuthStatusResponse, BackupInfo, CommandReport, ConfigResponse, RawConfigResponse,
    RestoreBackupResponse, SaveResponse, ServiceStatus,
};
use crate::config::model::DnsRecords;

#[server(AuthStatus, "/api")]
pub async fn auth_status() -> Result<AuthStatusResponse, ServerFnError> {
    let state = app_state()?;
    Ok(crate::server::services::auth_status(&state).await)
}

#[server(SetupPassword, "/api")]
pub async fn setup_password(password: String) -> Result<AuthResponse, ServerFnError> {
    let state = app_state()?;
    crate::server::services::setup_password(&state, password)
        .await
        .map_err(server_error)
}

#[server(Login, "/api")]
pub async fn login(password: String) -> Result<AuthResponse, ServerFnError> {
    let state = app_state()?;
    crate::server::services::login(&state, password)
        .await
        .map_err(server_error)
}

#[server(Logout, "/api")]
pub async fn logout(token: Option<String>) -> Result<(), ServerFnError> {
    let state = app_state()?;
    crate::server::services::logout(&state, token.as_deref()).await;
    Ok(())
}

#[server(GetConfig, "/api")]
pub async fn get_config(token: Option<String>) -> Result<ConfigResponse, ServerFnError> {
    let state = authed_state(token).await?;
    crate::server::services::get_config(&state)
        .await
        .map_err(server_error)
}

#[server(SaveRecords, "/api")]
pub async fn save_records(
    token: Option<String>,
    records: DnsRecords,
    apply: bool,
) -> Result<SaveResponse, ServerFnError> {
    let state = authed_state(token).await?;
    crate::server::services::save_records(&state, records, apply)
        .await
        .map_err(server_error)
}

#[server(GetRawConfig, "/api")]
pub async fn get_raw_config(token: Option<String>) -> Result<RawConfigResponse, ServerFnError> {
    let state = authed_state(token).await?;
    crate::server::services::get_raw_config(&state)
        .await
        .map_err(server_error)
}

#[server(SaveRawConfig, "/api")]
pub async fn save_raw_config(
    token: Option<String>,
    content: String,
    apply: bool,
) -> Result<SaveResponse, ServerFnError> {
    let state = authed_state(token).await?;
    crate::server::services::save_raw_config(&state, content, apply)
        .await
        .map_err(server_error)
}

#[server(TestConfig, "/api")]
pub async fn test_config(
    token: Option<String>,
    content: Option<String>,
) -> Result<CommandReport, ServerFnError> {
    let state = authed_state(token).await?;
    crate::server::services::test_config(&state, content)
        .await
        .map_err(server_error)
}

#[server(ReloadDnsmasq, "/api")]
pub async fn reload_dnsmasq(token: Option<String>) -> Result<CommandReport, ServerFnError> {
    let state = authed_state(token).await?;
    crate::server::services::reload_dnsmasq(&state)
        .await
        .map_err(server_error)
}

#[server(Status, "/api")]
pub async fn status(token: Option<String>) -> Result<ServiceStatus, ServerFnError> {
    let state = authed_state(token).await?;
    Ok(crate::server::services::status(&state).await)
}

#[server(ListBackups, "/api")]
pub async fn list_backups(token: Option<String>) -> Result<Vec<BackupInfo>, ServerFnError> {
    let state = authed_state(token).await?;
    crate::server::services::list_backups(&state)
        .await
        .map_err(server_error)
}

#[server(RestoreBackup, "/api")]
pub async fn restore_backup(
    token: Option<String>,
    id: String,
) -> Result<RestoreBackupResponse, ServerFnError> {
    let state = authed_state(token).await?;
    crate::server::services::restore_backup(&state, id)
        .await
        .map_err(server_error)
}

#[cfg(feature = "ssr")]
fn app_state() -> Result<crate::server::state::AppState, ServerFnError> {
    use_context::<crate::server::state::AppState>()
        .ok_or_else(|| ServerFnError::ServerError(String::from("missing app state")))
}

#[cfg(feature = "ssr")]
async fn authed_state(
    token: Option<String>,
) -> Result<crate::server::state::AppState, ServerFnError> {
    let state = app_state()?;
    let authorized = match token {
        Some(token) => state.verify_session(&token).await,
        None => false,
    };
    if authorized {
        Ok(state)
    } else {
        Err(ServerFnError::ServerError(String::from("unauthorized")))
    }
}

#[cfg(feature = "ssr")]
fn server_error(error: crate::error::AppError) -> ServerFnError {
    ServerFnError::ServerError(error.to_string())
}
