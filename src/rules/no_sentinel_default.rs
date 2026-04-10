use crate::diagnostic::Diagnostic;
use crate::rules::{Rule, Severity};

pub struct NoSentinelDefault;

impl Rule for NoSentinelDefault {
    fn name(&self) -> &'static str {
        "no-sentinel-default"
    }

    fn severity(&self) -> Severity {
        Severity::Warning
    }

    fn node_kinds(&self) -> &'static [&'static str] {
        &["typed_default_parameter", "assignment"]
    }

    fn check(
        &self,
        node: &tree_sitter::Node,
        source: &[u8],
        ancestors: &[tree_sitter::Node],
        diagnostics: &mut Vec<Diagnostic>,
    ) {
        match node.kind() {
            "typed_default_parameter" => self.check_param(node, source, diagnostics),
            "assignment" => self.check_assignment(node, source, ancestors, diagnostics),
            _ => {}
        }
    }
}

impl NoSentinelDefault {
    fn check_param(
        &self,
        node: &tree_sitter::Node,
        source: &[u8],
        diagnostics: &mut Vec<Diagnostic>,
    ) {
        let Some(value_node) = node.child_by_field_name("value") else {
            return;
        };
        if !is_sentinel_value(&value_node, source) {
            return;
        }
        self.emit(node, diagnostics);
    }

    fn check_assignment(
        &self,
        node: &tree_sitter::Node,
        source: &[u8],
        ancestors: &[tree_sitter::Node],
        diagnostics: &mut Vec<Diagnostic>,
    ) {
        let in_class = ancestors.iter().any(|a| a.kind() == "class_definition");
        if !in_class {
            return;
        }

        let mut has_type = false;
        let mut value_node = None;
        let mut found_eq = false;

        for i in 0..node.child_count() {
            let Some(child) = node.child(i) else { continue };
            match child.kind() {
                "type" if !has_type => {
                    has_type = true;
                }
                "=" => {
                    found_eq = true;
                }
                _ if found_eq && value_node.is_none() => {
                    value_node = Some(child);
                }
                _ => {}
            }
        }

        if !has_type {
            return;
        }
        let Some(value_node) = value_node else { return };
        if !is_sentinel_value(&value_node, source) {
            return;
        }
        self.emit(node, diagnostics);
    }

    fn emit(&self, node: &tree_sitter::Node, diagnostics: &mut Vec<Diagnostic>) {
        diagnostics.push(Diagnostic {
            path: String::new(),
            line: node.start_position().row + 1,
            col: node.start_position().column,
            rule_id: "no-sentinel-default",
            severity: Severity::Error,
            message:
                "Avoid sentinel/placeholder default values; thread the real value or use `None`"
                    .to_string(),
        });
    }
}

fn is_sentinel_value(node: &tree_sitter::Node, source: &[u8]) -> bool {
    match node.kind() {
        "string" => is_sentinel_string(node, source),
        "call" => is_sentinel_call(node, source),
        _ => false,
    }
}

fn extract_string_content<'a>(node: &tree_sitter::Node, source: &'a [u8]) -> &'a str {
    for i in 0..node.child_count() {
        let child = node.child(i).unwrap();
        if child.kind() == "string_content" {
            return child.utf8_text(source).unwrap_or("");
        }
    }
    ""
}

fn is_sentinel_string(node: &tree_sitter::Node, source: &[u8]) -> bool {
    let text = extract_string_content(node, source);
    if text.is_empty() {
        return false;
    }
    is_sentinel_text(text)
}

fn is_sentinel_text(text: &str) -> bool {
    let lower = text.to_ascii_lowercase();

    if is_placeholder_uuid(text) {
        return true;
    }

    if is_placeholder_keyword(&lower) {
        return true;
    }

    if lower.contains("example.com") || lower.contains("example.org") || lower.contains("example.net") {
        return true;
    }

    if lower.starts_with("/path/to/") {
        return true;
    }

    if lower.starts_with("sk-") {
        return true;
    }

    false
}

fn is_placeholder_keyword(lower: &str) -> bool {
    matches!(
        lower,
        "deadbeef"
            | "replace_me"
            | "changeme"
            | "insert_key_here"
            | "your-api-key-here"
            | "your-api-key"
            | "your-key-here"
            | "fake-jwt"
            | "placeholder"
            | "test_string"
            | "foo"
            | "bar"
            | "baz"
            | "qux"
            | "quux"
            | "lorem ipsum"
            | "hello world"
    )
}

fn is_placeholder_uuid(text: &str) -> bool {
    let parts: Vec<&str> = text.split('-').collect();
    if parts.len() != 5 {
        return false;
    }
    if parts[0].len() != 8
        || parts[1].len() != 4
        || parts[2].len() != 4
        || parts[3].len() != 4
        || parts[4].len() != 12
    {
        return false;
    }

    let hex_only: String = parts.concat();
    if hex_only.len() != 32 {
        return false;
    }
    if !hex_only.chars().all(|c| c.is_ascii_hexdigit()) {
        return false;
    }

    if text == "123e4567-e89b-12d3-a456-426614174000" {
        return true;
    }

    if parts[0] == "00000000" && parts[1] == "0000" && parts[2] == "0000" && parts[3] == "0000" {
        return true;
    }

    if parts[0] == "12345678" {
        return true;
    }

    let unique_chars: std::collections::HashSet<char> = hex_only.chars().collect();
    if unique_chars.len() <= 3 {
        return true;
    }

    false
}

fn is_sentinel_call(node: &tree_sitter::Node, source: &[u8]) -> bool {
    let Some(func_node) = node.child_by_field_name("function") else {
        return false;
    };
    if !is_uuid_constructor(&func_node, source) {
        return false;
    }

    let Some(args_node) = node.child_by_field_name("arguments") else {
        return false;
    };

    for i in 0..args_node.child_count() {
        let Some(arg) = args_node.child(i) else {
            continue;
        };
        match arg.kind() {
            "keyword_argument" => {
                let Some(name) = arg.child_by_field_name("name") else {
                    continue;
                };
                let Some(value) = arg.child_by_field_name("value") else {
                    continue;
                };
                if name.utf8_text(source).unwrap_or("") == "int" && value.kind() == "integer" {
                    let int_text = value.utf8_text(source).unwrap_or("");
                    if let Ok(n) = int_text.parse::<u64>() {
                        if n < 10_000 {
                            return true;
                        }
                    }
                }
            }
            "string" => {
                if is_sentinel_string(&arg, source) {
                    return true;
                }
            }
            _ => {}
        }
    }

    false
}

fn is_uuid_constructor(func_node: &tree_sitter::Node, source: &[u8]) -> bool {
    match func_node.kind() {
        "identifier" => func_node.utf8_text(source).unwrap_or("") == "UUID",
        "attribute" => func_node
            .child_by_field_name("attribute")
            .map_or(false, |a| a.utf8_text(source).unwrap_or("") == "UUID"),
        _ => false,
    }
}
