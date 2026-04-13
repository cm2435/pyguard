mod helpers;
use helpers::lint_with_rule;

const RULE: &str = "no-or-empty-coalesce";

// ── Should flag: empty dict ─────────────────────────────────────────

#[test]
fn attr_or_empty_dict() {
    let d = lint_with_rule("x = foo.bar or {}", RULE);
    assert_eq!(d.len(), 1);
}

#[test]
fn nested_attr_or_empty_dict() {
    let d = lint_with_rule("x = foo.bar.baz or {}", RULE);
    assert_eq!(d.len(), 1);
}

#[test]
fn deeply_nested_attr_or_empty_dict() {
    let d = lint_with_rule("x = foo.bar.baz.qux or {}", RULE);
    assert_eq!(d.len(), 1);
}

// ── Should flag: empty list ─────────────────────────────────────────

#[test]
fn attr_or_empty_list() {
    let d = lint_with_rule("x = foo.bar or []", RULE);
    assert_eq!(d.len(), 1);
}

// ── Should flag: empty string ───────────────────────────────────────

#[test]
fn attr_or_empty_string_double() {
    let d = lint_with_rule("x = foo.bar or \"\"", RULE);
    assert_eq!(d.len(), 1);
}

#[test]
fn attr_or_empty_string_single() {
    let d = lint_with_rule("x = foo.bar or ''", RULE);
    assert_eq!(d.len(), 1);
}

#[test]
fn attr_or_empty_triple_quoted() {
    let d = lint_with_rule("x = foo.bar or \"\"\"\"\"\"", RULE);
    assert_eq!(d.len(), 1);
}

// ── Should flag: empty tuple ────────────────────────────────────────

#[test]
fn attr_or_empty_tuple() {
    let d = lint_with_rule("x = foo.bar or ()", RULE);
    assert_eq!(d.len(), 1);
}

// ── Should flag: zero int ───────────────────────────────────────────

#[test]
fn attr_or_zero_int() {
    let d = lint_with_rule("x = foo.bar or 0", RULE);
    assert_eq!(d.len(), 1);
}

// ── Should flag: zero float ─────────────────────────────────────────

#[test]
fn attr_or_zero_float() {
    let d = lint_with_rule("x = foo.bar or 0.0", RULE);
    assert_eq!(d.len(), 1);
}

// ── Should flag: False ──────────────────────────────────────────────

#[test]
fn attr_or_false() {
    let d = lint_with_rule("x = foo.bar or False", RULE);
    assert_eq!(d.len(), 1);
}

// ── Should flag: empty bytes ────────────────────────────────────────

#[test]
fn attr_or_empty_bytes() {
    let d = lint_with_rule("x = foo.bar or b\"\"", RULE);
    assert_eq!(d.len(), 1);
}

// ── Should flag: empty set() call ───────────────────────────────────

#[test]
fn attr_or_empty_set_call() {
    let d = lint_with_rule("x = foo.bar or set()", RULE);
    assert_eq!(d.len(), 1);
}

// ── Should flag: self.attr ──────────────────────────────────────────

#[test]
fn self_attr_or_empty_list() {
    let source = "class C:\n    def m(self):\n        x = self.items or []";
    let d = lint_with_rule(source, RULE);
    assert_eq!(d.len(), 1);
}

#[test]
fn self_nested_attr_or_empty_list() {
    let source = "class C:\n    def m(self):\n        x = self.config.items or []";
    let d = lint_with_rule(source, RULE);
    assert_eq!(d.len(), 1);
}

// ── Should flag: various statement positions ────────────────────────

#[test]
fn in_return_statement() {
    let source = "def f(obj):\n    return obj.data or {}";
    let d = lint_with_rule(source, RULE);
    assert_eq!(d.len(), 1);
}

#[test]
fn in_function_call_arg() {
    let source = "process(obj.items or [])";
    let d = lint_with_rule(source, RULE);
    assert_eq!(d.len(), 1);
}

#[test]
fn in_for_loop() {
    let source = "for x in obj.items or []:\n    pass";
    let d = lint_with_rule(source, RULE);
    assert_eq!(d.len(), 1);
}

#[test]
fn multiple_violations_one_line() {
    let source = "a, b = foo.x or {}, foo.y or []";
    let d = lint_with_rule(source, RULE);
    assert_eq!(d.len(), 2);
}

// ── Should NOT flag: non-empty defaults ─────────────────────────────

#[test]
fn non_empty_dict_ok() {
    let d = lint_with_rule("x = foo.bar or {\"key\": \"val\"}", RULE);
    assert_eq!(d.len(), 0);
}

#[test]
fn non_empty_list_ok() {
    let d = lint_with_rule("x = foo.bar or [1, 2, 3]", RULE);
    assert_eq!(d.len(), 0);
}

#[test]
fn non_empty_string_ok() {
    let d = lint_with_rule("x = foo.bar or \"default\"", RULE);
    assert_eq!(d.len(), 0);
}

#[test]
fn non_zero_int_ok() {
    let d = lint_with_rule("x = foo.bar or 42", RULE);
    assert_eq!(d.len(), 0);
}

#[test]
fn non_zero_float_ok() {
    let d = lint_with_rule("x = foo.bar or 1.5", RULE);
    assert_eq!(d.len(), 0);
}

#[test]
fn true_ok() {
    let d = lint_with_rule("x = foo.bar or True", RULE);
    assert_eq!(d.len(), 0);
}

#[test]
fn non_empty_tuple_ok() {
    let d = lint_with_rule("x = foo.bar or (1, 2)", RULE);
    assert_eq!(d.len(), 0);
}

// ── Should NOT flag: or None ────────────────────────────────────────

#[test]
fn or_none_ok() {
    let d = lint_with_rule("x = foo.bar or None", RULE);
    assert_eq!(d.len(), 0);
}

// ── Should NOT flag: non-attribute LHS ──────────────────────────────

#[test]
fn bare_variable_or_empty_dict_ok() {
    let d = lint_with_rule("x = y or {}", RULE);
    assert_eq!(d.len(), 0);
}

#[test]
fn function_call_or_empty_list_ok() {
    let d = lint_with_rule("x = get_items() or []", RULE);
    assert_eq!(d.len(), 0);
}

#[test]
fn subscript_or_empty_ok() {
    let d = lint_with_rule("x = d['key'] or {}", RULE);
    assert_eq!(d.len(), 0);
}

// ── Should NOT flag: wrong operator ─────────────────────────────────

#[test]
fn and_operator_ok() {
    let d = lint_with_rule("x = foo.bar and {}", RULE);
    assert_eq!(d.len(), 0);
}

// ── Should NOT flag: set() with args ────────────────────────────────

#[test]
fn set_with_args_ok() {
    let d = lint_with_rule("x = foo.bar or set([1, 2])", RULE);
    assert_eq!(d.len(), 0);
}

// ── Should NOT flag: in comments / strings ──────────────────────────

#[test]
fn in_comment_ok() {
    let d = lint_with_rule("# x = foo.bar or {}", RULE);
    assert_eq!(d.len(), 0);
}

#[test]
fn in_string_literal_ok() {
    let d = lint_with_rule("s = 'foo.bar or {}'", RULE);
    assert_eq!(d.len(), 0);
}

// ── Line number accuracy ────────────────────────────────────────────

#[test]
fn reports_correct_line() {
    let source = "a = 1\nb = 2\nx = foo.bar or {}";
    let d = lint_with_rule(source, RULE);
    assert_eq!(d.len(), 1);
    assert_eq!(d[0].line, 3);
}
