use std::path::{Path, PathBuf};

#[derive(Clone, Debug)]
pub struct StoragePaths {
    pub config_file: PathBuf,
    pub backup_dir: PathBuf,
}

impl StoragePaths {
    pub fn new(config_file: impl Into<PathBuf>, backup_dir: impl Into<PathBuf>) -> Self {
        Self {
            config_file: config_file.into(),
            backup_dir: backup_dir.into(),
        }
    }

    pub fn parent_dir(&self) -> &Path {
        self.config_file.parent().unwrap_or_else(|| Path::new("."))
    }
}
