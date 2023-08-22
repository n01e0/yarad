use walkdir::WalkDir;
use yara::{Compiler, Rules};

use crate::config::Config;
use anyhow::Result;

#[derive(Debug)]
pub struct ScanResult {
    rule: String,
    file: String,
}

pub fn compile_rules(conf: &Config) -> Result<Rules> {
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
