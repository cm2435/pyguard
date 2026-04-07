use crate::diagnostic::Diagnostic;
use crate::rules::Rule;

pub struct NoTypingAny;

impl Rule for NoTypingAny {
    fn name(&self) -> &'static str {
        "no-typing-any"
    }

    fn node_kinds(&self) -> &'static [&'static str] {
        &["type", "import_from_statement"]
    }

    fn check(
        &self,
        node: &tree_sitter::Node,
        source: &[u8],
        _ancestors: &[tree_sitter::Node],
        diagnostics: &mut Vec<Diagnostic>,
    ) {
        match node.kind() {
            "type" => self.check_type_annotation(node, source, diagnostics),
            "import_from_statement" => self.check_import(node, source, diagnostics),
            _ => {}
        }
    }
}

impl NoTypingAny {
    /// Flag `Any` identifiers inside type annotations.
    fn check_type_annotation(
        &self,
        node: &tree_sitter::Node,
        source: &[u8],
        diagnostics: &mut Vec<Diagnostic>,
    ) {
        find_any_identifiers(node, source, diagnostics);
    }

    /// Flag `from typing import Any` and `from typing_extensions import Any`.
    fn check_import(
        &self,
        node: &tree_sitter::Node,
        source: &[u8],
        diagnostics: &mut Vec<Diagnostic>,
    ) {
        let Some(module) = node.child_by_field_name("module_name") else {
            return;
        };
        let module_text = module.utf8_text(source).unwrap_or("");
        if module_text != "typing" && module_text != "typing_extensions" {
            return;
        }

        for i in 0..node.child_count() {
            let child = node.child(i).unwrap();
            match child.kind() {
                "dotted_name" => {
                    if child.utf8_text(source).unwrap_or("") == "Any" {
                        diagnostics.push(Diagnostic {
                            path: String::new(),
                            line: node.start_position().row + 1,
                            col: node.start_position().column,
                            rule_id: "no-typing-any",
                            message: "Avoid importing `Any`; use specific types or protocols"
                                .to_string(),
                        });
                        return;
                    }
                }
                "aliased_import" => {
                    if let Some(name) = child.child_by_field_name("name") {
                        if name.utf8_text(source).unwrap_or("") == "Any" {
                            diagnostics.push(Diagnostic {
                                path: String::new(),
                                line: node.start_position().row + 1,
                                col: node.start_position().column,
                                rule_id: "no-typing-any",
                                message:
                                    "Avoid importing `Any`; use specific types or protocols"
                                        .to_string(),
                            });
                            return;
                        }
                    }
                }
                "import_list" => {
                    for j in 0..child.child_count() {
                        let item = child.child(j).unwrap();
                        let name_text = match item.kind() {
                            "dotted_name" => item.utf8_text(source).unwrap_or(""),
                            "aliased_import" => item
                                .child_by_field_name("name")
                                .and_then(|n| n.utf8_text(source).ok())
                                .unwrap_or(""),
                            _ => continue,
                        };
                        if name_text == "Any" {
                            diagnostics.push(Diagnostic {
                                path: String::new(),
                                line: node.start_position().row + 1,
                                col: node.start_position().column,
                                rule_id: "no-typing-any",
                                message:
                                    "Avoid importing `Any`; use specific types or protocols"
                                        .to_string(),
                            });
                            return;
                        }
                    }
                }
                _ => {}
            }
        }
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
            message: "Avoid `Any` in type annotations; use specific types or protocols"
                .to_string(),
        });
        return;
    }

    for i in 0..node.child_count() {
        let child = node.child(i).unwrap();
        if child.kind() == "type" {
            continue; // engine dispatches these separately
        }
        find_any_identifiers(&child, source, diagnostics);
    }
}
