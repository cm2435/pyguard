use crate::diagnostic::Diagnostic;
use crate::rules::Rule;

pub struct NoAssert;

impl Rule for NoAssert {
    fn name(&self) -> &'static str {
        "no-assert"
    }

    fn node_kinds(&self) -> &'static [&'static str] {
        &["assert_statement"]
    }

    fn check(
        &self,
        node: &tree_sitter::Node,
        _source: &[u8],
        _ancestors: &[tree_sitter::Node],
        diagnostics: &mut Vec<Diagnostic>,
    ) {
        diagnostics.push(Diagnostic {
            path: String::new(),
            line: node.start_position().row + 1,
            col: node.start_position().column,
            rule_id: "no-assert",
            message: "Avoid `assert` in production code; use `if not ...: raise ValueError(...)` instead".to_string(),
        });
    }
}

/// Check if a file path looks like a test file.
pub fn is_test_file(path: &str) -> bool {
    let filename = path.rsplit('/').next().unwrap_or(path);
    let filename = filename.rsplit('\\').next().unwrap_or(filename);

    if filename.starts_with("test_") || filename.ends_with("_test.py") {
        return true;
    }

    // Check if any path component is "tests", "test", or "testing"
    for component in path.split(['/', '\\']) {
        if matches!(component, "tests" | "test" | "testing") {
            return true;
        }
    }

    false
}
