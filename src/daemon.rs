use daemonize::{Daemonize, User};
use log::info;
use nix::unistd::geteuid;
use std::fs::{create_dir, OpenOptions};
use std::path::Path;

use crate::config::Config;
use crate::error::{Error, Result};

pub fn daemonize(conf: &Config) -> Result<()> {
    let username = &conf.user;
    let as_su = geteuid().is_root();

    if &username[..] == "root" && !as_su {
        return Err(Error::NoPermission);
    }

    let open_opts = OpenOptions::new()
        .truncate(false)
        .create(true)
        .write(true)
        .to_owned();

    let working_directory = if Path::new(&conf.working_dir).exists() {
        &conf.working_dir
    } else {
        create_dir(&conf.working_dir)?;
        &conf.working_dir
    };

    let (stdout, stderr, pid_file) = if as_su {
        (
            Some(open_opts.open("/var/log/yarad.out")?),
            Some(open_opts.open("/var/log/yarad.log")?),
            "/var/run/yarad/yarad.pid",
        )
    } else {
        (None, None, "yarad.pid")
    };

    open_opts.open(pid_file)?;

    let daemonize = Daemonize::new()
        .user(User::Name(username.clone()))
        .pid_file(pid_file)
        .chown_pid_file(false)
        .working_directory(working_directory);

    if stdout.is_some() {
        daemonize
            .stdout(stdout.unwrap())
            .stderr(stderr.unwrap())
            .start()?;
    } else {
        daemonize.start()?;
    }

    info!("yarad started by {}", username);
    Ok(())
}
