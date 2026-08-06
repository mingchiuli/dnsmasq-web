use std::time::Duration;

use crate::api_types::{CommandReport, ServiceStatus};
use crate::dnsmasq::process;
use crate::error::{AppError, AppResult};

#[derive(Clone, Debug)]
pub struct Systemd {
    service: String,
    timeout: Duration,
}

impl Systemd {
    pub fn new(service: impl Into<String>, timeout: Duration) -> Self {
        Self {
            service: service.into(),
            timeout,
        }
    }

    pub async fn restart(&self) -> AppResult<CommandReport> {
        self.run_systemctl(&["restart", &self.service]).await
    }

    pub async fn status(&self) -> ServiceStatus {
        match self.run_systemctl(&["is-active", &self.service]).await {
            Ok(report) => {
                let stdout = report.stdout.trim().to_string();
                ServiceStatus {
                    active: stdout == "active",
                    description: if stdout.is_empty() {
                        report.stderr.trim().into()
                    } else {
                        stdout
                    },
                }
            }
            Err(AppError::CommandFailed { stdout, stderr, .. }) => ServiceStatus {
                active: false,
                description: if stdout.trim().is_empty() {
                    stderr.trim().into()
                } else {
                    stdout.trim().into()
                },
            },
            Err(error) => ServiceStatus {
                active: false,
                description: error.to_string(),
            },
        }
    }

    async fn run_systemctl(&self, args: &[&str]) -> AppResult<CommandReport> {
        process::run("systemctl", args, self.timeout).await
    }
}
