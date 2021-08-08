use walkdir::WalkDir;
use yara::{Compiler, Rules, Scanner};

use crate::config::Config;
use crate::error::*;

pub fn compile_rules(conf: &Config) -> Result<Rules> {
    let rule_files = WalkDir::new(&conf.rules_dir)
        .into_iter()
        .filter_map(|f| f.ok())
        .filter(|f| f.file_type().is_file())
        .collect::<Vec<_>>();

    let mut compiler = Compiler::new()?;
    for rule_file in rule_files {
        compiler.add_rules_file(rule_file.path())?;
    }

    Ok(compiler.compile_rules()?)
}

