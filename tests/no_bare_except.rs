mod helpers;
use helpers::lint_with_rule;

#[test]
fn bare_except() {
    let source = "try:\n    pass\nexcept:\n    pass";
    let d = lint_with_rule(source, "no-bare-except");
    assert_eq!(d.len(), 1);
    assert_eq!(d[0].line, 3);
}

#[test]
fn except_exception_not_flagged() {
    // except Exception is handled by no-broad-except, not no-bare-except
    let source = "try:\n    pass\nexcept Exception:\n    pass";
    let d = lint_with_rule(source, "no-bare-except");
    assert_eq!(d.len(), 0);
}

#[test]
fn specific_type_ok() {
    let source = "try:\n    pass\nexcept ValueError:\n    pass";
    let d = lint_with_rule(source, "no-bare-except");
    assert_eq!(d.len(), 0);
}

#[test]
fn tuple_ok() {
    let source = "try:\n    pass\nexcept (KeyError, TypeError):\n    pass";
    let d = lint_with_rule(source, "no-bare-except");
    assert_eq!(d.len(), 0);
}

#[test]
fn mixed_clauses_bare_second() {
    let source = "try:\n    pass\nexcept ValueError:\n    pass\nexcept:\n    pass";
    let d = lint_with_rule(source, "no-bare-except");
    assert_eq!(d.len(), 1);
    assert_eq!(d[0].line, 5);
}

#[test]
fn in_comment() {
    let d = lint_with_rule("# except:", "no-bare-except");
    assert_eq!(d.len(), 0);
}
