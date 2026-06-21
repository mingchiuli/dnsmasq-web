use leptos::prelude::*;

#[cfg(feature = "ssr")]
use axum::http::header::{ACCEPT_LANGUAGE, COOKIE, SET_COOKIE};
#[cfg(feature = "ssr")]
use axum::http::request::Parts;
#[cfg(feature = "ssr")]
use axum::http::{HeaderName, HeaderValue};
#[cfg(feature = "ssr")]
use leptos_axum::ResponseOptions;

use crate::api_types::{
    AuthResponse, AuthStatusResponse, BackupInfo, CommandReport, ConfigResponse, RawConfigResponse,
    RestoreBackupResponse, SaveResponse, ServiceStatus,
};
use crate::config::model::DnsRecords;
#[cfg(feature = "ssr")]
use crate::error::AppError;
use crate::i18n::Locale;
#[cfg(feature = "ssr")]
use crate::server::auth::SESSION_COOKIE;
#[cfg(feature = "ssr")]
use crate::server::services;
#[cfg(feature = "ssr")]
use crate::server::state::AppState;

#[cfg(feature = "ssr")]
const LOCALE_COOKIE: &str = "dnsmasqweb_locale";
#[server(AuthStatus, "/api")]
pub async fn auth_status() -> Result<AuthStatusResponse, ServerFnError> {
    let state = app_state()?;
    Ok(services::auth_status(
        &state,
        request_cookie(SESSION_COOKIE).as_deref(),
        request_locale(),
    )
    .await)
}

#[server(SetLocale, "/api")]
pub async fn set_locale(locale: Locale) -> Result<(), ServerFnError> {
    set_locale_cookie(locale)?;
    Ok(())
}

#[server(SetupPassword, "/api")]
pub async fn setup_password(password: String) -> Result<AuthResponse, ServerFnError> {
    let state = app_state()?;
    let session = services::setup_password(&state, password)
        .await
        .map_err(server_error)?;
    set_session_cookie(&session.token)?;
    Ok(AuthResponse {
        expires_at: session.expires_at,
    })
}

#[server(Login, "/api")]
pub async fn login(password: String) -> Result<AuthResponse, ServerFnError> {
    let state = app_state()?;
    let session = services::login(&state, password)
        .await
        .map_err(server_error)?;
    set_session_cookie(&session.token)?;
    Ok(AuthResponse {
        expires_at: session.expires_at,
    })
}

#[server(Logout, "/api")]
pub async fn logout() -> Result<(), ServerFnError> {
    let state = app_state()?;
    let token = request_cookie(SESSION_COOKIE);
    services::logout(&state, token.as_deref()).await;
    clear_session_cookie()?;
    Ok(())
}

#[server(GetConfig, "/api")]
pub async fn get_config() -> Result<ConfigResponse, ServerFnError> {
    let state = app_state()?;
    services::get_config(&state).await.map_err(server_error)
}

#[server(SaveRecords, "/api")]
pub async fn save_records(records: DnsRecords, apply: bool) -> Result<SaveResponse, ServerFnError> {
    let state = app_state()?;
    services::save_records(&state, records, apply)
        .await
        .map_err(server_error)
}

#[server(GetRawConfig, "/api")]
pub async fn get_raw_config() -> Result<RawConfigResponse, ServerFnError> {
    let state = app_state()?;
    services::get_raw_config(&state).await.map_err(server_error)
}

#[server(SaveRawConfig, "/api")]
pub async fn save_raw_config(content: String, apply: bool) -> Result<SaveResponse, ServerFnError> {
    let state = app_state()?;
    services::save_raw_config(&state, content, apply)
        .await
        .map_err(server_error)
}

#[server(TestConfig, "/api")]
pub async fn test_config(content: Option<String>) -> Result<CommandReport, ServerFnError> {
    let state = app_state()?;
    services::test_config(&state, content)
        .await
        .map_err(server_error)
}

#[server(ReloadDnsmasq, "/api")]
pub async fn reload_dnsmasq() -> Result<CommandReport, ServerFnError> {
    let state = app_state()?;
    services::reload_dnsmasq(&state).await.map_err(server_error)
}

#[server(Status, "/api")]
pub async fn status() -> Result<ServiceStatus, ServerFnError> {
    let state = app_state()?;
    Ok(services::status(&state).await)
}

#[server(ListBackups, "/api")]
pub async fn list_backups() -> Result<Vec<BackupInfo>, ServerFnError> {
    let state = app_state()?;
    services::list_backups(&state).await.map_err(server_error)
}

#[server(RestoreBackup, "/api")]
pub async fn restore_backup(id: String) -> Result<RestoreBackupResponse, ServerFnError> {
    let state = app_state()?;
    services::restore_backup(&state, id)
        .await
        .map_err(server_error)
}

#[server(DeleteBackup, "/api")]
pub async fn delete_backup(id: String) -> Result<(), ServerFnError> {
    let state = app_state()?;
    services::delete_backup(&state, id)
        .await
        .map_err(server_error)
}

#[cfg(feature = "ssr")]
fn app_state() -> Result<AppState, ServerFnError> {
    use_context::<AppState>()
        .ok_or_else(|| ServerFnError::ServerError(String::from("missing app state")))
}

#[cfg(feature = "ssr")]
fn server_error(error: AppError) -> ServerFnError {
    ServerFnError::ServerError(error.to_string())
}

#[cfg(feature = "ssr")]
fn request_locale() -> Locale {
    request_cookie(LOCALE_COOKIE)
        .as_deref()
        .map(Locale::from)
        .unwrap_or_else(|| {
            request_header(ACCEPT_LANGUAGE)
                .as_deref()
                .map(Locale::from)
                .unwrap_or_default()
        })
}

#[cfg(feature = "ssr")]
fn request_cookie(cookie_name: &str) -> Option<String> {
    let parts = use_context::<Parts>()?;
    let cookie_header = parts.headers.get(COOKIE)?.to_str().ok()?;
    cookie_header
        .split(';')
        .filter_map(|cookie| cookie.trim().split_once('='))
        .find_map(|(name, value)| {
            if name == cookie_name {
                Some(value.to_string())
            } else {
                None
            }
        })
}

#[cfg(feature = "ssr")]
fn request_header(header_name: HeaderName) -> Option<String> {
    let parts = use_context::<Parts>()?;
    parts
        .headers
        .get(header_name)?
        .to_str()
        .ok()
        .map(String::from)
}

#[cfg(feature = "ssr")]
fn set_session_cookie(token: &str) -> Result<(), ServerFnError> {
    let cookie = format!("{SESSION_COOKIE}={token}; Path=/; Max-Age=86400; SameSite=Lax; HttpOnly");
    append_set_cookie(cookie)
}

#[cfg(feature = "ssr")]
fn set_locale_cookie(locale: Locale) -> Result<(), ServerFnError> {
    let cookie = format!(
        "{LOCALE_COOKIE}={}; Path=/; Max-Age=31536000; SameSite=Lax; HttpOnly",
        locale.code()
    );
    append_set_cookie(cookie)
}

#[cfg(feature = "ssr")]
fn clear_session_cookie() -> Result<(), ServerFnError> {
    let cookie = format!(
        "{SESSION_COOKIE}=; Path=/; Max-Age=0; Expires=Thu, 01 Jan 1970 00:00:00 GMT; SameSite=Lax; HttpOnly"
    );
    append_set_cookie(cookie)
}

#[cfg(feature = "ssr")]
fn append_set_cookie(cookie: String) -> Result<(), ServerFnError> {
    let Some(response) = use_context::<ResponseOptions>() else {
        return Err(ServerFnError::new(String::from("missing response options")));
    };
    let value = HeaderValue::from_str(&cookie)
        .map_err(|error| ServerFnError::new(format!("invalid session cookie: {error}")))?;
    response.append_header(SET_COOKIE, value);
    Ok(())
}
