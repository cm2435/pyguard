use crate::diagnostic::Diagnostic;
use crate::rules::{Rule, Severity};

pub struct NoDataclass;

impl Rule for NoDataclass {
    fn name(&self) -> &'static str {
        "no-dataclass"
    }

    fn severity(&self) -> Severity { Severity::Warning }

    fn node_kinds(&self) -> &'static [&'static str] {
        &["decorator", "import_statement", "import_from_statement"]
    }

    fn check(
        &self,
        node: &tree_sitter::Node,
        source: &[u8],
        _ancestors: &[tree_sitter::Node],
        diagnostics: &mut Vec<Diagnostic>,
    ) {
        match node.kind() {
            "decorator" => self.check_decorator(node, source, diagnostics),
            "import_statement" => self.check_import(node, source, diagnostics),
            "import_from_statement" => self.check_from_import(node, source, diagnostics),
            _ => {}
        }
    }
}

impl NoDataclass {
    fn check_decorator(
        &self,
        node: &tree_sitter::Node,
        source: &[u8],
        diagnostics: &mut Vec<Diagnostic>,
    ) {
        // decorator has children: "@" and then the expression
        // We need to find the expression child (not the "@" token)
        for i in 0..node.child_count() {
            let Some(child) = node.child(i) else { continue };
            if is_dataclass_expr(&child, source) {
                diagnostics.push(make_diagnostic(node));
                return;
            }
        }
    }

    fn check_import(
        &self,
        node: &tree_sitter::Node,
        source: &[u8],
        diagnostics: &mut Vec<Diagnostic>,
    ) {
        // `import dataclasses`
        for i in 0..node.child_count() {
            let Some(child) = node.child(i) else { continue };
            if child.kind() == "dotted_name"
                && child.utf8_text(source).unwrap_or("") == "dataclasses"
            {
                diagnostics.push(make_diagnostic(node));
                return;
            }
            if child.kind() == "aliased_import" {
                if let Some(name) = child.child_by_field_name("name") {
                    if name.utf8_text(source).unwrap_or("") == "dataclasses" {
                        diagnostics.push(make_diagnostic(node));
                        return;
                    }
                }
            }
        }
    }

    fn check_from_import(
        &self,
        node: &tree_sitter::Node,
        source: &[u8],
        diagnostics: &mut Vec<Diagnostic>,
    ) {
        // `from dataclasses import ...`
        let Some(module) = node.child_by_field_name("module_name") else {
            return;
        };
        if module.utf8_text(source).unwrap_or("") == "dataclasses" {
            diagnostics.push(make_diagnostic(node));
        }
    }
}

fn is_dataclass_expr(node: &tree_sitter::Node, source: &[u8]) -> bool {
    match node.kind() {
        "identifier" => node.utf8_text(source).unwrap_or("") == "dataclass",
        "attribute" => {
            if let Some(attr) = node.child_by_field_name("attribute") {
                attr.utf8_text(source).unwrap_or("") == "dataclass"
            } else {
                false
            }
        }
        "call" => {
            if let Some(func) = node.child_by_field_name("function") {
                is_dataclass_expr(&func, source)
            } else {
                false
            }
        }
        _ => false,
    }
}

fn make_diagnostic(node: &tree_sitter::Node) -> Diagnostic {
    Diagnostic {
        path: String::new(),
        line: node.start_position().row + 1,
        col: node.start_position().column,
        rule_id: "no-dataclass",
            severity: crate::rules::Severity::Error,
        message: "Avoid `dataclass`; use Pydantic `BaseModel` or project-standard model base"
            .to_string(),
    }
}
