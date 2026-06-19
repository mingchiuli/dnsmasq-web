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
    #[error("unauthorized")]
    Unauthorized,

    #[cfg(feature = "ssr")]
    #[error("authentication error: {0}")]
    Auth(String),
}

pub type AppResult<T> = Result<T, AppError>;
