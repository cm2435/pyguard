mod helpers;
use helpers::lint_with_rule;

#[test]
fn basic_assert() {
    let d = lint_with_rule("assert x > 0", "no-assert");
    assert_eq!(d.len(), 1);
    assert_eq!(d[0].line, 1);
}

#[test]
fn assert_with_message() {
    let d = lint_with_rule("assert x > 0, \"x must be positive\"", "no-assert");
    assert_eq!(d.len(), 1);
}

#[test]
fn assert_in_function() {
    let source = "def f(x):\n    assert x is not None";
    let d = lint_with_rule(source, "no-assert");
    assert_eq!(d.len(), 1);
    assert_eq!(d[0].line, 2);
}

#[test]
fn multiple_asserts() {
    let source = "assert a\nassert b\nassert c";
    let d = lint_with_rule(source, "no-assert");
    assert_eq!(d.len(), 3);
}

#[test]
fn assert_in_comment_ok() {
    let d = lint_with_rule("# assert x > 0", "no-assert");
    assert_eq!(d.len(), 0);
}

#[test]
fn assert_in_string_ok() {
    let d = lint_with_rule("x = \"assert x > 0\"", "no-assert");
    assert_eq!(d.len(), 0);
}

#[test]
fn no_assert_present() {
    let d = lint_with_rule("if not x:\n    raise ValueError(\"bad\")", "no-assert");
    assert_eq!(d.len(), 0);
}
