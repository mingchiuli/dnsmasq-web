use std::path::{Path, PathBuf};

use tokio::fs;
use uuid::Uuid;

use crate::error::AppResult;

pub async fn write_temp_near(target: &Path, content: &str) -> AppResult<PathBuf> {
    let parent = target.parent().unwrap_or_else(|| Path::new("."));
    let file_name = target
        .file_name()
        .and_then(|name| name.to_str())
        .unwrap_or("dnsmasq.conf");
    let temp_path = parent.join(format!(".{file_name}.dnsmasqweb-{}.tmp", Uuid::new_v4()));
    fs::write(&temp_path, content).await?;
    Ok(temp_path)
}

pub async fn replace(target: &Path, content: &str) -> AppResult<()> {
    let temp_path = write_temp_near(target, content).await?;
    fs::rename(&temp_path, target).await?;
    Ok(())
}
