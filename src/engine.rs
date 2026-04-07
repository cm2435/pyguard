use std::collections::HashMap;

use crate::config::Config;
use crate::diagnostic::Diagnostic;
use crate::rules::{self, Rule};
use crate::rules::no_assert;
use crate::suppression;

/// Lint source code with all rules and default config.
pub fn lint_source(source: &str, path: &str) -> Vec<Diagnostic> {
    lint_source_with_config(source, path, &Config::default())
}

/// Lint source code with all rules, applying config exclusions and inline suppression.
pub fn lint_source_with_config(source: &str, path: &str, config: &Config) -> Vec<Diagnostic> {
    let all = rules::all_rules();
    let active: Vec<&dyn Rule> = all
        .iter()
        .filter(|r| !config.exclude.iter().any(|e| e == r.name()))
        .map(|r| r.as_ref())
        .collect();
    lint_source_with_rules(source, path, &active)
}

/// Lint source code with a specific set of rules.
pub fn lint_source_with_rules(source: &str, path: &str, rules: &[&dyn Rule]) -> Vec<Diagnostic> {
    let source_bytes = source.as_bytes();

    let mut parser = tree_sitter::Parser::new();
    parser
        .set_language(&tree_sitter_python::LANGUAGE.into())
        .expect("failed to load Python grammar");

    let tree = match parser.parse(source_bytes, None) {
        Some(t) => t,
        None => return Vec::new(),
    };

    let dispatch_map = build_dispatch_map(rules);

    let mut diagnostics = Vec::new();
    let mut ancestors: Vec<tree_sitter::Node> = Vec::new();
    let mut cursor = tree.walk();

    loop {
        let node = cursor.node();

        if let Some(matching_rules) = dispatch_map.get(node.kind()) {
            for rule in matching_rules {
                rule.check(&node, source_bytes, &ancestors, &mut diagnostics);
            }
        }

        if cursor.goto_first_child() {
            ancestors.push(node);
            continue;
        }

        loop {
            if cursor.goto_next_sibling() {
                break;
            }
            if !cursor.goto_parent() {
                for d in &mut diagnostics {
                    d.path = path.to_string();
                }
                let mut diagnostics = suppression::filter_suppressed(diagnostics, source);

                // Skip no-assert in test files
                if no_assert::is_test_file(path) {
                    diagnostics.retain(|d| d.rule_id != "no-assert");
                }

                return diagnostics;
            }
            ancestors.pop();
        }
    }
}

fn build_dispatch_map<'a>(rules: &[&'a dyn Rule]) -> HashMap<&'static str, Vec<&'a dyn Rule>> {
    let mut map: HashMap<&'static str, Vec<&'a dyn Rule>> = HashMap::new();
    for rule in rules {
        for kind in rule.node_kinds() {
            map.entry(kind).or_default().push(*rule);
        }
    }
    map
}
