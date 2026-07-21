use std::io::ErrorKind;
use std::os::unix::fs::PermissionsExt;
use std::path::Path;

use tokio::fs;
use tokio::io::AsyncWriteExt;
use uuid::Uuid;

use crate::error::{AppError, AppResult};

pub async fn load(path: &Path) -> AppResult<Option<String>> {
    let content = match fs::read_to_string(path).await {
        Ok(content) => content,
        Err(error) if error.kind() == ErrorKind::NotFound => return Ok(None),
        Err(error) => return Err(error.into()),
    };
    let password_hash = content.trim();
    if password_hash.is_empty() {
        return Err(AppError::Auth(String::from("credentials file is empty")));
    }
    Ok(Some(password_hash.into()))
}

pub async fn store(path: &Path, password_hash: &str) -> AppResult<()> {
    let parent = path.parent().unwrap_or_else(|| Path::new("."));
    ensure_private_parent(parent).await?;

    let file_name = path
        .file_name()
        .and_then(|name| name.to_str())
        .unwrap_or("password.hash");
    let temp_path = parent.join(format!(".{file_name}.dnsmasqweb-{}.tmp", Uuid::new_v4()));

    let result = async {
        let mut options = fs::OpenOptions::new();
        options.write(true).create_new(true).mode(0o600);
        let mut file = options.open(&temp_path).await?;
        file.write_all(password_hash.as_bytes()).await?;
        file.write_all(b"\n").await?;
        file.sync_all().await?;
        drop(file);

        fs::rename(&temp_path, path).await?;
        fs::File::open(parent).await?.sync_all().await?;
        AppResult::Ok(())
    }
    .await;

    if result.is_err() {
        let _ = fs::remove_file(&temp_path).await;
    }
    result
}

async fn ensure_private_parent(parent: &Path) -> AppResult<()> {
    match fs::metadata(parent).await {
        Ok(metadata) if metadata.is_dir() => Ok(()),
        Ok(_) => Err(AppError::Auth(format!(
            "credentials parent is not a directory: {}",
            parent.display()
        ))),
        Err(error) if error.kind() == ErrorKind::NotFound => {
            fs::create_dir_all(parent).await?;
            fs::set_permissions(parent, std::fs::Permissions::from_mode(0o700)).await?;
            Ok(())
        }
        Err(error) => Err(error.into()),
    }
}
