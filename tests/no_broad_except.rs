mod helpers;
use helpers::lint_with_rule;

#[test]
fn except_exception() {
    let source = "try:\n    pass\nexcept Exception:\n    pass";
    let d = lint_with_rule(source, "no-broad-except");
    assert_eq!(d.len(), 1);
    assert_eq!(d[0].line, 3);
}

#[test]
fn except_base_exception() {
    let source = "try:\n    pass\nexcept BaseException:\n    pass";
    let d = lint_with_rule(source, "no-broad-except");
    assert_eq!(d.len(), 1);
}

#[test]
fn except_exception_with_alias() {
    let source = "try:\n    pass\nexcept Exception as e:\n    pass";
    let d = lint_with_rule(source, "no-broad-except");
    assert_eq!(d.len(), 1);
}

#[test]
fn tuple_with_exception() {
    let source = "try:\n    pass\nexcept (Exception, ValueError):\n    pass";
    let d = lint_with_rule(source, "no-broad-except");
    assert_eq!(d.len(), 1);
}

#[test]
fn tuple_with_base_exception() {
    let source = "try:\n    pass\nexcept (ValueError, BaseException):\n    pass";
    let d = lint_with_rule(source, "no-broad-except");
    assert_eq!(d.len(), 1);
}

#[test]
fn specific_type_ok() {
    let source = "try:\n    pass\nexcept ValueError:\n    pass";
    let d = lint_with_rule(source, "no-broad-except");
    assert_eq!(d.len(), 0);
}

#[test]
fn specific_tuple_ok() {
    let source = "try:\n    pass\nexcept (KeyError, TypeError):\n    pass";
    let d = lint_with_rule(source, "no-broad-except");
    assert_eq!(d.len(), 0);
}

#[test]
fn bare_except_not_flagged() {
    // Bare except: is handled by no-bare-except, not no-broad-except
    let source = "try:\n    pass\nexcept:\n    pass";
    let d = lint_with_rule(source, "no-broad-except");
    assert_eq!(d.len(), 0);
}

#[test]
fn in_comment() {
    let d = lint_with_rule("# except Exception:", "no-broad-except");
    assert_eq!(d.len(), 0);
}
