#![allow(dead_code)]
use pyguard::{Diagnostic, lint_source_with_rules};
use pyguard::rules::{self, Rule};

pub fn count_rule(diagnostics: &[Diagnostic], rule_id: &str) -> usize {
    diagnostics.iter().filter(|d| d.rule_id == rule_id).count()
}

pub fn lint_with_rule(source: &str, rule_name: &str) -> Vec<Diagnostic> {
    let all = rules::all_rules();
    let rule: Vec<&dyn Rule> = all
        .iter()
        .filter(|r| r.name() == rule_name)
        .map(|r| r.as_ref())
        .collect();
    lint_source_with_rules(source, "<test>", &rule)
}
