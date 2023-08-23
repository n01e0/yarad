#[derive(Debug)]
pub struct ScanResult {
    rule: String,
    file: String,
}

impl ScanResult {
    pub fn new<S: Into<String>>(rule: S, file: S) -> Self {
        ScanResult {
            rule: rule.into(),
            file: file.into()
        }
    }
}
