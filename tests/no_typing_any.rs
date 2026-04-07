mod helpers;
use helpers::lint_with_rule;

// -- Import detection --

#[test]
fn import_any() {
    let d = lint_with_rule("from typing import Any", "no-typing-any");
    assert_eq!(d.len(), 1);
    assert_eq!(d[0].line, 1);
}

#[test]
fn import_any_among_others() {
    let d = lint_with_rule("from typing import Any, Optional, Dict", "no-typing-any");
    assert_eq!(d.len(), 1);
}

#[test]
fn import_any_aliased() {
    let d = lint_with_rule("from typing import Any as AnyType", "no-typing-any");
    assert_eq!(d.len(), 1);
}

#[test]
fn import_any_from_extensions() {
    let d = lint_with_rule("from typing_extensions import Any", "no-typing-any");
    assert_eq!(d.len(), 1);
}

#[test]
fn import_typing_module() {
    // `import typing` alone doesn't use Any -- no violation
    let d = lint_with_rule("import typing", "no-typing-any");
    assert_eq!(d.len(), 0);
}

#[test]
fn import_other_from_typing_ok() {
    let d = lint_with_rule("from typing import Optional, Dict", "no-typing-any");
    assert_eq!(d.len(), 0);
}

// -- Usage in type annotations --

#[test]
fn param_typed_any() {
    let d = lint_with_rule("def f(x: Any):\n    pass", "no-typing-any");
    assert_eq!(d.len(), 1);
}

#[test]
fn param_typed_any_with_default() {
    let d = lint_with_rule("def f(x: Any = None):\n    pass", "no-typing-any");
    assert_eq!(d.len(), 1);
}

#[test]
fn return_type_any() {
    let d = lint_with_rule("def f() -> Any:\n    pass", "no-typing-any");
    assert_eq!(d.len(), 1);
}

#[test]
fn variable_annotation_any() {
    let d = lint_with_rule("x: Any = 5", "no-typing-any");
    assert_eq!(d.len(), 1);
}

#[test]
fn generic_any() {
    let d = lint_with_rule("def f(x: dict[str, Any]):\n    pass", "no-typing-any");
    assert_eq!(d.len(), 1);
}

#[test]
fn union_any() {
    let d = lint_with_rule("def f(x: str | Any):\n    pass", "no-typing-any");
    assert_eq!(d.len(), 1);
}

// -- False positive avoidance --

#[test]
fn variable_named_any_ok() {
    // `Any` as a regular variable name, not in a type context
    let d = lint_with_rule("Any = 42\nprint(Any)", "no-typing-any");
    assert_eq!(d.len(), 0);
}

#[test]
fn string_containing_any_ok() {
    let d = lint_with_rule("x = \"Any\"", "no-typing-any");
    assert_eq!(d.len(), 0);
}

#[test]
fn comment_containing_any_ok() {
    let d = lint_with_rule("# x: Any", "no-typing-any");
    assert_eq!(d.len(), 0);
}

#[test]
fn function_named_any_ok() {
    let d = lint_with_rule("def any_handler():\n    pass", "no-typing-any");
    assert_eq!(d.len(), 0);
}

#[test]
fn builtin_any_call_ok() {
    // The builtin any() function is not typing.Any
    let d = lint_with_rule("result = any(items)", "no-typing-any");
    assert_eq!(d.len(), 0);
}

// -- Combined import + usage --

#[test]
fn import_and_usage_both_flagged() {
    let source = "from typing import Any\n\ndef f(x: Any) -> Any:\n    pass";
    let d = lint_with_rule(source, "no-typing-any");
    // 1 import + 2 usage (param + return)
    assert_eq!(d.len(), 3);
}
