use crate::diagnostic::Diagnostic;
use crate::rules::{Rule, Severity};

pub struct NoPrint;

impl Rule for NoPrint {
    fn name(&self) -> &'static str {
        "no-print"
    }

    fn severity(&self) -> Severity { Severity::Warning }

    fn help(&self) -> &'static str {
        "Use `logging.getLogger(__name__)` or `structlog.get_logger()` instead \
         of `print()`. Structured logging supports log levels, context fields, \
         and downstream filtering. If this is a CLI entrypoint or user-facing \
         script where `print()` is intentional, add the path glob to \
         `[tool.slopcop.per-file-ignores]` in `pyproject.toml`."
    }

    fn node_kinds(&self) -> &'static [&'static str] {
        &["call"]
    }

    fn check(
        &self,
        node: &tree_sitter::Node,
        source: &[u8],
        _ancestors: &[tree_sitter::Node],
        diagnostics: &mut Vec<Diagnostic>,
    ) {
        let Some(func) = node.child_by_field_name("function") else {
            return;
        };

        // Only flag bare `print(...)`, not `obj.print(...)` which is a method call
        if func.kind() != "identifier" || func.utf8_text(source).unwrap_or("") != "print" {
            return;
        }

        diagnostics.push(Diagnostic {
            path: String::new(),
            line: node.start_position().row + 1,
            col: node.start_position().column,
            rule_id: "no-print",
            severity: crate::rules::Severity::Error,
            message: "Avoid `print()`; use structured logging (structlog, logging, etc.)"
                .to_string(),
        });
    }
}
