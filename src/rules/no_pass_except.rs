use crate::diagnostic::Diagnostic;
use crate::rules::Rule;

pub struct NoPassExcept;

impl Rule for NoPassExcept {
    fn name(&self) -> &'static str {
        "no-pass-except"
    }

    fn help(&self) -> &'static str {
        "An `except` block with only `pass` silently swallows errors, hiding \
         bugs. At minimum log the exception with `logger.exception(...)`. If \
         the exception truly should be ignored (e.g. optional cleanup), add a \
         comment explaining why and suppress with \
         `# slopcop: ignore[no-pass-except]`."
    }

    fn node_kinds(&self) -> &'static [&'static str] {
        &["except_clause"]
    }

    fn check(
        &self,
        node: &tree_sitter::Node,
        _source: &[u8],
        _ancestors: &[tree_sitter::Node],
        diagnostics: &mut Vec<Diagnostic>,
    ) {
        // Find the block child of the except clause
        let mut block = None;
        for i in 0..node.child_count() {
            let child = node.child(i).unwrap();
            if child.kind() == "block" {
                block = Some(child);
                break;
            }
        }

        let Some(block) = block else { return };

        // Check if the block contains only a single `pass_statement`
        let mut statement_count = 0;
        let mut only_pass = true;

        for i in 0..block.child_count() {
            let child = block.child(i).unwrap();
            if child.is_named() {
                statement_count += 1;
                if child.kind() != "pass_statement" {
                    only_pass = false;
                }
            }
        }

        if statement_count == 1 && only_pass {
            diagnostics.push(Diagnostic {
                path: String::new(),
                line: node.start_position().row + 1,
                col: node.start_position().column,
                rule_id: "no-pass-except",
            severity: crate::rules::Severity::Error,
                message: "`except` block contains only `pass`; silently swallowing exceptions hides bugs".to_string(),
            });
        }
    }
}
