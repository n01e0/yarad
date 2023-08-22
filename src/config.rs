use crate::error::*;
use clap::Parser;
use tia::Tia;
use log::Level;
use serde::Deserialize;
use std::fs::read_to_string;
use std::str::FromStr;

pub const DEFAULT_CONFIG_PATH: &str = "/etc/yarad/config.yml";


#[derive(Debug, Parser, Tia)]
#[clap(author, version, about, long_about=None)]
#[tia(rg)]
pub struct Args {
    /// config file
    #[clap(short, long)]
    config: String,
    /// without daemonize
    #[clap(short, long, default_value_t = false, action)]
    pub(self) no_daemonize: bool,
}

#[derive(Debug, Deserialize)]
struct ConfigFile {
    log_level: Option<String>,
    local_socket: Option<String>,
    local_socket_group: Option<String>,
    local_socket_mode: Option<String>,
    rules_dir: Option<String>,
    working_dir: Option<String>,
    user: Option<String>,
    auto_recompile_rules: Option<bool>,
    pid_file: Option<String>,
}

#[derive(Debug, Tia, Eq, PartialEq)]
#[tia(rg)]
pub struct Config {
    log_level: String,
    local_socket: String,
    local_socket_group: String,
    local_socket_mode: u32,
    rules_dir: String,
    working_dir: String,
    user: String,
    auto_recompile_rules: bool,
    pid_file: String,
}

impl std::convert::TryFrom<String> for Config {
    type Error = Error;
    fn try_from(path: String) -> Result<Self> {
        let content = read_to_string(&path)
            .map_err(|e| match e.kind() {
                std::io::ErrorKind::NotFound => Error::ConfigNotFound(path),
                std::io::ErrorKind::PermissionDenied => Error::ConfigPermissionDenied(path),
                _ => e.into()
            })?;
        let config_file: ConfigFile = serde_yaml::from_str(&content)?;

        config_file.convert()
    }
}

impl ConfigFile {
    fn convert(self) -> Result<Config> {
        let log_level = Level::from_str(&self.log_level.unwrap_or("warn".into()))?
            .as_str()
            .into();
        let local_socket = self 
            .local_socket
            .unwrap_or("/var/run/yarad/yarad.ctl".into());
        let local_socket_group = self.local_socket_group.unwrap_or("yarad".into());
        let local_socket_mode: u32 = {
            let mut perm = self.local_socket_mode.unwrap_or("0o666".into());
            if !perm.starts_with("0o") {
                perm = format!("0o{}", perm);
            }
            parse_int::parse::<u32>(&perm).unwrap_or(0o666)
        };
        let rules_dir = self.rules_dir.unwrap_or("/var/lib/yarad/rules".into());
        let working_dir = self.working_dir.unwrap_or("/var/run/yarad".into());
        let user = self.user.unwrap_or("yarad".into());
        let auto_recompile_rules = self.auto_recompile_rules.unwrap_or(true);
        let pid_file = self.pid_file.unwrap_or("/var/run/yarad/yarad.pid".into());

        Ok(Config {
            log_level,
            local_socket,
            local_socket_group,
            local_socket_mode,
            rules_dir,
            working_dir,
            user,
            auto_recompile_rules,
            pid_file
        })
    }
}
