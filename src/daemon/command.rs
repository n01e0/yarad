use clap::{Parser, ValueEnum};
use tia::Tia;
use crate::config::DEFAULT_CONFIG_PATH;

#[derive(Debug, ValueEnum, Clone)]
pub enum Command {
    /// start daemon
    #[clap(name = "start")]
    Start,
    /// show daemon pid
    #[clap(name = "pid")]
    Pid,
    /// reload config
    #[clap(name = "reload")]
    Reload,
    /// show rules directory path
    #[clap(name = "show-rules-dir")]
    ShowRulesDir,
}

#[derive(Debug, Parser, Tia)]
#[clap(author, version, about, long_about = None)]
#[tia(rg)]
pub struct Args {
    /// config file
    #[clap(short, long)]
    config: Option<String>,
    /// daemon start
    #[clap(long)]
    foreground: bool,
    /// command
    #[clap(value_enum)]
    command: Option<Command>,
}

impl Args {
    pub fn config_path(&self) -> String {
        match self.config {
            Some(ref path) => path.clone(),
            None => DEFAULT_CONFIG_PATH.to_string(),
        }
    }
}
