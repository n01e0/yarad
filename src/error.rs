use daemonize::DaemonizeError;
use std::{io, fmt};

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
    }
}

pub type Result<T> = std::result::Result<T, Error>;

impl From<io::Error> for Error {
    fn from(err: io::Error) -> Self {
        Self::IO {
            reason: err.to_string(),
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

impl fmt::Display for Error {
    fn fmt(&self, f: fmt::Formatter) -> fmt::Result {
        use Error::*;
        match self {
            IO(e) => write!(f, "{}", e.reason),
            Daemonize(e) => write!(f, "{}", e.reason),
            DaemonNotRunning => write!(f, "The yarad daemon may not be running"),
            NoPermission => write!(f, "Permission denied"),
            ConfigNotFound => write!(f, "Config file does not found"),
            ConfigLack(s) => write!(f, "Wrong config {}", s),
            ConfigParseError(e) => write!(f, "Config file parse error by {}", e.reason),
        }
    }
}
