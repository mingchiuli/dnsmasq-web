use std::process::Stdio;
use std::time::Duration;

use tokio::process::Command;
use tokio::time;

use crate::api_types::CommandReport;
use crate::error::{AppError, AppResult};

pub async fn run(program: &str, args: &[&str], timeout: Duration) -> AppResult<CommandReport> {
    let mut command = Command::new(program);
    command
        .args(args)
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .kill_on_drop(true);

    let output = match time::timeout(timeout, command.output()).await {
        Ok(result) => result?,
        Err(_) => {
            return Err(AppError::CommandTimedOut {
                program: program.into(),
                args: args.join(" "),
                timeout_seconds: timeout.as_secs().max(1),
            });
        }
    };

    let report = CommandReport {
        success: output.status.success(),
        stdout: String::from_utf8_lossy(&output.stdout).into_owned(),
        stderr: String::from_utf8_lossy(&output.stderr).into_owned(),
    };

    if report.success {
        Ok(report)
    } else {
        Err(AppError::CommandFailed {
            program: program.into(),
            args: args.join(" "),
            status: output.status.to_string(),
            stdout: report.stdout,
            stderr: report.stderr,
        })
    }
}

#[cfg(test)]
mod tests {
    use std::time::{Duration, Instant};

    use super::run;
    use crate::error::AppError;

    #[tokio::test]
    async fn reports_timeout_without_waiting_for_command_completion() {
        let started = Instant::now();
        let result = run("/bin/sh", &["-c", "sleep 5"], Duration::from_millis(50)).await;

        assert!(matches!(result, Err(AppError::CommandTimedOut { .. })));
        assert!(started.elapsed() < Duration::from_secs(2));
    }
}
