use std::fs;
use std::io::prelude::*;
use std::os::unix::net::{UnixListener, UnixStream};
use std::path::Path;
use std::thread;

use crate::config::Config;
use crate::error::*;

pub fn handler(mut stream: UnixStream) -> Result<()> {
    let mut buf = [0; 1024];

    let n = stream.read(&mut buf);
    Ok(())
}
