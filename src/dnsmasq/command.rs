use std::path::Path;
use std::process::Stdio;

use tokio::process::Command;

use crate::api_types::CommandReport;
use crate::error::{AppError, AppResult};

#[derive(Clone, Debug)]
pub struct DnsmasqCommand {
    bin: String,
}

impl DnsmasqCommand {
    pub fn new(bin: impl Into<String>) -> Self {
        Self { bin: bin.into() }
    }

    pub async fn test_config(&self, config_path: &Path) -> AppResult<CommandReport> {
        let arg = format!("--conf-file={}", config_path.display());
        let output = Command::new(&self.bin)
            .arg("--test")
            .arg(&arg)
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .output()
            .await?;

        let report = CommandReport {
            success: output.status.success(),
            stdout: String::from_utf8_lossy(&output.stdout).to_string(),
            stderr: String::from_utf8_lossy(&output.stderr).to_string(),
        };

        if report.success {
            Ok(report)
        } else {
            Err(AppError::CommandFailed {
                program: self.bin.clone(),
                args: format!("--test {arg}"),
                status: output.status.to_string(),
                stdout: report.stdout,
                stderr: report.stderr,
            })
        }
    }
}
