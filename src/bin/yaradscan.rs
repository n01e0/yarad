use yarad::{
    error::*,
    protocol::Command,
    client::args::Args,
};
use clap::Parser;
use std::os::unix::net::UnixStream;
use std::io::prelude::*;

fn main() -> Result<()> {
    let args = Args::parse();

    let command = Vec::<Command>::from(args.get_command());

    let mut stream = UnixStream::connect("/var/run/yarad/yarad.ctl")?;
    stream.write_all(command.into_iter().map(|c| c.to_string()).collect::<String>().as_bytes())?;

    let mut resp = String::new();
    stream.read_to_string(&mut resp)?;
    print!("{}", resp);
    Ok(())
}
