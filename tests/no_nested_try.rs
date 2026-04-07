mod helpers;
use helpers::lint_with_rule;

#[test]
fn nested_try() {
    let source = "try:\n    try:\n        pass\n    except:\n        pass\nexcept:\n    pass";
    let d = lint_with_rule(source, "no-nested-try");
    assert_eq!(d.len(), 1);
    assert_eq!(d[0].line, 2);
}

#[test]
fn single_try_ok() {
    let source = "try:\n    pass\nexcept ValueError:\n    pass";
    let d = lint_with_rule(source, "no-nested-try");
    assert_eq!(d.len(), 0);
}

#[test]
fn nested_in_except_block() {
    let source = "try:\n    pass\nexcept:\n    try:\n        pass\n    except:\n        pass";
    let d = lint_with_rule(source, "no-nested-try");
    assert_eq!(d.len(), 1);
    assert_eq!(d[0].line, 4);
}

#[test]
fn nested_in_function_inside_try() {
    // try inside a function inside a try: the function boundary
    // doesn't break the nesting -- the inner try IS inside the outer try's tree
    let source = "try:\n    def f():\n        try:\n            pass\n        except:\n            pass\nexcept:\n    pass";
    let d = lint_with_rule(source, "no-nested-try");
    assert_eq!(d.len(), 1);
}

#[test]
fn sequential_tries_ok() {
    let source = "try:\n    pass\nexcept:\n    pass\ntry:\n    pass\nexcept:\n    pass";
    let d = lint_with_rule(source, "no-nested-try");
    assert_eq!(d.len(), 0);
}

#[test]
fn triple_nested() {
    let source = "try:\n    try:\n        try:\n            pass\n        except:\n            pass\n    except:\n        pass\nexcept:\n    pass";
    let d = lint_with_rule(source, "no-nested-try");
    // Middle try has outer ancestor, innermost has both outer and middle
    assert_eq!(d.len(), 2);
}

#[test]
fn in_comment_ok() {
    let d = lint_with_rule("# try:\n#     try:", "no-nested-try");
    assert_eq!(d.len(), 0);
}
