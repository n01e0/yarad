use clap::{Parser, Subcommand};
use tia::Tia;

#[derive(Debug, Parser, Tia)]
#[clap(author, version, about, long_about=None)]
#[tia(rg)]
pub struct Args {
    /// Save scan report in FILE
    #[clap(short, long)]
    report: Option<String>,
    /// Command
    #[clap(subcommand)]
    command: Command,
}

#[derive(Debug, Subcommand)]
pub enum Command {
    /// ping to daemon
    Ping,
    /// daemon version
    Version,
    /// reload daemon
    Reload,
    /// shutdown daemon
    Shutdown,
    /// scan
    Scan {
        #[arg(required = true)]
        path: Vec<String>
    },
    /// Scan the file or directory at the given path (recursively) and don't stop the scanning
    /// when a malware found.
    ContScan {
        #[arg(required = true)]
        path: Vec<String>
    },
    /// Scan the file or directory at the given path (recursively) using multi thread.
    MultiScan{
        #[arg(required = true)]
        path: Vec<String>
    },
    /// Scan the file inside stream.
    InstreamScan{
        #[arg(required = true)]
        path: Vec<String>
    },
}
