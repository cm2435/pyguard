use std::fmt;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Diagnostic {
    pub path: String,
    pub line: usize,
    pub col: usize,
    pub rule_id: &'static str,
    pub message: String,
}

impl fmt::Display for Diagnostic {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(
            f,
            "{}:{}:{}  [{}] {}",
            self.path, self.line, self.col, self.rule_id, self.message
        )
    }
}
