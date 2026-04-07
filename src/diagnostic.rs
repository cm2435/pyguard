use std::fmt;

use crate::rules::Severity;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Diagnostic {
    pub path: String,
    pub line: usize,
    pub col: usize,
    pub rule_id: &'static str,
    pub severity: Severity,
    pub message: String,
}

impl Diagnostic {
    /// Create a diagnostic with default Error severity (engine overrides from Rule trait).
    pub fn new(rule_id: &'static str, line: usize, col: usize, message: String) -> Self {
        Self {
            path: String::new(),
            line,
            col,
            rule_id,
            severity: Severity::Error,
            message,
        }
    }
}

impl fmt::Display for Diagnostic {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let sev = match self.severity {
            Severity::Error => "error",
            Severity::Warning => "warning",
        };
        write!(
            f,
            "{}:{}:{}  {sev}[{}] {}",
            self.path, self.line, self.col, self.rule_id, self.message
        )
    }
}
