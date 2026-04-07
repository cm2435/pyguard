mod helpers;
use helpers::lint_with_rule;

#[test]
fn bare_true() {
    let d = lint_with_rule("f(True)", "no-boolean-positional");
    assert_eq!(d.len(), 1);
}

#[test]
fn bare_false() {
    let d = lint_with_rule("f(False)", "no-boolean-positional");
    assert_eq!(d.len(), 1);
}

#[test]
fn multiple_booleans() {
    let d = lint_with_rule("f(True, False, True)", "no-boolean-positional");
    assert_eq!(d.len(), 3);
}

#[test]
fn keyword_arg_ok() {
    let d = lint_with_rule("f(verbose=True)", "no-boolean-positional");
    assert_eq!(d.len(), 0);
}

#[test]
fn mixed_args() {
    let d = lint_with_rule("f(x, True, y=False)", "no-boolean-positional");
    assert_eq!(d.len(), 1);
}

#[test]
fn no_booleans_ok() {
    let d = lint_with_rule("f(1, \"hello\", x)", "no-boolean-positional");
    assert_eq!(d.len(), 0);
}

#[test]
fn method_call() {
    let d = lint_with_rule("obj.method(True)", "no-boolean-positional");
    assert_eq!(d.len(), 1);
}

#[test]
fn bool_in_list_ok() {
    // True inside a list literal, not as a function argument
    let d = lint_with_rule("x = [True, False]", "no-boolean-positional");
    assert_eq!(d.len(), 0);
}

#[test]
fn bool_as_return_ok() {
    let d = lint_with_rule("def f():\n    return True", "no-boolean-positional");
    assert_eq!(d.len(), 0);
}

#[test]
fn bool_in_assert_ok() {
    let d = lint_with_rule("assert True", "no-boolean-positional");
    assert_eq!(d.len(), 0);
}

#[test]
fn in_comment_ok() {
    let d = lint_with_rule("# f(True)", "no-boolean-positional");
    assert_eq!(d.len(), 0);
}

#[test]
fn in_string_ok() {
    let d = lint_with_rule("x = \"f(True)\"", "no-boolean-positional");
    assert_eq!(d.len(), 0);
}

#[test]
fn nested_call() {
    // f(g(True)) -- the True is a positional arg to g(), should flag
    let d = lint_with_rule("f(g(True))", "no-boolean-positional");
    assert_eq!(d.len(), 1);
}
