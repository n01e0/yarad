use walkdir::WalkDir;
use yara::{Compiler, Rules};

use crate::config::Config;
use anyhow::Result;

#[derive(Debug)]
pub struct ScanResult {
    rule: String,
    file: String,
}

