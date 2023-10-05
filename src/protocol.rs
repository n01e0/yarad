use std::convert::{TryFrom, From};
use std::string::ToString;
use crate::error::*;
use crate::client::args;
use log::info;

#[derive(Debug)]
pub enum Command {
    /// Check the daemon's state. It should reply with "PONG\0".
    Ping,
    /// Check the daemon's version.
    Version,
    /// Reload the daemon's configuration and rules.
    Reload,
    /// Shutdown the daemon.
    Shutdown,
    /// Scan the file or directory at the given path (recursively).
    Scan(String),
    /// Scan the file or directory at the given path (recursively) and don't stop the scanning
    /// when a malware found.
    ContScan(String),
    /// Scan the file or directory at the given path (recursively) using multi thread.
    MultiScan(String),
    /// Scan the file inside stream.
    InstreamScan(String),
}

impl ToString for Command {
    fn to_string(&self) -> String {
        match self {
            Command::Ping => "zPING\0".into(),
            Command::Version => "zVERSION\0".into(),
            Command::Reload => "zRELOAD\0".into(),
            Command::Shutdown => "zSHUTDOWN\0".into(),
            Command::Scan(s) => format!("zSCAN {}\0", s),
            Command::ContScan(s) => format!("zCONTSCAN {}\0", s),
            Command::MultiScan(s) => format!("zMULTISCAN {}\0", s),
            Command::InstreamScan(s) => format!("zINSTREAM {}\0", s)
        }
    }
}

impl From<&args::Command> for Vec<Command> {
    fn from(cmd: &args::Command) -> Self {
        match cmd {
            args::Command::Ping => vec![Command::Ping],
            args::Command::Version => vec![Command::Version],
            args::Command::Reload => vec![Command::Reload],
            args::Command::Shutdown => vec![Command::Shutdown],
            args::Command::Scan{path} => path.iter().map(|p| Command::Scan(p.to_string())).collect(),
            args::Command::ContScan{path} => path.iter().map(|p| Command::ContScan(p.to_string())).collect(),
            args::Command::MultiScan{path} => path.iter().map(|p| Command::MultiScan(p.to_string())).collect(),
            args::Command::InstreamScan{path} => path.iter().map(|p| Command::InstreamScan(p.to_string())).collect(),
        }
    }
}

impl TryFrom<&str> for Command {
    type Error = Error;

    fn try_from(s: &str) -> Result<Self> {

        info!("Parsing command: {}", s);
        match s {
            "PING" => Ok(Command::Ping),
            "VERSION" => Ok(Command::Version),
            "RELOAD" => Ok(Command::Reload),
            "SHUTDOWN" => Ok(Command::Shutdown),
            other => {
                if let Some(path) = other.strip_prefix("SCAN ") {
                    let path = path.trim();
                    if path.is_empty() {
                        Err(Error::InvalidCommand(s.to_string()))
                    } else {
                        Ok(Command::Scan(path.to_string()))
                    }
                } else if let Some(path) = other.strip_prefix("CONTSCAN ") {
                    let path = path.trim();
                    if path.is_empty() {
                        Err(Error::InvalidCommand(s.to_string()))
                    } else {
                        Ok(Command::ContScan(path.to_string()))
                    }
                } else if let Some(path) = other.strip_prefix("MULTISCAN ") {
                    let path = path.trim();
                    if path.is_empty() {
                        Err(Error::InvalidCommand(s.to_string()))
                    } else {
                        Ok(Command::MultiScan(path.to_string()))
                    }
                } else if let Some(path) = other.strip_prefix("INSTREAM ") {
                    let path = path.trim();
                    if path.is_empty() {
                        Err(Error::InvalidCommand(s.to_string()))
                    } else {
                        Ok(Command::InstreamScan(path.to_string()))
                    }
                } else {
                    Err(Error::InvalidCommand(s.to_string()))
                }
            }
        }
    }
}

pub fn parse_commands(data: Vec<u8>) -> Result<Vec<Result<Command>>> {
    let data = String::from_utf8(data)?;

    if data.is_empty() {
        return Ok(Vec::new())
    }

    let delim_type = data.chars().next().unwrap();
    info!("Delimiter type: {}", delim_type);
    let delimiter = match delim_type {
        'z' => Ok('\0'),
        'n' => Ok('\n'),
        _ => Err(Error::InvalidCommand(format!("Invalid delimiter specification: {}", delim_type))),
    }?;

    let commands = data
        .split(delimiter)
        .filter(|s| s.starts_with(delim_type))
        .map(|s| Command::try_from(&s[1..]))
        .collect::<Vec<_>>();

    Ok(commands)
}
