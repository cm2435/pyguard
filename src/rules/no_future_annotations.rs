use crate::diagnostic::Diagnostic;
use crate::rules::Rule;

pub struct NoFutureAnnotations;

impl Rule for NoFutureAnnotations {
    fn name(&self) -> &'static str {
        "no-future-annotations"
    }

    fn node_kinds(&self) -> &'static [&'static str] {
        &["future_import_statement"]
    }

    fn check(
        &self,
        node: &tree_sitter::Node,
        source: &[u8],
        _ancestors: &[tree_sitter::Node],
        diagnostics: &mut Vec<Diagnostic>,
    ) {
        for i in 0..node.child_count() {
            let child = node.child(i).unwrap();
            match child.kind() {
                "dotted_name" => {
                    if child.utf8_text(source).unwrap_or("") == "annotations" {
                        diagnostics.push(make_diagnostic(node));
                        return;
                    }
                }
                "aliased_import" => {
                    if let Some(name) = child.child_by_field_name("name") {
                        if name.utf8_text(source).unwrap_or("") == "annotations" {
                            diagnostics.push(make_diagnostic(node));
                            return;
                        }
                    }
                }
                _ => {}
            }
        }
    }
}

fn make_diagnostic(node: &tree_sitter::Node) -> Diagnostic {
    Diagnostic {
        path: String::new(),
        line: node.start_position().row + 1,
        col: node.start_position().column,
        rule_id: "no-future-annotations",
        message: "Do not use `from __future__ import annotations`; unnecessary on Python 3.13+ and breaks runtime annotation inspection".to_string(),
    }
}
