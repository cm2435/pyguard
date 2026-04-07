use std::collections::HashMap;

use crate::config::Config;
use crate::diagnostic::Diagnostic;
use crate::rules::{self, Rule, Severity};
use crate::rules::no_assert;
use crate::suppression;

/// Lint source code with all rules and default config.
pub fn lint_source(source: &str, path: &str) -> Vec<Diagnostic> {
    lint_source_with_config(source, path, &Config::default())
}

/// Lint source code with all rules, applying config exclusions, per-file ignores,
/// Python version filtering, and inline suppression.
pub fn lint_source_with_config(source: &str, path: &str, config: &Config) -> Vec<Diagnostic> {
    let all = rules::all_rules_with_config(config);

    let per_file_excludes = config.excludes_for_path(path);

    let mut active: Vec<&dyn Rule> = all
        .iter()
        .filter(|r| !per_file_excludes.contains(r.name()))
        .map(|r| r.as_ref())
        .collect();

    // Auto-disable no-future-annotations if Python version < 3.13
    if config.min_python_version.is_some_and(|v| v < (3, 13)) {
        active.retain(|r| r.name() != "no-future-annotations");
    }

    lint_source_with_rules(source, path, &active)
}

/// Lint source code with a specific set of rules (low-level API for tests).
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

    // Build a dispatch map AND a severity map from the active rules
    let dispatch_map = build_dispatch_map(rules);
    let severity_map: HashMap<&str, Severity> = rules
        .iter()
        .map(|r| (r.name(), r.severity()))
        .collect();

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
                // Set path and severity on all diagnostics
                for d in &mut diagnostics {
                    d.path = path.to_string();
                    if let Some(&sev) = severity_map.get(d.rule_id) {
                        d.severity = sev;
                    }
                }

                let mut diagnostics = suppression::filter_suppressed(diagnostics, source);

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
