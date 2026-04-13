use crate::diagnostic::Diagnostic;
use crate::rules::{Rule, Severity};

pub struct NoOrEmptyCoalesce;

impl Rule for NoOrEmptyCoalesce {
    fn name(&self) -> &'static str {
        "no-or-empty-coalesce"
    }

    fn severity(&self) -> Severity {
        Severity::Warning
    }

    fn help(&self) -> &'static str {
        "`obj.attr or {}` / `or []` / `or \"\"` etc. silently replaces \
         *any* falsy value (including legitimate empty containers, zero, \
         or empty strings) with a new default. If the attribute can be \
         `None`, check explicitly with `if obj.attr is None`; if it \
         cannot be `None`, the `or` fallback is dead code and should be \
         removed."
    }

    fn node_kinds(&self) -> &'static [&'static str] {
        &["boolean_operator"]
    }

    fn check(
        &self,
        node: &tree_sitter::Node,
        source: &[u8],
        _ancestors: &[tree_sitter::Node],
        diagnostics: &mut Vec<Diagnostic>,
    ) {
        let Some(op_node) = node.child_by_field_name("operator") else {
            return;
        };
        if op_node.utf8_text(source).unwrap_or("") != "or" {
            return;
        }

        let Some(right) = node.child_by_field_name("right") else {
            return;
        };
        if !is_empty_falsy_literal(&right, source) {
            return;
        }

        let Some(left) = node.child_by_field_name("left") else {
            return;
        };
        if !is_attribute_access(&left) {
            return;
        }

        let rhs_text = right.utf8_text(source).unwrap_or("<empty>");
        diagnostics.push(Diagnostic {
            path: String::new(),
            line: node.start_position().row + 1,
            col: node.start_position().column,
            rule_id: "no-or-empty-coalesce",
            severity: Severity::Error,
            message: format!(
                "Avoid `... or {rhs_text}` — if the attribute can be None, \
                 check explicitly with `is None`; otherwise remove the fallback"
            ),
        });
    }
}

fn is_attribute_access(node: &tree_sitter::Node) -> bool {
    node.kind() == "attribute"
}

fn is_empty_falsy_literal(node: &tree_sitter::Node, source: &[u8]) -> bool {
    match node.kind() {
        "dictionary" => is_empty_container(node),
        "list" => is_empty_container(node),
        "tuple" => is_empty_container(node),
        "string" => is_empty_string(node),
        "integer" => node.utf8_text(source).unwrap_or("") == "0",
        "float" => node.utf8_text(source).unwrap_or("") == "0.0",
        "false" => true,
        "concatenated_string" => is_empty_concat_string(node),
        "call" => is_empty_set_or_frozenset_call(node, source),
        _ => false,
    }
}

fn is_empty_container(node: &tree_sitter::Node) -> bool {
    for i in 0..node.child_count() {
        let Some(child) = node.child(i) else { continue };
        let k = child.kind();
        if k != "{" && k != "}" && k != "[" && k != "]" && k != "(" && k != ")" {
            return false;
        }
    }
    true
}

fn is_empty_string(node: &tree_sitter::Node) -> bool {
    if node.kind() != "string" {
        return false;
    }
    for i in 0..node.child_count() {
        let Some(child) = node.child(i) else { continue };
        if child.kind() == "string_content" {
            return false;
        }
    }
    true
}

fn is_empty_concat_string(node: &tree_sitter::Node) -> bool {
    for i in 0..node.child_count() {
        let Some(child) = node.child(i) else { continue };
        if child.kind() == "string" && !is_empty_string(&child) {
            return false;
        }
    }
    true
}

fn is_empty_set_or_frozenset_call(node: &tree_sitter::Node, source: &[u8]) -> bool {
    let Some(func) = node.child_by_field_name("function") else {
        return false;
    };
    if func.kind() != "identifier" {
        return false;
    }
    let name = func.utf8_text(source).unwrap_or("");
    if name != "set" && name != "frozenset" {
        return false;
    }

    let Some(args) = node.child_by_field_name("arguments") else {
        return false;
    };
    for i in 0..args.child_count() {
        let Some(child) = args.child(i) else { continue };
        let k = child.kind();
        if k != "(" && k != ")" {
            return false;
        }
    }
    true
}
