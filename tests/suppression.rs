mod helpers;
use helpers::count_rule;
use slopcop::lint_source;

#[test]
fn blanket() {
    let source = "hasattr(obj, \"x\")  # slopcop: ignore";
    let d = lint_source(source, "<test>");
    assert_eq!(count_rule(&d, "no-hasattr-getattr"), 0);
}

#[test]
fn targeted_match() {
    let source = "hasattr(obj, \"x\")  # slopcop: ignore[no-hasattr-getattr]";
    let d = lint_source(source, "<test>");
    assert_eq!(count_rule(&d, "no-hasattr-getattr"), 0);
}

#[test]
fn targeted_mismatch() {
    let source = "hasattr(obj, \"x\")  # slopcop: ignore[no-print]";
    let d = lint_source(source, "<test>");
    assert!(count_rule(&d, "no-hasattr-getattr") >= 1);
}

#[test]
fn multi_target() {
    let source = "hasattr(obj, \"x\")  # slopcop: ignore[no-hasattr-getattr, no-print]";
    let d = lint_source(source, "<test>");
    assert_eq!(count_rule(&d, "no-hasattr-getattr"), 0);
}

#[test]
fn print() {
    let source = "print(\"hi\")  # slopcop: ignore";
    let d = lint_source(source, "<test>");
    assert_eq!(count_rule(&d, "no-print"), 0);
}

#[test]
fn wrong_comment() {
    let source = "hasattr(obj, \"x\")  # some other comment";
    let d = lint_source(source, "<test>");
    assert!(count_rule(&d, "no-hasattr-getattr") >= 1);
}

#[test]
fn on_previous_line_does_not_work() {
    let source = "# slopcop: ignore\nhasattr(obj, \"x\")";
    let d = lint_source(source, "<test>");
    assert!(count_rule(&d, "no-hasattr-getattr") >= 1);
}

#[test]
fn function_import() {
    let source = "def f():\n    import os  # slopcop: ignore";
    let d = lint_source(source, "<test>");
    assert_eq!(count_rule(&d, "guarded-function-import"), 0);
}
