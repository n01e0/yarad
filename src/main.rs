#[macro_use]
extern crate clap;

mod config;
mod daemon;
mod error;
mod scan;
mod sock;

use std::convert::TryFrom;

fn main() -> error::Result<()> {
    let yml = load_yaml!("cmd.yml");
    let args = clap::App::from_yaml(yml).get_matches();
    let config = config::Config::try_from(&args)?;
    daemon::daemonize(&config)?;
    let rules = scan::compile_rules(&config)?;
    let scanner = rules.scanner()?;
    Ok(())
}
