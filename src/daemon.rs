pub mod command;
pub mod rule;

use daemonize::{Daemonize, User};
use log::{info, warn, error};
use nix::unistd::geteuid;
use std::fs::{create_dir, OpenOptions};
use std::path::Path;
use tia::Tia;
use yara::{Rules, Compiler};
use walkdir::WalkDir;
#[cfg(target_os = "linux")]
use nix::poll::{poll, PollFd, PollFlags};
#[cfg(target_os = "linux")]
use fanotify::high_level::{Fanotify, FanEvent, FanotifyMode};
use crate::config::Config;
use crate::error::*;
use crate::sock::{Listener, Stream};
use crate::protocol::Command;
use std::sync::{Arc, Mutex};

#[derive(Tia)]
#[tia(rg)]
pub struct Yarad {
    config: Config,
}

impl Yarad {
    pub fn new(config: Config) -> Result<Self> {
        Ok(Self { config })
    }

    pub async fn run(self) -> Result<()> {
        info!("initial compiling rules");
        let rules = Arc::new(Mutex::new(compile_rules(&self.config)?));
        info!("rules compiled");
        info!("yarad started");

        let listener = Listener::new(&self.config).await?;
        let stream = listener.accept().await?;

        #[cfg(target_os = "linux")]
        if *self.get_config().get_auto_recompile_rules() && cap_check().is_ok() {
            info!("auto recompile enabled");
            let rules_dir = self.config.get_rules_dir();
            let fan = Fanotify::new_with_nonblocking(FanotifyMode::CONTENT);
            fan.add_path(FanEvent::CloseWrite.into(), rules_dir)?;
            let realtime_rules = Arc::clone(&rules);

            let auto_recompile_thread: Result<()> = tokio::spawn(async move {
                let mut fds = [PollFd::new(fan.as_raw_fd(), PollFlags::POLLIN)];
                info!("starting recopile thread");
                loop {
                    let poll_num = poll(&mut fds, -1)?;
                    if poll_num > 0 {
                        let event = fan.read_event();
                        if event.iter().any(|e| e.events.contains(&FanEvent::CloseWrite)) {
                            info!("recompiling rules");
                            let new_rules = compile_rules(&self.config)?;
                            let mut locked_rules = realtime_rules.lock().map_err(|e| Error::ThreadError{ error: Box::new(format!("Lock failed: {:?}", e))})?;
                            *locked_rules = new_rules;
                            info!("recompilation done");
                        }
                    } else {
                        error!("polling failed");
                        return Err(Error::PollingFailed)
                    }
                }
            }).await?;

            auto_recompile_thread?;
        }

        let main_loop: Result<()> = tokio::spawn(async move {
            loop {
                stream.readable().await?;
                let commands = stream.try_parse_commands()?;
                for command in commands {
                    if let Err(Error::InvalidCommand(e)) = command {
                        let message = format!("Invalid command: {}", e);
                        warn!("Received {}", message);
                        stream.try_write(&message.as_bytes())?;
                        continue;
                    }
                    match command? {
                        Command::Ping => {
                            info!("Received ping");
                            stream.try_write(b"PONG")?;
                        },
                        Command::Version => {
                            info!("Received version");
                            stream.try_write(b"yarad 0.1.0")?;
                        },
                        _ => Err(Error::InvalidCommand("Invalid command".to_string()))?,
                    }
                }
            }
        }).await?;

        main_loop?;
        Ok(())
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
}

fn cap_check() -> Result<()> {
    #[cfg(target_os = "linux")]
    {
        if !caps::has_cap(
            None,
            caps::CapSet::Permitted,
            caps::Capability::CAP_SYS_ADMIN
        )? {
            return Err(Error::NoPermission("auto recompile needs CAP_SYS_ADMIN".to_string()))
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
