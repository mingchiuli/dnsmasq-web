use std::future::Future;
use std::path::Path;
use std::pin::Pin;

use tokio::fs;
use tracing::{error, info};

use crate::api_types::{BackupInfo, CommandReport};
use crate::dnsmasq::command::DnsmasqCommand;
use crate::dnsmasq::systemd::Systemd;
use crate::error::{AppError, AppResult, RollbackStatus};
use crate::storage::{atomic_write, backup};

pub type CommandFuture<'a> = Pin<Box<dyn Future<Output = AppResult<CommandReport>> + Send + 'a>>;

pub trait ConfigTester: Send + Sync {
    fn test_config<'a>(&'a self, config_path: &'a Path) -> CommandFuture<'a>;
}

pub trait ServiceRestarter: Send + Sync {
    fn restart(&self) -> CommandFuture<'_>;
}

impl ConfigTester for DnsmasqCommand {
    fn test_config<'a>(&'a self, config_path: &'a Path) -> CommandFuture<'a> {
        Box::pin(DnsmasqCommand::test_config(self, config_path))
    }
}

impl ServiceRestarter for Systemd {
    fn restart(&self) -> CommandFuture<'_> {
        Box::pin(Systemd::restart(self))
    }
}

pub struct ConfigApplyRequest<'a> {
    pub config_file: &'a Path,
    pub backup_dir: &'a Path,
    pub content: &'a str,
    pub apply: bool,
    pub source: &'static str,
}

#[derive(Debug)]
pub struct ConfigApplyResult {
    pub backup: BackupInfo,
    pub test: CommandReport,
    pub reload: Option<CommandReport>,
}

pub async fn apply_config<T, R>(
    request: ConfigApplyRequest<'_>,
    tester: &T,
    restarter: &R,
) -> AppResult<ConfigApplyResult>
where
    T: ConfigTester + ?Sized,
    R: ServiceRestarter + ?Sized,
{
    let config_file = atomic_write::resolve_target(request.config_file).await?;
    let test = test_content(&config_file, request.content, tester).await?;
    let backup = backup::create_backup(&config_file, request.backup_dir).await?;
    atomic_write::replace(&config_file, request.content).await?;

    let reload = if request.apply {
        match restarter.restart().await {
            Ok(report) => Some(report),
            Err(reload_error) => {
                error!(
                    source = request.source,
                    %reload_error,
                    backup = %backup.path,
                    "dnsmasq restart failed after config replacement; rolling back"
                );
                let rollback =
                    rollback_config(&config_file, request.backup_dir, &backup, tester, restarter)
                        .await;
                log_rollback_status(request.source, &backup, &rollback);
                return Err(AppError::ConfigApplyFailed {
                    reload_error: Box::new(reload_error),
                    rollback,
                });
            }
        }
    } else {
        None
    };

    info!(
        source = request.source,
        apply = request.apply,
        backup = %backup.path,
        "config saved"
    );

    Ok(ConfigApplyResult {
        backup,
        test,
        reload,
    })
}

pub async fn test_content<T>(
    config_file: &Path,
    content: &str,
    tester: &T,
) -> AppResult<CommandReport>
where
    T: ConfigTester + ?Sized,
{
    let temp_path = atomic_write::write_temp_near(config_file, content).await?;
    let report = tester.test_config(&temp_path).await;
    let _ = fs::remove_file(&temp_path).await;
    report
}

async fn rollback_config<T, R>(
    config_file: &Path,
    backup_dir: &Path,
    backup: &BackupInfo,
    tester: &T,
    restarter: &R,
) -> RollbackStatus
where
    T: ConfigTester + ?Sized,
    R: ServiceRestarter + ?Sized,
{
    let backup_path = match backup::checked_backup_path(backup_dir, &backup.id) {
        Ok(path) => path,
        Err(error) => {
            return RollbackStatus::Failed {
                error: error.to_string(),
            };
        }
    };
    let content = match fs::read_to_string(&backup_path).await {
        Ok(content) => content,
        Err(error) => {
            return RollbackStatus::Failed {
                error: format!("failed to read backup {}: {error}", backup.path),
            };
        }
    };

    if let Err(error) = test_content(config_file, &content, tester).await {
        return RollbackStatus::Failed {
            error: format!("previous config backup failed dnsmasq test: {error}"),
        };
    }

    if let Err(error) = atomic_write::replace(config_file, &content).await {
        return RollbackStatus::Failed {
            error: format!(
                "failed to replace config with backup {}: {error}",
                backup.path
            ),
        };
    }

    match restarter.restart().await {
        Ok(_) => RollbackStatus::Restored,
        Err(error) => RollbackStatus::RestoredRestartFailed {
            error: error.to_string(),
        },
    }
}

fn log_rollback_status(source: &'static str, backup: &BackupInfo, rollback: &RollbackStatus) {
    match rollback {
        RollbackStatus::Restored => {
            info!(
                source,
                backup = %backup.path,
                "previous config restored after restart failure"
            );
        }
        RollbackStatus::RestoredRestartFailed {
            error: rollback_error,
        } => {
            error!(
                source,
                backup = %backup.path,
                %rollback_error,
                "previous config restored but dnsmasq restart failed"
            );
        }
        RollbackStatus::Failed {
            error: rollback_error,
        } => {
            error!(
                source,
                backup = %backup.path,
                %rollback_error,
                "failed to restore previous config after restart failure"
            );
        }
    }
}

#[cfg(test)]
mod tests {
    use std::collections::VecDeque;
    use std::os::unix::fs::symlink;
    use std::path::{Path, PathBuf};
    use std::sync::atomic::{AtomicUsize, Ordering};
    use std::time::{SystemTime, UNIX_EPOCH};

    use tokio::fs;
    use tokio::sync::Mutex;

    use super::{CommandFuture, ConfigApplyRequest, ConfigTester, ServiceRestarter, apply_config};
    use crate::api_types::CommandReport;
    use crate::error::{AppError, RollbackStatus};

    #[tokio::test]
    async fn restart_failure_rolls_back_to_previous_config() {
        let paths = TestPaths::new("restart-failure-rolls-back");
        paths.write_config("address=/old.example/10.0.0.1\n").await;
        let tester = FakeTester::default();
        let restarter = FakeRestarter::new(vec![
            Err(command_error("restart dnsmasq")),
            Ok(command_report("rollback restart")),
        ]);

        let result = apply_config(
            ConfigApplyRequest {
                config_file: &paths.config_file,
                backup_dir: &paths.backup_dir,
                content: "address=/new.example/10.0.0.2\n",
                apply: true,
                source: "test",
            },
            &tester,
            &restarter,
        )
        .await;

        let error = result.expect_err("restart failure should fail apply");
        match error {
            AppError::ConfigApplyFailed { rollback, .. } => {
                assert!(matches!(rollback, RollbackStatus::Restored));
            }
            other => panic!("unexpected error: {other}"),
        }
        assert_eq!(paths.read_config().await, "address=/old.example/10.0.0.1\n");
        assert_eq!(restarter.calls(), 2);
        assert_eq!(tester.calls(), 2);
        paths.cleanup();
    }

    #[tokio::test]
    async fn apply_false_replaces_config_without_restart() {
        let paths = TestPaths::new("apply-false-no-restart");
        paths.write_config("address=/old.example/10.0.0.1\n").await;
        let tester = FakeTester::default();
        let restarter = FakeRestarter::new(Vec::new());

        let result = apply_config(
            ConfigApplyRequest {
                config_file: &paths.config_file,
                backup_dir: &paths.backup_dir,
                content: "address=/new.example/10.0.0.2\n",
                apply: false,
                source: "test",
            },
            &tester,
            &restarter,
        )
        .await
        .expect("save without apply");

        assert!(result.reload.is_none());
        assert_eq!(paths.read_config().await, "address=/new.example/10.0.0.2\n");
        assert_eq!(restarter.calls(), 0);
        assert_eq!(tester.calls(), 1);
        paths.cleanup();
    }

    #[tokio::test]
    async fn apply_through_symlink_preserves_link_and_backs_up_target() {
        let paths = TestPaths::new("symlink-target");
        let real_config = paths.root.join("dnsmasq.real.conf");
        fs::write(&real_config, "address=/old.example/10.0.0.1\n")
            .await
            .expect("write real config");
        symlink(&real_config, &paths.config_file).expect("create config symlink");
        let tester = FakeTester::default();
        let restarter = FakeRestarter::new(Vec::new());

        let result = apply_config(
            ConfigApplyRequest {
                config_file: &paths.config_file,
                backup_dir: &paths.backup_dir,
                content: "address=/new.example/10.0.0.2\n",
                apply: false,
                source: "test",
            },
            &tester,
            &restarter,
        )
        .await
        .expect("apply through symlink");

        assert!(
            fs::symlink_metadata(&paths.config_file)
                .await
                .expect("symlink metadata")
                .file_type()
                .is_symlink()
        );
        assert_eq!(
            fs::read_to_string(&real_config).await.unwrap(),
            "address=/new.example/10.0.0.2\n"
        );
        assert_eq!(
            fs::read_to_string(&result.backup.path).await.unwrap(),
            "address=/old.example/10.0.0.1\n"
        );
        paths.cleanup();
    }

    #[tokio::test]
    async fn rollback_test_failure_keeps_new_config_and_reports_failure() {
        let paths = TestPaths::new("rollback-test-failure");
        paths.write_config("address=/old.example/10.0.0.1\n").await;
        let tester = FakeTester::new(vec![
            Ok(command_report("candidate")),
            Err(command_error("old test")),
        ]);
        let restarter = FakeRestarter::new(vec![Err(command_error("restart dnsmasq"))]);

        let result = apply_config(
            ConfigApplyRequest {
                config_file: &paths.config_file,
                backup_dir: &paths.backup_dir,
                content: "address=/new.example/10.0.0.2\n",
                apply: true,
                source: "test",
            },
            &tester,
            &restarter,
        )
        .await;

        let error = result.expect_err("rollback test failure should fail apply");
        match error {
            AppError::ConfigApplyFailed { rollback, .. } => {
                assert!(matches!(rollback, RollbackStatus::Failed { .. }));
            }
            other => panic!("unexpected error: {other}"),
        }
        assert_eq!(paths.read_config().await, "address=/new.example/10.0.0.2\n");
        assert_eq!(restarter.calls(), 1);
        assert_eq!(tester.calls(), 2);
        paths.cleanup();
    }

    #[derive(Default)]
    struct FakeTester {
        responses: Mutex<VecDeque<AppResult<CommandReport>>>,
        calls: AtomicUsize,
    }

    impl FakeTester {
        fn new(responses: Vec<AppResult<CommandReport>>) -> Self {
            Self {
                responses: Mutex::new(responses.into()),
                calls: AtomicUsize::new(0),
            }
        }

        fn calls(&self) -> usize {
            self.calls.load(Ordering::SeqCst)
        }
    }

    impl ConfigTester for FakeTester {
        fn test_config<'a>(&'a self, _config_path: &'a Path) -> CommandFuture<'a> {
            Box::pin(async move {
                self.calls.fetch_add(1, Ordering::SeqCst);
                self.responses
                    .lock()
                    .await
                    .pop_front()
                    .unwrap_or_else(|| Ok(command_report("test")))
            })
        }
    }

    struct FakeRestarter {
        responses: Mutex<VecDeque<AppResult<CommandReport>>>,
        calls: AtomicUsize,
    }

    impl FakeRestarter {
        fn new(responses: Vec<AppResult<CommandReport>>) -> Self {
            Self {
                responses: Mutex::new(responses.into()),
                calls: AtomicUsize::new(0),
            }
        }

        fn calls(&self) -> usize {
            self.calls.load(Ordering::SeqCst)
        }
    }

    impl ServiceRestarter for FakeRestarter {
        fn restart(&self) -> CommandFuture<'_> {
            Box::pin(async move {
                self.calls.fetch_add(1, Ordering::SeqCst);
                self.responses
                    .lock()
                    .await
                    .pop_front()
                    .unwrap_or_else(|| Ok(command_report("restart")))
            })
        }
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
                "dnsmasqweb-config-apply-{name}-{}-{nanos}",
                std::process::id()
            ));
            let config_file = root.join("dnsmasq.conf");
            let backup_dir = root.join("backups");
            std::fs::create_dir_all(&root).expect("create temp root");
            Self {
                root,
                config_file,
                backup_dir,
            }
        }

        async fn write_config(&self, content: &str) {
            fs::write(&self.config_file, content)
                .await
                .expect("write config");
        }

        async fn read_config(&self) -> String {
            fs::read_to_string(&self.config_file)
                .await
                .expect("read config")
        }

        fn cleanup(&self) {
            let _ = std::fs::remove_dir_all(&self.root);
        }
    }

    fn command_report(stdout: &str) -> CommandReport {
        CommandReport {
            success: true,
            stdout: stdout.into(),
            stderr: String::new(),
        }
    }

    fn command_error(stderr: &str) -> AppError {
        AppError::CommandFailed {
            program: String::from("fake"),
            args: String::from("restart"),
            status: String::from("exit status: 1"),
            stdout: String::new(),
            stderr: stderr.into(),
        }
    }

    type AppResult<T> = Result<T, AppError>;
}
