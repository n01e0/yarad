use daemonize::{User, Daemonize};
use log::info;
use std::fs::{create_dir, OpenOptions};
use std::io::{self, Write};
use std::path::Path;
use nix::unistd::geteuid;

use crate::error::Result;

pub fn daemonize() -> Result<()> {
    let as_su = geteuid().is_root();
    let username = if as_su {
        "root".into()
    } else {
        username::get_user_name().unwrap_or("nobody".into())
    };

    let open_opts = OpenOptions::new()
        .truncate(false)
        .create(true)
        .write(true)
        .to_owned();

    let workind_directory = if !Path::new("/var/run/yarad").exists() {
        "/var/run/yarad"
    } else {
        create_dir("/var/run/yarad")?;
        "/var/run/yarad"
    };

    let (stdout, stderr, pid_file) = if as_su {
        (
            Some(open_opts.open("/var/log/yarad.out")?),
            Some(open_opts.open("/var/log/yarad.err")?),
            "/var/run/yarad/yarad.pid",
        )
    } else {
        (
            None,
            None,
            "/var/run/yarad/yarad.pid",
        )
    };

    let daemonize = Daemonize::new()
        .user(User::Name(username.clone()))
        .pid_file(pid_file)
        .chown_pid_file(false)
        .working_directory(workind_directory);

    if stdout.is_some() {
        daemonize.stdout(stdout.unwrap())
            .stderr(stderr.unwrap())
            .start()?;
    } else {
        daemonize.start()?;
    }

    info!("yarad started by {}", username);
    Ok(())
}
