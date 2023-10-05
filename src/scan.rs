use yara::Rule;

#[derive(Debug)]
pub struct ScanResult {
    pub rule: Vec<String>,
    pub path: String,
}

impl ScanResult {
    pub fn new(rule: Vec<Rule>, path: String) -> Self {
        ScanResult {
            rule: rule.into_iter().map(|x| x.identifier.to_string()).collect(),
            path
        }
    }
}
