mod helpers;
use helpers::lint_with_rule;

#[test]
fn basic() {
    let d = lint_with_rule("from __future__ import annotations", "no-future-annotations");
    assert_eq!(d.len(), 1);
    assert_eq!(d[0].line, 1);
}

#[test]
fn division_ok() {
    let d = lint_with_rule("from __future__ import division", "no-future-annotations");
    assert_eq!(d.len(), 0);
}

#[test]
fn multi_import() {
    let d = lint_with_rule("from __future__ import annotations, division", "no-future-annotations");
    assert_eq!(d.len(), 1);
}

#[test]
fn parenthesized() {
    let d = lint_with_rule("from __future__ import (annotations)", "no-future-annotations");
    assert_eq!(d.len(), 1);
}

#[test]
fn multiline_parenthesized() {
    let source = "from __future__ import (\n    annotations,\n)";
    let d = lint_with_rule(source, "no-future-annotations");
    assert_eq!(d.len(), 1);
}

#[test]
fn plain_import() {
    let d = lint_with_rule("import __future__", "no-future-annotations");
    assert_eq!(d.len(), 0);
}

#[test]
fn in_comment() {
    let d = lint_with_rule("# from __future__ import annotations", "no-future-annotations");
    assert_eq!(d.len(), 0);
}

#[test]
fn in_string() {
    let d = lint_with_rule("x = \"from __future__ import annotations\"", "no-future-annotations");
    assert_eq!(d.len(), 0);
}

#[test]
fn aliased() {
    let d = lint_with_rule("from __future__ import annotations as ann", "no-future-annotations");
    assert_eq!(d.len(), 1);
}

#[test]
fn inside_function() {
    let source = "def f():\n    from __future__ import annotations";
    let d = lint_with_rule(source, "no-future-annotations");
    assert_eq!(d.len(), 1);
}
