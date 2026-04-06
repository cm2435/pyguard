mod helpers;
use helpers::count_rule;
use pyguard::{lint_source_with_config, Config};

#[test]
fn default_all_active() {
    let source = "hasattr(obj, \"x\")";
    let d = lint_source_with_config(source, "<test>", &Config::default());
    assert!(count_rule(&d, "no-hasattr-getattr") >= 1);
}

#[test]
fn exclude_rule() {
    let source = "hasattr(obj, \"x\")";
    let config = Config { exclude: vec!["no-hasattr-getattr".to_string()] };
    let d = lint_source_with_config(source, "<test>", &config);
    assert_eq!(count_rule(&d, "no-hasattr-getattr"), 0);
}

#[test]
fn exclude_unrelated_rule() {
    let source = "hasattr(obj, \"x\")";
    let config = Config { exclude: vec!["no-print".to_string()] };
    let d = lint_source_with_config(source, "<test>", &config);
    assert!(count_rule(&d, "no-hasattr-getattr") >= 1);
}

#[test]
fn exclude_print() {
    let source = "print(\"hi\")";
    let config = Config { exclude: vec!["no-print".to_string()] };
    let d = lint_source_with_config(source, "<test>", &config);
    assert_eq!(count_rule(&d, "no-print"), 0);
}

#[test]
fn empty_exclude() {
    let source = "print(\"hi\")";
    let config = Config { exclude: vec![] };
    let d = lint_source_with_config(source, "<test>", &config);
    assert!(count_rule(&d, "no-print") >= 1);
}
