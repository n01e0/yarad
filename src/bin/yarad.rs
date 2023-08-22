use clap::Parser;
use std::convert::TryFrom;
use yarad::config;
use yarad::daemon::{self, command::Args};
use yarad::error::*;
use yarad::scan;


const DEFAULT_CONFIG_PATH: &str = "/etc/yarad/config.yml";

fn main() -> Result<()> {
    let command = Args::parse();
    let config_path = command.config_path();
    let config = config::Config::try_from(config_path)?;
    if !*command.get_foreground() {
        daemon::daemonize(&config).map_err(|e| Error::from(e))?;
    }
    let rules = scan::compile_rules(&config)?;
    let _scanner = rules.scanner()?;
    Ok(())
}
