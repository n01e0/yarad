use clap::Parser;
use tia::Tia;

#[derive(Debug, Parser, Tia)]
#[clap(author, version, about, long_about = None)]
#[tia(rg)]
pub struct Args {
    /// config file
    #[clap(short, long)]
    config: Option<String>,
    /// daemon start
    #[clap(short = 'd', long = "daemon")]
    daemonize: bool,
    /// show rules directory path
    #[clap(long = "show_rules_dir")]
    show_rules_dir: bool,
    /// show rules name
    #[clap(long = "show_rules_name")]
    show_rules_name: bool,
}