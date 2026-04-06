mod helpers;
use helpers::lint_with_rule;

#[test]
fn bare_hasattr() {
    let d = lint_with_rule(r#"hasattr(obj, "x")"#, "no-hasattr-getattr");
    assert_eq!(d.len(), 1);
    assert_eq!(d[0].line, 1);
    assert_eq!(d[0].rule_id, "no-hasattr-getattr");
}

#[test]
fn bare_getattr() {
    let d = lint_with_rule(r#"getattr(obj, "x")"#, "no-hasattr-getattr");
    assert_eq!(d.len(), 1);
}

#[test]
fn getattr_three_args() {
    let d = lint_with_rule(r#"getattr(obj, "x", None)"#, "no-hasattr-getattr");
    assert_eq!(d.len(), 1);
}

#[test]
fn builtins_hasattr() {
    let d = lint_with_rule(r#"builtins.hasattr(obj, "x")"#, "no-hasattr-getattr");
    assert_eq!(d.len(), 1);
}

#[test]
fn builtins_getattr() {
    let d = lint_with_rule(r#"builtins.getattr(obj, "x")"#, "no-hasattr-getattr");
    assert_eq!(d.len(), 1);
}

#[test]
fn in_comment() {
    let d = lint_with_rule("# hasattr(obj, \"x\")", "no-hasattr-getattr");
    assert_eq!(d.len(), 0);
}

#[test]
fn in_string() {
    let d = lint_with_rule("x = \"hasattr(obj, 'x')\"", "no-hasattr-getattr");
    assert_eq!(d.len(), 0);
}

#[test]
fn in_multiline_string() {
    let d = lint_with_rule("x = '''hasattr(obj, 'x')'''", "no-hasattr-getattr");
    assert_eq!(d.len(), 0);
}

#[test]
fn method_call() {
    let d = lint_with_rule("obj.hasattr()", "no-hasattr-getattr");
    assert_eq!(d.len(), 1);
}

#[test]
fn in_expression() {
    let d = lint_with_rule("result = getattr(obj, key) if cond else default", "no-hasattr-getattr");
    assert_eq!(d.len(), 1);
}

#[test]
fn multiple_in_file() {
    let source = "hasattr(a, \"x\")\ngetattr(b, \"y\")\nhasattr(c, \"z\")";
    let d = lint_with_rule(source, "no-hasattr-getattr");
    assert_eq!(d.len(), 3);
}

#[test]
fn inside_function() {
    let source = "def f():\n    hasattr(self, \"x\")";
    let d = lint_with_rule(source, "no-hasattr-getattr");
    assert_eq!(d.len(), 1);
    assert_eq!(d[0].line, 2);
}

#[test]
fn inside_lambda() {
    let source = "f = lambda: hasattr(x, \"y\")";
    let d = lint_with_rule(source, "no-hasattr-getattr");
    assert_eq!(d.len(), 1);
}
