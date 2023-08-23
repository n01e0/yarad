use std::convert::TryFrom;
use crate::error::*;

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

impl TryFrom<&str> for Command {
    type Error = Error;

    fn try_from(s: &str) -> Result<Self> {

        match s {
            "PING" => Ok(Command::Ping),
            "VERSION" => Ok(Command::Version),
            "RELOAD" => Ok(Command::Reload),
            "SHUTDOWN" => Ok(Command::Shutdown),
            other => {
                if other.starts_with("SCAN ") {
                    let path = other[5..].trim();
                    if path.is_empty() {
                        Err(Error::InvalidCommand(s.to_string()))
                    } else {
                        Ok(Command::Scan(path.to_string()))
                    }
                } else if other.starts_with("CONTSCAN ") {
                    let path = other[8..].trim();
                    if path.is_empty() {
                        Err(Error::InvalidCommand(s.to_string()))
                    } else {
                        Ok(Command::ContScan(path.to_string()))
                    }
                } else if other.starts_with("MULTISCAN ") {
                    let path = other[9..].trim();
                    if path.is_empty() {
                        Err(Error::InvalidCommand(s.to_string()))
                    } else {
                        Ok(Command::MultiScan(path.to_string()))
                    }
                } else if other.starts_with("INSTREAM ") {
                    let path = other[8..].trim();
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

    let delim_type = data.chars().nth(0).unwrap();
    let delimiter = match delim_type {
        'z' => Ok('\0'),
        'n' => Ok('\n'),
        _ => Err(Error::InvalidCommand(format!("Invalid delimiter specification: {}", delim_type))),
    }?;

    let commands = data
        .split(delimiter)
        .filter(|s| s.starts_with(delimiter))
        .map(|s| Command::try_from(&s[1..]))
        .collect::<Vec<_>>();

    Ok(commands)
}
