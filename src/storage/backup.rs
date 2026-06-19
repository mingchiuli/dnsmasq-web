use std::path::{Path, PathBuf};

use chrono::{DateTime, Utc};
use tokio::fs;

use crate::api_types::BackupInfo;
use crate::error::AppResult;

pub async fn create_backup(config_file: &Path, backup_dir: &Path) -> AppResult<BackupInfo> {
    fs::create_dir_all(backup_dir).await?;
    let created_at = Utc::now();
    let id = created_at.format("%Y%m%d-%H%M%S%.9f").to_string();
    let backup_path = backup_dir.join(format!("dnsmasq.conf.{id}"));
    fs::copy(config_file, &backup_path).await?;
    backup_info(id, backup_path, created_at).await
}

pub async fn list_backups(backup_dir: &Path) -> AppResult<Vec<BackupInfo>> {
    let mut backups = Vec::new();
    if fs::metadata(backup_dir).await.is_err() {
        return Ok(backups);
    }

    let mut entries = fs::read_dir(backup_dir).await?;
    while let Some(entry) = entries.next_entry().await? {
        let path = entry.path();
        if !path.is_file() {
            continue;
        }
        let Some(file_name) = path.file_name().and_then(|name| name.to_str()) else {
            continue;
        };
        let Some(id) = file_name.strip_prefix("dnsmasq.conf.") else {
            continue;
        };
        let metadata = fs::metadata(&path).await?;
        let created_at = metadata
            .modified()
            .ok()
            .map(DateTime::<Utc>::from)
            .unwrap_or_else(Utc::now);
        backups.push(BackupInfo {
            id: id.into(),
            path: path.display().to_string(),
            created_at,
            size: metadata.len(),
        });
    }

    backups.sort_by(|a, b| b.created_at.cmp(&a.created_at));
    Ok(backups)
}

pub fn backup_path(backup_dir: &Path, id: &str) -> PathBuf {
    backup_dir.join(format!("dnsmasq.conf.{id}"))
}

async fn backup_info(
    id: String,
    path: PathBuf,
    created_at: DateTime<Utc>,
) -> AppResult<BackupInfo> {
    let size = fs::metadata(&path).await?.len();
    Ok(BackupInfo {
        id,
        path: path.display().to_string(),
        created_at,
        size,
    })
}
