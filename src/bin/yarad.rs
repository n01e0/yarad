use anyhow::anyhow;
use clap::Parser;
use std::convert::TryFrom;
use yarad::config;
use yarad::daemon::{
    command::{Args, Command},
    Yarad,
};
use yarad::error::*;
use std::fs::read_to_string;
use log::{info, error};

#[tokio::main]
async fn main() -> Result<()> {
    let args = Args::parse();
    let config_path = args.config_path();
    let config = config::Config::try_from(config_path)?;
    env_logger::init();

    if let Some(command) = args.get_command() {
        match command {
            Command::Start => {
                info!("Starting yarad");
                let daemon = Yarad::new(config)?;
                if !*args.get_foreground() {
                    info!("Starting daemon");
                    daemon.daemonize()?;
                }
                loop {
                    if let Err(e) = daemon.run().await {
                        error!("Error: {}", e);
                    }
                }
            }
            Command::ShowRulesDir => {
                println!("{}", config.get_rules_dir());
            }
            Command::Pid => {
                let pid_file = config.get_pid_file();
                println!("{}", read_to_string(pid_file)?);
            }
            _ => {
                return Err(anyhow!("Not implemented").into());
            }
        }
        return Ok(());
    }
    Ok(())
}
