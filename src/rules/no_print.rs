use crate::diagnostic::Diagnostic;
use crate::rules::Rule;

pub struct NoPrint;

impl Rule for NoPrint {
    fn name(&self) -> &'static str {
        "no-print"
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

        let is_print = match func.kind() {
            "identifier" => func.utf8_text(source).unwrap_or("") == "print",
            "attribute" => {
                if let Some(attr) = func.child_by_field_name("attribute") {
                    attr.utf8_text(source).unwrap_or("") == "print"
                } else {
                    false
                }
            }
            _ => false,
        };

        if !is_print {
            return;
        }

        diagnostics.push(Diagnostic {
            path: String::new(),
            line: node.start_position().row + 1,
            col: node.start_position().column,
            rule_id: "no-print",
            message: "Avoid `print()`; use structured logging (structlog, logging, etc.)"
                .to_string(),
        });
    }
}
