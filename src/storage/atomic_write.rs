use std::io;
use std::os::unix::fs::MetadataExt;
use std::path::{Path, PathBuf};

use tokio::fs;
use tokio::io::AsyncWriteExt;
use tokio::task;
use uuid::Uuid;
use xattr::FileExt;

use crate::error::{AppError, AppResult};

pub async fn resolve_target(target: &Path) -> AppResult<PathBuf> {
    let link_metadata = fs::symlink_metadata(target).await?;
    let target = if link_metadata.file_type().is_symlink() {
        fs::canonicalize(target).await?
    } else {
        target.to_path_buf()
    };
    let metadata = fs::metadata(&target).await?;
    if !metadata.is_file() {
        return Err(AppError::InvalidConfig(format!(
            "config path is not a regular file: {}",
            target.display()
        )));
    }
    Ok(target)
}

pub async fn write_temp_near(target: &Path, content: &str) -> AppResult<PathBuf> {
    let target = resolve_target(target).await?;
    let metadata = fs::metadata(&target).await?;
    write_temp_for_target(&target, content, metadata).await
}

pub async fn replace(target: &Path, content: &str) -> AppResult<()> {
    let target = resolve_target(target).await?;
    let metadata = fs::metadata(&target).await?;
    let temp_path = write_temp_for_target(&target, content, metadata).await?;

    if let Err(error) = fs::rename(&temp_path, &target).await {
        let _ = fs::remove_file(&temp_path).await;
        return Err(error.into());
    }
    sync_parent(&target).await?;
    Ok(())
}

async fn write_temp_for_target(
    target: &Path,
    content: &str,
    metadata: std::fs::Metadata,
) -> AppResult<PathBuf> {
    let parent = target.parent().unwrap_or_else(|| Path::new("."));
    let file_name = target
        .file_name()
        .and_then(|name| name.to_str())
        .unwrap_or("dnsmasq.conf");
    let temp_path = parent.join(format!(".{file_name}.dnsmasqweb-{}.tmp", Uuid::new_v4()));

    let result = async {
        let mut options = fs::OpenOptions::new();
        options.write(true).create_new(true).mode(metadata.mode());
        let mut file = options.open(&temp_path).await?;
        file.write_all(content.as_bytes()).await?;
        file.flush().await?;

        let file = file.into_std().await;
        let source = target.to_path_buf();
        task::spawn_blocking(move || preserve_metadata_and_sync(source, file, metadata))
            .await
            .map_err(join_error)??;
        AppResult::Ok(())
    }
    .await;

    match result {
        Ok(()) => Ok(temp_path),
        Err(error) => {
            let _ = fs::remove_file(&temp_path).await;
            Err(error)
        }
    }
}

fn preserve_metadata_and_sync(
    source_path: PathBuf,
    destination: std::fs::File,
    metadata: std::fs::Metadata,
) -> io::Result<()> {
    std::os::unix::fs::fchown(&destination, Some(metadata.uid()), Some(metadata.gid()))?;
    destination.set_permissions(metadata.permissions())?;

    let source = std::fs::File::open(source_path)?;
    match source.list_xattr() {
        Ok(attributes) => {
            for name in attributes {
                if let Some(value) = source.get_xattr(&name)? {
                    destination.set_xattr(name, &value)?;
                }
            }
        }
        Err(error) if error.kind() == io::ErrorKind::Unsupported => {}
        Err(error) => return Err(error),
    }
    destination.sync_all()
}

async fn sync_parent(target: &Path) -> AppResult<()> {
    let parent = target
        .parent()
        .unwrap_or_else(|| Path::new("."))
        .to_path_buf();
    task::spawn_blocking(move || std::fs::File::open(parent)?.sync_all())
        .await
        .map_err(join_error)??;
    Ok(())
}

fn join_error(error: task::JoinError) -> AppError {
    io::Error::other(format!("filesystem task failed: {error}")).into()
}

#[cfg(test)]
mod tests {
    use std::os::unix::fs::{MetadataExt, PermissionsExt, symlink};
    use std::path::{Path, PathBuf};
    use std::time::{SystemTime, UNIX_EPOCH};

    use tokio::fs;

    use super::replace;

    #[tokio::test]
    async fn replacement_preserves_mode_ownership_and_extended_attributes() {
        let paths = TestPaths::new("metadata");
        let config = paths.root.join("dnsmasq.conf");
        fs::write(&config, "old=true\n")
            .await
            .expect("write original config");
        fs::set_permissions(&config, std::fs::Permissions::from_mode(0o640))
            .await
            .expect("set original permissions");
        let xattr_name = "user.dnsmasqweb-test";
        let xattr_was_set = xattr::set(&config, xattr_name, b"preserve-me").is_ok();
        let before = fs::metadata(&config).await.expect("original metadata");

        replace(&config, "new=true\n")
            .await
            .expect("replace config");

        let after = fs::metadata(&config).await.expect("replacement metadata");
        assert_eq!(fs::read_to_string(&config).await.unwrap(), "new=true\n");
        assert_eq!(after.permissions().mode() & 0o7777, 0o640);
        assert_eq!(after.uid(), before.uid());
        assert_eq!(after.gid(), before.gid());
        if xattr_was_set {
            assert_eq!(
                xattr::get(&config, xattr_name).expect("read replacement xattr"),
                Some(b"preserve-me".to_vec())
            );
        }
        assert_no_temp_files(&paths.root);
    }

    #[tokio::test]
    async fn replacement_follows_symlink_without_replacing_it() {
        let paths = TestPaths::new("symlink");
        let config = paths.root.join("dnsmasq.real.conf");
        let link = paths.root.join("dnsmasq.conf");
        fs::write(&config, "old=true\n")
            .await
            .expect("write original config");
        symlink(&config, &link).expect("create config symlink");

        replace(&link, "new=true\n")
            .await
            .expect("replace symlink target");

        assert!(
            fs::symlink_metadata(&link)
                .await
                .expect("symlink metadata")
                .file_type()
                .is_symlink()
        );
        assert_eq!(fs::read_to_string(&config).await.unwrap(), "new=true\n");
        assert_no_temp_files(&paths.root);
    }

    #[tokio::test]
    async fn replacement_rejects_non_regular_targets() {
        let paths = TestPaths::new("directory");
        let directory = paths.root.join("dnsmasq.conf");
        fs::create_dir(&directory)
            .await
            .expect("create non-file target");

        let error = replace(&directory, "new=true\n")
            .await
            .expect_err("reject directory target");

        assert!(error.to_string().contains("not a regular file"));
        assert_no_temp_files(&paths.root);
    }

    fn assert_no_temp_files(directory: &Path) {
        let has_temp = std::fs::read_dir(directory)
            .expect("read test directory")
            .filter_map(Result::ok)
            .filter_map(|entry| entry.file_name().into_string().ok())
            .any(|name| name.contains(".dnsmasqweb-") && name.ends_with(".tmp"));
        assert!(!has_temp, "temporary file should be cleaned up");
    }

    struct TestPaths {
        root: PathBuf,
    }

    impl TestPaths {
        fn new(name: &str) -> Self {
            let nanos = SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .map(|duration| duration.as_nanos())
                .unwrap_or_default();
            let root = std::env::temp_dir().join(format!(
                "dnsmasqweb-atomic-write-{name}-{}-{nanos}",
                std::process::id()
            ));
            std::fs::create_dir_all(&root).expect("create test directory");
            Self { root }
        }
    }

    impl Drop for TestPaths {
        fn drop(&mut self) {
            let _ = std::fs::remove_dir_all(&self.root);
        }
    }
}
