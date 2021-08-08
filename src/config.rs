use crate::error;
use log::Level;
use serde::Deserialize;
use std::convert::TryFrom;
use std::fs::read_to_string;
use std::str::FromStr;

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
}

#[derive(Debug)]
pub struct Config {
    pub log_level: String,
    pub local_socket: String,
    pub local_socket_group: String,
    pub local_socket_mode: u32,
    pub rules_dir: String,
    pub working_dir: String,
    pub user: String,
    pub auto_recompile_rules: bool,
}

impl<'a> TryFrom<&clap::ArgMatches<'a>> for Config {
    type Error = error::Error;
    fn try_from(args: &clap::ArgMatches) -> error::Result<Self> {
        let path = args.value_of("config_file").unwrap();
        let config: ConfigFile = serde_yaml::from_str(&read_to_string(path)?)?;

        let log_level = Level::from_str(&config.log_level.unwrap_or("warn".into()))?.as_str().into();
        let local_socket = config.local_socket.unwrap_or("/var/run/yarad/yarad.ctl".into());
        let local_socket_group = config.local_socket_group.unwrap_or("yarad".into());
        let local_socket_mode: u32 = {
            let mut perm = config.local_socket_mode.unwrap_or("0o666".into());
            if !perm.starts_with("0o") {
                perm = format!("0o{}", perm);
            }
            parse_int::parse::<u32>(&perm).unwrap_or(0o666)
        };
        let rules_dir = config.rules_dir.unwrap_or("/var/lib/yarad/rules".into());
        let working_dir = config.working_dir.unwrap_or("/var/run/yarad".into());
        let user = config.user.unwrap_or("yarad".into());
        let auto_recompile_rules = config.auto_recompile_rules.unwrap_or(true);

        Ok(Config{
            log_level,
            local_socket,
            local_socket_group,
            local_socket_mode,
            rules_dir,
            working_dir,
            user,
            auto_recompile_rules
        })
    }
}
