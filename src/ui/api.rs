use crate::api_types::{
    AuthResponse, AuthStatusResponse, BackupInfo, CommandReport, ConfigResponse, RawConfigResponse,
    RestoreBackupResponse, SaveRawRequest, SaveRecordsRequest, SaveResponse, TestConfigRequest,
};
use crate::server_fns;

pub async fn auth_status() -> Result<AuthStatusResponse, String> {
    server_fns::auth_status().await.map_err(server_fn_error)
}

pub async fn setup_password(password: String) -> Result<AuthResponse, String> {
    server_fns::setup_password(password)
        .await
        .map_err(server_fn_error)
}

pub async fn login(password: String) -> Result<AuthResponse, String> {
    server_fns::login(password).await.map_err(server_fn_error)
}

pub async fn logout(token: Option<String>) -> Result<(), String> {
    server_fns::logout(token).await.map_err(server_fn_error)
}

pub async fn get_config(token: Option<String>) -> Result<ConfigResponse, String> {
    server_fns::get_config(token).await.map_err(server_fn_error)
}

pub async fn save_records(
    token: Option<String>,
    payload: SaveRecordsRequest,
) -> Result<SaveResponse, String> {
    server_fns::save_records(token, payload.records, payload.apply)
        .await
        .map_err(server_fn_error)
}

pub async fn get_raw_config(token: Option<String>) -> Result<RawConfigResponse, String> {
    server_fns::get_raw_config(token)
        .await
        .map_err(server_fn_error)
}

pub async fn save_raw_config(
    token: Option<String>,
    payload: SaveRawRequest,
) -> Result<SaveResponse, String> {
    server_fns::save_raw_config(token, payload.content, payload.apply)
        .await
        .map_err(server_fn_error)
}

pub async fn test_config(
    token: Option<String>,
    payload: TestConfigRequest,
) -> Result<CommandReport, String> {
    server_fns::test_config(token, payload.content)
        .await
        .map_err(server_fn_error)
}

pub async fn list_backups(token: Option<String>) -> Result<Vec<BackupInfo>, String> {
    server_fns::list_backups(token)
        .await
        .map_err(server_fn_error)
}

pub async fn restore_backup(
    token: Option<String>,
    id: String,
) -> Result<RestoreBackupResponse, String> {
    server_fns::restore_backup(token, id)
        .await
        .map_err(server_fn_error)
}

fn server_fn_error(error: leptos::prelude::ServerFnError) -> String {
    error.to_string()
}
