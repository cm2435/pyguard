use crate::diagnostic::Diagnostic;
use crate::rules::{Rule, Severity};

pub struct NoBooleanPositional;

impl Rule for NoBooleanPositional {
    fn name(&self) -> &'static str {
        "no-boolean-positional"
    }

    fn severity(&self) -> Severity { Severity::Warning }

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
        let Some(args) = node.child_by_field_name("arguments") else {
            return;
        };

        for i in 0..args.child_count() {
            let arg = args.child(i).unwrap();

            // Skip keyword arguments -- those are explicit
            if arg.kind() == "keyword_argument" {
                continue;
            }

            // Check for bare True/False as positional arguments
            if (arg.kind() == "true" || arg.kind() == "false")
                || (arg.kind() == "identifier"
                    && matches!(
                        arg.utf8_text(source).unwrap_or(""),
                        "True" | "False"
                    ))
            {
                diagnostics.push(Diagnostic {
                    path: String::new(),
                    line: arg.start_position().row + 1,
                    col: arg.start_position().column,
                    rule_id: "no-boolean-positional",
            severity: crate::rules::Severity::Error,
                    message: "Avoid bare boolean positional arguments; use keyword arguments for clarity".to_string(),
                });
            }
        }
    }
}
