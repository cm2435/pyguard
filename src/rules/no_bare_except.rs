use crate::diagnostic::Diagnostic;
use crate::rules::Rule;

pub struct NoBareExcept;

impl Rule for NoBareExcept {
    fn name(&self) -> &'static str {
        "no-bare-except"
    }

    fn node_kinds(&self) -> &'static [&'static str] {
        &["except_clause"]
    }

    fn check(
        &self,
        node: &tree_sitter::Node,
        source: &[u8],
        _ancestors: &[tree_sitter::Node],
        diagnostics: &mut Vec<Diagnostic>,
    ) {
        let exception_type = find_exception_type(node, source);

        match exception_type {
            None => {
                diagnostics.push(Diagnostic {
                    path: String::new(),
                    line: node.start_position().row + 1,
                    col: node.start_position().column,
                    rule_id: "no-bare-except",
                    message: "Bare `except:` catches all exceptions including KeyboardInterrupt; specify an exception type".to_string(),
                });
            }
            Some(type_text) if type_text == "Exception" || type_text == "BaseException" => {
                diagnostics.push(Diagnostic {
                    path: String::new(),
                    line: node.start_position().row + 1,
                    col: node.start_position().column,
                    rule_id: "no-bare-except",
                    message: format!(
                        "`except {type_text}` is too broad; catch specific exception types"
                    ),
                });
            }
            Some(_) => {}
        }
    }
}

fn find_exception_type<'a>(node: &tree_sitter::Node, source: &'a [u8]) -> Option<&'a str> {
    let mut found_except_keyword = false;
    for i in 0..node.child_count() {
        let child = node.child(i).unwrap();
        let kind = child.kind();
        let text = child.utf8_text(source).unwrap_or("");

        if text == "except" {
            found_except_keyword = true;
            continue;
        }

        if !found_except_keyword {
            continue;
        }

        if kind == ":" || kind == "block" {
            return None;
        }

        if matches!(
            kind,
            "identifier" | "attribute" | "tuple" | "parenthesized_expression"
        ) {
            if text == "as" {
                continue;
            }
            return Some(text);
        }

        if kind == "as_pattern" {
            if let Some(type_node) = child.child(0) {
                return Some(type_node.utf8_text(source).unwrap_or(""));
            }
        }
    }
    None
}
