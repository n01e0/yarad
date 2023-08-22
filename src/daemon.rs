pub mod command;
pub mod rule;

use daemonize::{Daemonize, User};
use log::info;
use nix::unistd::geteuid;
use std::fs::{create_dir, OpenOptions};
use std::path::Path;
use anyhow::Result;
use tia::Tia;
use yara::{Rules, Compiler};
use walkdir::WalkDir;
use nix::poll::{poll, PollFd, PollFlags};

use crate::config::Config;
use crate::error::*;

#[derive(Tia)]
#[tia(rg)]
pub struct Yarad {
    config: Config,
    rules: Rules
}

impl Yarad {
    pub fn new(config: Config) -> Result<Self> {
        let rules = compile_rules(&config)?;
        Ok(Self { config, rules })
    }

    pub fn daemonize(&self) -> Result<()> {
        let username = self.get_config().get_user();
        let as_su = geteuid().is_root();
    
        let open_opts = OpenOptions::new()
            .truncate(false)
            .create(true)
            .write(true)
            .to_owned();
    
        let workdir = self.get_config().get_working_dir();
        let working_directory = if Path::new(workdir).exists() {
            workdir
        } else {
            create_dir(workdir)?;
            workdir
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
            .user(User::from(&username[..]))
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

    pub async fn run(&mut self) -> Result<()> {
        #[cfg(target_os = "linux")]
        if *self.get_config().get_auto_recompile_rules() && cap_check().is_ok() {
            let rules_dir = self.config.get_rules_dir();
            let fan = Fanotify::new_with_nonblocking(FanotifyMode::CONTENT);
            fan.add_path(FanEvent::CloseWrite, rules_dir)?;

            tokio::spawn(async move {
                let mut fds = [PollFd::new(fan.as_raw_fd(), PollFlags::POLLIN)];
                loop {
                    let poll_num = poll(&mut fds, -1)?;
                    if poll_num > 0 {
                        let event = fan.read_event()?;
                        if event.events.contains(&FanEvent::CloseWrite) {
                            info!("recompiling rules");
                            let rules = compile_rules(&self.config)?;
                            self.rules = rules;
                        }
                    } else {
                        error!("polling fanotify failed");
                        return Err(Error::PollingFailed);
                    }
                }
            })
        }

        let scanner = self.get_rules();

        Ok(())
    }
}

fn cap_check() -> Result<()> {
    #[cfg(target_os = "linux")]
    {
        if !caps::has_cap(
            None,
            caps::CapSet::Permitted,
            caps::Capability::CAP_SYS_ADMIN
        )? {
            Err(Error::NoPermission("auto recompile needs CAP_SYS_ADMIN".to_string()));
        }
    }
    Ok(())
}

fn compile_rules(conf: &Config) -> Result<Rules> {
    let rule_files = WalkDir::new(conf.get_rules_dir())
        .into_iter()
        .filter_map(|f| f.ok())
        .filter(|f| f.file_type().is_file())
        .collect::<Vec<_>>();

    let compiler = rule_files.into_iter().try_fold(Compiler::new()?, |compiler, f| {
        compiler.add_rules_file(f.path())
    })?;


    Ok(compiler.compile_rules()?)
}
