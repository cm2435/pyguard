mod helpers;
use helpers::count_rule;
use pyguard::lint_source;

#[test]
fn empty_file() {
    let d = lint_source("", "<test>");
    assert_eq!(d.len(), 0);
}

#[test]
fn comments_only() {
    let d = lint_source("# just comments\n# more comments", "<test>");
    assert_eq!(d.len(), 0);
}

#[test]
fn syntax_error_no_panic() {
    let d = lint_source("def f(\n    broken syntax", "<test>");
    let _ = d;
}

#[test]
fn binary_content_no_panic() {
    let d = lint_source("\x00\x01\x02", "<test>");
    let _ = d;
}

#[test]
fn deeply_nested() {
    let mut source = String::new();
    for i in 0..50 {
        let indent = "    ".repeat(i);
        source.push_str(&format!("{}def f{}():\n", indent, i));
    }
    let deep_indent = "    ".repeat(50);
    source.push_str(&format!("{}import os", deep_indent));
    let d = lint_source(&source, "<test>");
    assert!(count_rule(&d, "guarded-function-import") >= 1);
}

#[test]
fn module_level_if_import_ok() {
    let d = lint_source("if True:\n    import os", "<test>");
    assert_eq!(count_rule(&d, "guarded-function-import"), 0);
}

#[test]
fn all_rules_fire_together() {
    let source = r#"from __future__ import annotations
import dataclasses
hasattr(obj, "x")
print("hello")
def f():
    import os
try:
    pass
except:
    pass
"#;
    let d = lint_source(source, "<test>");
    let ids: Vec<&str> = d.iter().map(|d| d.rule_id).collect();
    assert!(ids.contains(&"no-future-annotations"), "missing no-future-annotations");
    assert!(ids.contains(&"no-dataclass"), "missing no-dataclass");
    assert!(ids.contains(&"no-hasattr-getattr"), "missing no-hasattr-getattr");
    assert!(ids.contains(&"no-print"), "missing no-print");
    assert!(ids.contains(&"guarded-function-import"), "missing guarded-function-import");
    assert!(ids.contains(&"no-bare-except"), "missing no-bare-except");
}
