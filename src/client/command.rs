use clap::Parser;
use tia::Tia;
use std::path::Path;

#[derive(Debug, Parser, Tia)]
#[clap(author, version, about, long_about=None)]
#[tia(rg)]
pub struct Command {
    /// Be verbose
    #[clap(short, long)]
    verbose: bool,
    /// Save scan report in FILE
    #[clap(short, long)]
    log: Option<String>,
    /// Scan files from FILE
    #[clap(short, long)]
    file_list: Option<String>,
    /// The path to the file to scan
    file: String,
}
