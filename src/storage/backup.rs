use std::cmp::Reverse;
use std::num::NonZeroUsize;
use std::os::unix::fs::PermissionsExt;
use std::path::{Path, PathBuf};

use chrono::{DateTime, Utc};
use tokio::fs::{self, OpenOptions};
use tokio::io::AsyncWriteExt;

use crate::api_types::BackupInfo;
use crate::error::{AppError, AppResult};

const BACKUP_PREFIX: &str = "dnsmasq.conf.";

pub async fn create_backup(config_file: &Path, backup_dir: &Path) -> AppResult<BackupInfo> {
    ensure_backup_dir(backup_dir).await?;
    let created_at = Utc::now();
    let id = created_at.format("%Y%m%d-%H%M%S%.9f").to_string();
    let backup_path = backup_dir.join(format!("{BACKUP_PREFIX}{id}"));

    let result = async {
        let mut source = OpenOptions::new().read(true).open(config_file).await?;
        let mut destination = OpenOptions::new()
            .write(true)
            .create_new(true)
            .mode(0o600)
            .open(&backup_path)
            .await?;
        tokio::io::copy(&mut source, &mut destination).await?;
        destination.flush().await?;
        destination.sync_all().await?;
        backup_info(id, &backup_path, created_at).await
    }
    .await;

    if result.is_err() {
        let _ = fs::remove_file(&backup_path).await;
    }
    result
}

pub async fn list_backups(backup_dir: &Path) -> AppResult<Vec<BackupInfo>> {
    let mut backups = Vec::new();
    match fs::symlink_metadata(backup_dir).await {
        Ok(metadata) if metadata.file_type().is_dir() => {}
        Ok(_) => {
            return Err(AppError::InvalidConfig(format!(
                "backup directory is not a directory: {}",
                backup_dir.display()
            )));
        }
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => return Ok(backups),
        Err(error) => return Err(error.into()),
    }

    let mut entries = fs::read_dir(backup_dir).await?;
    while let Some(entry) = entries.next_entry().await? {
        let path = entry.path();
        let Some(file_name) = path.file_name().and_then(|name| name.to_str()) else {
            continue;
        };
        let Some(id) = file_name.strip_prefix(BACKUP_PREFIX) else {
            continue;
        };
        let metadata = fs::symlink_metadata(&path).await?;
        if !metadata.file_type().is_file() {
            return Err(AppError::InvalidConfig(format!(
                "backup is not a regular file: {id}"
            )));
        }
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

    backups.sort_by_key(|backup| Reverse(backup.created_at));
    Ok(backups)
}

pub async fn prune_backups(backup_dir: &Path, max_backups: NonZeroUsize) -> AppResult<usize> {
    let backups = list_backups(backup_dir).await?;
    let mut removed = 0;
    for backup in backups.into_iter().skip(max_backups.get()) {
        let path = checked_backup_file(backup_dir, &backup.id).await?;
        fs::remove_file(path).await?;
        removed += 1;
    }
    Ok(removed)
}

pub async fn delete_backup(backup_dir: &Path, id: &str) -> AppResult<()> {
    let path = checked_backup_file(backup_dir, id).await?;
    fs::remove_file(path).await?;
    Ok(())
}

pub async fn checked_backup_file(backup_dir: &Path, id: &str) -> AppResult<PathBuf> {
    let path = checked_backup_path(backup_dir, id)?;
    let metadata = fs::symlink_metadata(&path).await?;
    if !metadata.file_type().is_file() {
        return Err(AppError::InvalidConfig(format!(
            "backup is not a regular file: {id}"
        )));
    }
    Ok(path)
}

pub fn checked_backup_path(backup_dir: &Path, id: &str) -> AppResult<PathBuf> {
    validate_backup_id(id)?;
    Ok(backup_dir.join(format!("{BACKUP_PREFIX}{id}")))
}

async fn ensure_backup_dir(backup_dir: &Path) -> AppResult<()> {
    match fs::symlink_metadata(backup_dir).await {
        Ok(metadata) if metadata.file_type().is_dir() => {}
        Ok(_) => {
            return Err(AppError::InvalidConfig(format!(
                "backup directory is not a directory: {}",
                backup_dir.display()
            )));
        }
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => {
            fs::create_dir_all(backup_dir).await?;
        }
        Err(error) => return Err(error.into()),
    }

    let metadata = fs::symlink_metadata(backup_dir).await?;
    if !metadata.file_type().is_dir() {
        return Err(AppError::InvalidConfig(format!(
            "backup directory is not a directory: {}",
            backup_dir.display()
        )));
    }
    fs::set_permissions(backup_dir, std::fs::Permissions::from_mode(0o700)).await?;
    Ok(())
}

fn validate_backup_id(id: &str) -> AppResult<()> {
    if id.is_empty()
        || id.contains('/')
        || id.contains('\\')
        || id == "."
        || id == ".."
        || id.contains("..")
    {
        return Err(AppError::InvalidConfig(format!("invalid backup id: {id}")));
    }
    Ok(())
}

async fn backup_info(id: String, path: &Path, created_at: DateTime<Utc>) -> AppResult<BackupInfo> {
    let size = fs::symlink_metadata(&path).await?.len();
    Ok(BackupInfo {
        id,
        path: path.display().to_string(),
        created_at,
        size,
    })
}

#[cfg(test)]
mod tests {
    use std::num::NonZeroUsize;
    use std::os::unix::fs::{PermissionsExt, symlink};
    use std::path::PathBuf;
    use std::time::{SystemTime, UNIX_EPOCH};

    use tokio::fs;

    use super::{checked_backup_file, create_backup, list_backups, prune_backups};
    use crate::error::AppError;

    #[tokio::test]
    async fn rejects_backup_symlink() {
        let paths = TestPaths::new("reject-symlink");
        fs::create_dir_all(&paths.backup_dir)
            .await
            .expect("create backup dir");
        fs::write(&paths.config_file, "address=/example/10.0.0.1\n")
            .await
            .expect("write config");
        symlink(
            &paths.config_file,
            paths.backup_dir.join("dnsmasq.conf.unsafe"),
        )
        .expect("create symlink");

        let result = checked_backup_file(&paths.backup_dir, "unsafe").await;
        assert!(matches!(result, Err(AppError::InvalidConfig(_))));
        assert!(matches!(
            list_backups(&paths.backup_dir).await,
            Err(AppError::InvalidConfig(_))
        ));
        paths.cleanup();
    }

    #[tokio::test]
    async fn pruning_keeps_newest_backups() {
        let paths = TestPaths::new("prune");
        fs::write(&paths.config_file, "one")
            .await
            .expect("write config");
        let first = create_backup(&paths.config_file, &paths.backup_dir)
            .await
            .expect("first backup");
        fs::write(&paths.config_file, "two")
            .await
            .expect("update config");
        let second = create_backup(&paths.config_file, &paths.backup_dir)
            .await
            .expect("second backup");

        let directory_mode = std::fs::metadata(&paths.backup_dir)
            .expect("backup directory metadata")
            .permissions()
            .mode()
            & 0o777;
        let file_mode = std::fs::metadata(&second.path)
            .expect("backup metadata")
            .permissions()
            .mode()
            & 0o777;
        assert_eq!(directory_mode, 0o700);
        assert_eq!(file_mode, 0o600);

        let removed = prune_backups(
            &paths.backup_dir,
            NonZeroUsize::new(1).expect("non-zero limit"),
        )
        .await
        .expect("prune backups");

        assert_eq!(removed, 1);
        let backups = list_backups(&paths.backup_dir).await.expect("list backups");
        assert_eq!(backups.len(), 1);
        assert_eq!(backups[0].id, second.id);
        assert!(!PathBuf::from(first.path).exists());
        paths.cleanup();
    }

    struct TestPaths {
        root: PathBuf,
        config_file: PathBuf,
        backup_dir: PathBuf,
    }

    impl TestPaths {
        fn new(name: &str) -> Self {
            let nanos = SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .expect("system time")
                .as_nanos();
            let root = std::env::temp_dir().join(format!(
                "dnsmasqweb-backup-{name}-{}-{nanos}",
                std::process::id()
            ));
            std::fs::create_dir_all(&root).expect("create root");
            Self {
                config_file: root.join("dnsmasq.conf"),
                backup_dir: root.join("backups"),
                root,
            }
        }

        fn cleanup(&self) {
            let _ = std::fs::remove_dir_all(&self.root);
        }
    }
}
