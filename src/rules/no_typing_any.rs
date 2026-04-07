use crate::diagnostic::Diagnostic;
use crate::rules::{Rule, Severity};

pub struct NoTypingAny;

impl Rule for NoTypingAny {
    fn name(&self) -> &'static str {
        "no-typing-any"
    }

    fn severity(&self) -> Severity { Severity::Warning }

    fn node_kinds(&self) -> &'static [&'static str] {
        &["type"]
    }

    fn check(
        &self,
        node: &tree_sitter::Node,
        source: &[u8],
        _ancestors: &[tree_sitter::Node],
        diagnostics: &mut Vec<Diagnostic>,
    ) {
        find_any_identifiers(node, source, diagnostics);
    }
}

/// Recursively find `identifier` nodes with text "Any" within a type annotation.
/// Skips child `type` nodes since the engine dispatches those independently.
fn find_any_identifiers(
    node: &tree_sitter::Node,
    source: &[u8],
    diagnostics: &mut Vec<Diagnostic>,
) {
    if node.kind() == "identifier" && node.utf8_text(source).unwrap_or("") == "Any" {
        diagnostics.push(Diagnostic {
            path: String::new(),
            line: node.start_position().row + 1,
            col: node.start_position().column,
            rule_id: "no-typing-any",
            severity: crate::rules::Severity::Error,
            message: "Avoid `Any` in type annotations; use specific types or protocols"
                .to_string(),
        });
        return;
    }

    for i in 0..node.child_count() {
        let child = node.child(i).unwrap();
        if child.kind() == "type" {
            continue;
        }
        find_any_identifiers(&child, source, diagnostics);
    }
}
