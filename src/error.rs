use daemonize::Error as DaemonizeError;
use log::ParseLevelError;
use std::{any::Any, io};
use yara::errors;

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("`{0}`")]
    IO(#[from] io::Error),
    #[error("`{0}`")]
    YaraIO(#[from] yara::errors::IoError),
    #[error("`{0}`")]
    Daemonize(#[from] DaemonizeError),
    #[error("Daemon does not running. please start daemon first.")]
    DaemonNotRunning,
    #[error("Permission denied in {0}")]
    NoPermission(String),
    #[error("Config file not found: {0}")]
    ConfigNotFound(String),
    #[error("Config file permission is not suitable: {0}")]
    ConfigPermissionDenied(String),
    #[error("Config file entries are missing: `{0}`")]
    ConfigLack(&'static str),
    #[error("Config file parse error: `{reason}`")]
    ConfigParseError {
        reason: String,
    },
    #[error("Yara rule comile error")]
    CompileError(errors::CompileErrors),
    #[error("Yara error: `{0}`")]
    Yara(errors::YaraError),
    #[error("Thread internal error: `{error:?}`")]
    ThreadError {
        error: Box<dyn Any + Send + 'static>,
    },
    #[error("User name error: `{reason}`")]
    UserNameError {
        reason: String,
    },
    #[error("Log level parse error")]
    ParseLogLevelError,
    #[error(transparent)]
    AnyHow(#[from] anyhow::Error),
    #[error("Polling failed")]
    PollingFailed,
    #[error("OS Error: `{0}`")]
    OSError(#[from] nix::errno::Errno),
    #[error("capability check error: `{0}`")]
    CapsError(#[from] caps::errors::CapsError),
    #[error("thread error: `{0}`")]
    ThreadJoinError(#[from] tokio::task::JoinError),

}

pub type Result<T> = core::result::Result<T, Error>;

impl From<username::Error> for Error {
    fn from(err: username::Error) -> Self {
        match err {
            username::Error::IO(e) => Self::from(e),
            username::Error::Var(e) => Self::UserNameError {
                reason: format!("{}", e),
            },
        }
    }
}

impl From<serde_yaml::Error> for Error {
    fn from(err: serde_yaml::Error) -> Self {
        Self::ConfigParseError {
            reason: err.to_string(),
        }
    }
}

impl From<yara::errors::YaraError> for Error {
    fn from(err: yara::errors::YaraError) -> Self {
        Self::Yara(err)
    }
}

impl From<yara::errors::Error> for Error {
    fn from(err: yara::errors::Error) -> Self {
        match err {
            yara::errors::Error::Io(e) => Self::YaraIO(e),
            yara::errors::Error::Yara(e) => Self::Yara(e),
            yara::errors::Error::Compile(e) => Self::CompileError(e),
        }
    }
}

impl From<ParseLevelError> for Error {
    fn from(_: ParseLevelError) -> Self {
        Self::ParseLogLevelError
    }
}

impl From<Box<dyn Any + Send + 'static>> for Error {
    fn from(error: Box<dyn Any + Send + 'static>) -> Self {
        Self::ThreadError { error }
    }
}
