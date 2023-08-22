
use std::io::prelude::*;
use std::os::unix::net::{UnixStream};





use anyhow::Result;

pub fn handler(mut stream: UnixStream) -> Result<()> {
    let mut buf = [0; 1024];

    let _n = stream.read(&mut buf);
    Ok(())
}
