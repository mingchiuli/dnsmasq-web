use std::process::Stdio;

use tokio::process::Command;

use crate::api_types::{CommandReport, ServiceStatus};
use crate::error::{AppError, AppResult};

#[derive(Clone, Debug)]
pub struct Systemd {
    service: String,
}

impl Systemd {
    pub fn new(service: impl Into<String>) -> Self {
        Self {
            service: service.into(),
        }
    }

    pub async fn reload(&self) -> AppResult<CommandReport> {
        self.run_systemctl(&["reload", &self.service]).await
    }

    pub async fn restart(&self) -> AppResult<CommandReport> {
        self.run_systemctl(&["restart", &self.service]).await
    }

    pub async fn reload_or_restart(&self) -> AppResult<CommandReport> {
        match self.reload().await {
            Ok(report) => Ok(report),
            Err(_) => self.restart().await,
        }
    }

    pub async fn status(&self) -> ServiceStatus {
        match Command::new("systemctl")
            .arg("is-active")
            .arg(&self.service)
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .output()
            .await
        {
            Ok(output) => {
                let stdout: String = String::from_utf8_lossy(&output.stdout).trim().into();
                ServiceStatus {
                    active: output.status.success() && stdout == "active",
                    description: if stdout.is_empty() {
                        String::from_utf8_lossy(&output.stderr).trim().into()
                    } else {
                        stdout
                    },
                }
            }
            Err(error) => ServiceStatus {
                active: false,
                description: error.to_string(),
            },
        }
    }

    async fn run_systemctl(&self, args: &[&str]) -> AppResult<CommandReport> {
        let output = Command::new("systemctl")
            .args(args)
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
                program: String::from("systemctl"),
                args: args.join(" "),
                status: output.status.to_string(),
                stdout: report.stdout,
                stderr: report.stderr,
            })
        }
    }
}
