use clap::Parser;
use std::convert::TryFrom;
use yarad::config;
use yarad::daemon::{self, command::{Args, Command}};
use yarad::error::*;
use yarad::scan;
use anyhow::{anyhow, Context};

fn main() -> Result<()> {
    let args = Args::parse();
    let config_path = args.config_path();
    let config = config::Config::try_from(config_path)?;

    if let Some(command) = args.get_command() {
        match command {
            Command::Start => {
                if !*args.get_foreground() && *config.get_daemonize() {
                    daemon::daemonize(&config).map_err(|e| Error::from(e))?;
                }
            },
            Command::Reload => {
                println!("{:#?}", config);
            },
            _ => {
                return Err(anyhow!("Not implemented").into());
            }
        }
        return Ok(());
    }
    //let rules = scan::compile_rules(&config).with_context(|| "Can't compile rules")?;
    //let _scanner = rules.scanner()?;
    Ok(())
}
