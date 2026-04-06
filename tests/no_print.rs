mod helpers;
use helpers::lint_with_rule;

#[test]
fn bare_print() {
    let d = lint_with_rule("print(\"hello\")", "no-print");
    assert_eq!(d.len(), 1);
    assert_eq!(d[0].line, 1);
}

#[test]
fn print_with_args() {
    let d = lint_with_rule("print(x, y, sep=\",\")", "no-print");
    assert_eq!(d.len(), 1);
}

#[test]
fn builtins_print() {
    let d = lint_with_rule("builtins.print(\"hello\")", "no-print");
    assert_eq!(d.len(), 1);
}

#[test]
fn in_comment() {
    let d = lint_with_rule("# print(\"hello\")", "no-print");
    assert_eq!(d.len(), 0);
}

#[test]
fn in_string() {
    let d = lint_with_rule("x = \"print('hello')\"", "no-print");
    assert_eq!(d.len(), 0);
}

#[test]
fn inside_function() {
    let source = "def f():\n    print(\"debug\")";
    let d = lint_with_rule(source, "no-print");
    assert_eq!(d.len(), 1);
    assert_eq!(d[0].line, 2);
}

#[test]
fn multiple() {
    let source = "print(\"a\")\nprint(\"b\")\nprint(\"c\")";
    let d = lint_with_rule(source, "no-print");
    assert_eq!(d.len(), 3);
}

#[test]
fn logging_ok() {
    let d = lint_with_rule("logging.info(\"hello\")", "no-print");
    assert_eq!(d.len(), 0);
}

#[test]
fn pprint_ok() {
    let d = lint_with_rule("pprint(obj)", "no-print");
    assert_eq!(d.len(), 0);
}
