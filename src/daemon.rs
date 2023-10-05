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
use crate::config::Config;
use crate::error::*;
use crate::sock::Listener;
use crate::scan::ScanResult;
use crate::protocol::Command;
use tokio::sync::Mutex;
use std::sync::Arc;
use std::os::fd::BorrowedFd;

#[derive(Tia)]
#[tia(rg)]
pub struct Yarad {
    config: Config,
    rules: Arc<Mutex<Rules>>,
}

impl Yarad {
    pub fn new(config: Config) -> Result<Self> {
        let rules_dir = config.get_rules_dir().to_string();
        Ok(Self {
            config,
            rules: Arc::new(Mutex::new(compile_rules(&rules_dir)?)),
        })
    }

    async fn scan(&self, path: String) -> Result<Vec<ScanResult>> {
        let mut results = Vec::new();
        let rules = self.rules.lock().await;
        
        let target = Path::new(&path);
        if target.is_dir() {
            for entry in WalkDir::new(path).into_iter().filter_map(|e| e.ok()) {
                if entry.file_type().is_file() {
                    let matches = rules.scan_file(entry.path(), *self.config.get_scan_timeout())?;
                    results.push(ScanResult::new(matches, format!("{}", entry.path().display())));
                }
            }
        } else if target.is_file() {
            let matches = rules.scan_file(&path, *self.config.get_scan_timeout())?;
            if matches.is_empty() {
                results.push(ScanResult{rule: vec!["OK".to_string()], path});
            } else {
                results.push(ScanResult::new(matches, path));
            }
        } else {
            return Err(Error::InvalidPath(path));
        }

        Ok(results)
    }

    pub async fn run(self) -> Result<()> {
        let config = Arc::new(Mutex::new(self.config.clone()));
        info!("yarad started");

        let listener = Listener::new(&*config.lock().await).await?;

        info!("starting main loop");
        let main_loop: Result<()> = tokio::spawn(async move {
            loop {
                let stream = listener.accept().await?;
                stream.readable().await?;
                let commands = stream.try_parse_commands()?;
                info!("received {} commands", commands.len());
                for command in commands {
                    if let Err(Error::InvalidCommand(e)) = command {
                        let message = format!("Invalid command: {}", e);
                        warn!("Received {}", message);
                        stream.try_write(message.as_bytes())?;
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
                        Command::Reload => {
                            info!("recompiling rules");
                            let config = config.lock().await;
                            let rules_dir = config.get_rules_dir();
                            let new_rules = compile_rules(rules_dir)?;
                            let mut locked_rules = self.rules.lock().await;
                            *locked_rules = new_rules;
                            info!("recompilation done");

                        }
                        Command::Scan(path) => {
                            info!("Received scan request for {}", path);
                            match self.scan(path).await {
                                Ok(results) => {
                                    for result in results {
                                        for rule in result.rule {
                                            info!("{}: {}", rule, result.path);
                                            let message = format!("{}: {}\n", rule, result.path);
                                            stream.try_write(message.as_bytes())?;
                                        }
                                    }
                                },
                                Err(e) => {
                                    error!("Error while scanning: {}", e);
                                    stream.try_write(format!("Error while scanning: {}\n", e).as_bytes())?;
                                }

                            }
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

fn compile_rules(rules_dir: &str) -> Result<Rules> {
    let rule_files = WalkDir::new(rules_dir)
        .into_iter()
        .filter_map(|f| f.ok())
        .filter(|f| f.path().extension().unwrap_or_default() == "yar" || f.path().extension().unwrap_or_default() == "yara")
        .filter(|f| f.file_type().is_file())
        .collect::<Vec<_>>();

    let compiler = rule_files.into_iter().try_fold(Compiler::new()?, |compiler, f| {
        compiler.add_rules_file(f.path())
    })?;


    Ok(compiler.compile_rules()?)
}
