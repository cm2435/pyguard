mod helpers;
use helpers::lint_with_rule;

#[test]
fn except_only_pass() {
    let source = "try:\n    pass\nexcept ValueError:\n    pass";
    let d = lint_with_rule(source, "no-pass-except");
    assert_eq!(d.len(), 1);
    assert_eq!(d[0].line, 3);
}

#[test]
fn bare_except_only_pass() {
    let source = "try:\n    pass\nexcept:\n    pass";
    let d = lint_with_rule(source, "no-pass-except");
    assert_eq!(d.len(), 1);
}

#[test]
fn except_with_logging_ok() {
    let source = "try:\n    pass\nexcept ValueError:\n    logger.error(\"fail\")";
    let d = lint_with_rule(source, "no-pass-except");
    assert_eq!(d.len(), 0);
}

#[test]
fn except_with_raise_ok() {
    let source = "try:\n    pass\nexcept ValueError:\n    raise";
    let d = lint_with_rule(source, "no-pass-except");
    assert_eq!(d.len(), 0);
}

#[test]
fn except_pass_plus_comment_ok() {
    // pass + a comment still counts as only pass (comment is not a statement)
    let source = "try:\n    pass\nexcept ValueError:\n    # intentional\n    pass";
    let d = lint_with_rule(source, "no-pass-except");
    assert_eq!(d.len(), 1);
}

#[test]
fn except_with_multiple_statements_ok() {
    let source = "try:\n    pass\nexcept ValueError:\n    x = 1\n    pass";
    let d = lint_with_rule(source, "no-pass-except");
    assert_eq!(d.len(), 0);
}

#[test]
fn multiple_except_one_pass() {
    let source = "try:\n    pass\nexcept ValueError:\n    raise\nexcept TypeError:\n    pass";
    let d = lint_with_rule(source, "no-pass-except");
    assert_eq!(d.len(), 1);
    assert_eq!(d[0].line, 5);
}

#[test]
fn except_with_ellipsis_not_flagged() {
    // `...` (Ellipsis) is sometimes used as a placeholder, different from pass
    let source = "try:\n    pass\nexcept ValueError:\n    ...";
    let d = lint_with_rule(source, "no-pass-except");
    assert_eq!(d.len(), 0);
}
