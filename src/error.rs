use daemonize::DaemonizeError;
use std::{io, fmt, any::Any};
use yara::errors;
use log::ParseLevelError;

#[derive(Debug)]
pub enum Error {
    IO {
        reason: String,
    },
    Daemonize {
        reason: String, 
    },
    DaemonNotRunning,
    NoPermission,
    ConfigNotFound,
    ConfigLack(&'static str),
    ConfigParseError {
        reason: String,
    },
    CompileError(errors::CompileErrors),
    Yara(errors::YaraError),
    ThreadError { error: Box<dyn Any + Send + 'static> },
    UserNameError { reason: String },
    ParseLogLevelError,
}

pub type Result<T> = std::result::Result<T, Error>;

impl From<io::Error> for Error {
    fn from(err: io::Error) -> Self {
        Self::IO {
            reason: err.to_string(),
        }
    }
}

impl From<username::Error> for Error {
    fn from(err: username::Error) -> Self {
        match err {
            username::Error::IO(e) => Error::from(e),
            username::Error::Var(e) => Self::UserNameError { reason: format!("{}", e) },
        }
    }
}

impl From<DaemonizeError> for Error {
    fn from(err: DaemonizeError) -> Self {
        Self::Daemonize {
            reason: err.to_string(),
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
            yara::errors::Error::Io(e) => Self::IO { reason: format!("{}", e), },
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

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        use Error::*;
        match self {
            IO{ reason } => write!(f, "{}", reason),
            Daemonize{ reason } => write!(f, "{}", reason),
            DaemonNotRunning => write!(f, "The yarad daemon may not be running"),
            NoPermission => write!(f, "Permission denied"),
            ConfigNotFound => write!(f, "Config file does not found"),
            ConfigLack(s) => write!(f, "Wrong config {}", s),
            ConfigParseError{ reason } => write!(f, "Config file parse error by {}", reason),
            CompileError(e) => write!(f, "Yara rules compiling error by {}", e),
            Yara(e) => write!(f, "Yara error by {}", e),
            ThreadError{ error: _ } => write!(f, "Error in thread"),
            UserNameError{ reason } => write!(f, "Can't get username by {}", reason),
            ParseLogLevelError => write!(f, "Can't parse log_level"),
        }
    }
}
