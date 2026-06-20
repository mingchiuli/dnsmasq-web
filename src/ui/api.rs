use crate::api_types::{
    AuthResponse, AuthStatusResponse, BackupInfo, CommandReport, ConfigResponse, RawConfigResponse,
    RestoreBackupResponse, SaveRawRequest, SaveRecordsRequest, SaveResponse, TestConfigRequest,
};
use crate::i18n::Locale;
use crate::server_fns;

pub async fn auth_status() -> Result<AuthStatusResponse, String> {
    server_fns::auth_status().await.map_err(server_fn_error)
}

pub async fn set_locale(locale: Locale) -> Result<(), String> {
    server_fns::set_locale(locale)
        .await
        .map_err(server_fn_error)
}

pub async fn setup_password(password: String) -> Result<AuthResponse, String> {
    server_fns::setup_password(password)
        .await
        .map_err(server_fn_error)
}

pub async fn login(password: String) -> Result<AuthResponse, String> {
    server_fns::login(password).await.map_err(server_fn_error)
}

pub async fn logout() -> Result<(), String> {
    server_fns::logout().await.map_err(server_fn_error)
}

pub async fn get_config() -> Result<ConfigResponse, String> {
    server_fns::get_config().await.map_err(server_fn_error)
}

pub async fn save_records(payload: SaveRecordsRequest) -> Result<SaveResponse, String> {
    server_fns::save_records(payload.records, payload.apply)
        .await
        .map_err(server_fn_error)
}

pub async fn get_raw_config() -> Result<RawConfigResponse, String> {
    server_fns::get_raw_config().await.map_err(server_fn_error)
}

pub async fn save_raw_config(payload: SaveRawRequest) -> Result<SaveResponse, String> {
    server_fns::save_raw_config(payload.content, payload.apply)
        .await
        .map_err(server_fn_error)
}

pub async fn test_config(payload: TestConfigRequest) -> Result<CommandReport, String> {
    server_fns::test_config(payload.content)
        .await
        .map_err(server_fn_error)
}

pub async fn list_backups() -> Result<Vec<BackupInfo>, String> {
    server_fns::list_backups().await.map_err(server_fn_error)
}

pub async fn restore_backup(id: String) -> Result<RestoreBackupResponse, String> {
    server_fns::restore_backup(id)
        .await
        .map_err(server_fn_error)
}

pub async fn delete_backup(id: String) -> Result<(), String> {
    server_fns::delete_backup(id).await.map_err(server_fn_error)
}

fn server_fn_error(error: leptos::prelude::ServerFnError) -> String {
    error.to_string()
}
