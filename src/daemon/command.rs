use clap::Parser;
use tia::Tia;
use crate::config::DEFAULT_CONFIG_PATH;

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
    /// check config
    #[clap(long)]
    check_config: bool,
    /// show rules directory path
    #[clap(long = "show_rules_dir")]
    show_rules_dir: bool,
    /// show rules name
    #[clap(long = "show_rules_name")]
    show_rules_name: bool,
    /// start daemon
    #[clap(long)]
    start: bool,
}

impl Args {
    pub fn config_path(&self) -> String {
        match self.config {
            Some(ref path) => path.clone(),
            None => DEFAULT_CONFIG_PATH.to_string(),
        }
    }
}
