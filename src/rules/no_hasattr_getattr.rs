use crate::diagnostic::Diagnostic;
use crate::rules::Rule;

const BANNED: &[&str] = &["hasattr", "getattr"];

pub struct NoHasattrGetattr;

impl Rule for NoHasattrGetattr {
    fn name(&self) -> &'static str {
        "no-hasattr-getattr"
    }

    fn help(&self) -> &'static str {
        "`hasattr()` and `getattr()` hide AttributeError and bypass type \
         checkers. Replace with an explicit `try`/`except AttributeError`, a \
         Protocol that declares the expected interface, or a direct attribute \
         access. If the call is genuinely dynamic (plugin dispatch, \
         deserialization), suppress with \
         `# slopcop: ignore[no-hasattr-getattr]`."
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

        let fn_name = match func.kind() {
            "identifier" => {
                let text = func.utf8_text(source).unwrap_or("");
                if BANNED.contains(&text) {
                    text
                } else {
                    return;
                }
            }
            "attribute" => {
                let Some(attr) = func.child_by_field_name("attribute") else {
                    return;
                };
                let text = attr.utf8_text(source).unwrap_or("");
                if BANNED.contains(&text) {
                    text
                } else {
                    return;
                }
            }
            _ => return,
        };

        diagnostics.push(Diagnostic {
            path: String::new(),
            line: node.start_position().row + 1,
            col: node.start_position().column,
            rule_id: "no-hasattr-getattr",
            severity: crate::rules::Severity::Error,
            message: format!(
                "Avoid `{fn_name}()`; use explicit attribute checks or protocols"
            ),
        });
    }
}
