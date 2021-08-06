#[macro_use]
extern crate clap;

mod daemon;
mod error;

fn main() -> error::Result<()> {
    println!("Hello, world!");
    daemon::daemonize()?;
    Ok(())
}
