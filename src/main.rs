#[macro_use]
extern crate clap;

mod daemon;
mod error;
mod config;

use std::convert::TryFrom;

fn main() -> error::Result<()> {
    let yml = load_yaml!("cmd.yml");
    let args = clap::App::from_yaml(yml).get_matches();
    let config = config::Config::try_from(&args)?;
    daemon::daemonize()?;
    Ok(())
}
