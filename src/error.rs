use thiserror::Error;

#[derive(Debug, Error)]
pub enum AppError {
    #[error("invalid configuration: {0}")]
    InvalidConfig(String),

    #[error("parse error on line {line}: {message}")]
    ParseLine { line: usize, message: String },

    #[error("io error: {0}")]
    Io(#[from] std::io::Error),

    #[cfg(feature = "ssr")]
    #[error("command failed: {program} {args} exited with {status}: {stderr}")]
    CommandFailed {
        program: String,
        args: String,
        status: String,
        stdout: String,
        stderr: String,
    },

    #[cfg(feature = "ssr")]
    #[error("command timed out after {timeout_seconds}s: {program} {args}")]
    CommandTimedOut {
        program: String,
        args: String,
        timeout_seconds: u64,
    },

    #[cfg(feature = "ssr")]
    #[error("configuration changed since it was loaded")]
    ConfigConflict,

    #[cfg(feature = "ssr")]
    #[error("dnsmasq restart failed after config replacement: {reload_error}; rollback {rollback}")]
    ConfigApplyFailed {
        reload_error: Box<AppError>,
        rollback: RollbackStatus,
    },

    #[cfg(feature = "ssr")]
    #[error("unauthorized")]
    Unauthorized,

    #[cfg(feature = "ssr")]
    #[error("authentication error: {0}")]
    Auth(String),

    #[cfg(feature = "ssr")]
    #[error("too many login attempts; try again in {retry_after_seconds}s")]
    RateLimited { retry_after_seconds: u64 },
}

#[cfg(feature = "ssr")]
#[derive(Debug)]
pub enum RollbackStatus {
    Restored,
    RestoredRestartFailed { error: String },
    Failed { error: String },
}

#[cfg(feature = "ssr")]
impl std::fmt::Display for RollbackStatus {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Restored => write!(formatter, "restored previous config and restarted dnsmasq"),
            Self::RestoredRestartFailed { error } => {
                write!(
                    formatter,
                    "restored previous config but dnsmasq restart failed: {error}"
                )
            }
            Self::Failed { error } => {
                write!(formatter, "failed to restore previous config: {error}")
            }
        }
    }
}

pub type AppResult<T> = Result<T, AppError>;
