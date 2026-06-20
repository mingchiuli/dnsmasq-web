use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

use crate::config::model::{DnsRecords, ValidationIssue};
use crate::i18n::Locale;

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct AuthStatusResponse {
    pub configured: bool,
    pub authenticated: bool,
    pub locale: Locale,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct AuthResponse {
    pub expires_at: DateTime<Utc>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ConfigResponse {
    pub records: DnsRecords,
    pub unmanaged_line_count: usize,
    pub warnings: Vec<ValidationIssue>,
    pub last_modified: Option<DateTime<Utc>>,
    pub service: ServiceStatus,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct SaveRecordsRequest {
    pub records: DnsRecords,
    pub apply: bool,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct SaveRawRequest {
    pub content: String,
    pub apply: bool,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct SaveResponse {
    pub applied: bool,
    pub backup: Option<BackupInfo>,
    pub test: CommandReport,
    pub reload: Option<CommandReport>,
    pub warnings: Vec<ValidationIssue>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct RawConfigResponse {
    pub content: String,
    pub last_modified: Option<DateTime<Utc>>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct TestConfigRequest {
    pub content: Option<String>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct CommandReport {
    pub success: bool,
    pub stdout: String,
    pub stderr: String,
}

#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub struct ServiceStatus {
    pub active: bool,
    pub description: String,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct BackupInfo {
    pub id: String,
    pub path: String,
    pub created_at: DateTime<Utc>,
    pub size: u64,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct RestoreBackupResponse {
    pub created_backup: BackupInfo,
    pub test: CommandReport,
    pub reload: Option<CommandReport>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ErrorResponse {
    pub message: String,
}
