mod helpers;
use helpers::lint_with_rule;

#[test]
fn decorator_bare() {
    let source = "@dataclass\nclass Foo:\n    x: int";
    let d = lint_with_rule(source, "no-dataclass");
    assert_eq!(d.len(), 1);
    assert_eq!(d[0].line, 1);
}

#[test]
fn decorator_with_args() {
    let source = "@dataclass(frozen=True)\nclass Foo:\n    x: int";
    let d = lint_with_rule(source, "no-dataclass");
    assert_eq!(d.len(), 1);
}

#[test]
fn decorator_qualified() {
    let source = "@dataclasses.dataclass\nclass Foo:\n    x: int";
    let d = lint_with_rule(source, "no-dataclass");
    assert_eq!(d.len(), 1);
}

#[test]
fn decorator_qualified_with_args() {
    let source = "@dataclasses.dataclass(eq=False)\nclass Foo:\n    pass";
    let d = lint_with_rule(source, "no-dataclass");
    assert_eq!(d.len(), 1);
}

#[test]
fn from_import() {
    let d = lint_with_rule("from dataclasses import dataclass", "no-dataclass");
    assert_eq!(d.len(), 1);
}

#[test]
fn from_import_multi() {
    let d = lint_with_rule("from dataclasses import dataclass, field", "no-dataclass");
    assert_eq!(d.len(), 1);
}

#[test]
fn import_module() {
    let d = lint_with_rule("import dataclasses", "no-dataclass");
    assert_eq!(d.len(), 1);
}

#[test]
fn unrelated_decorator() {
    let d = lint_with_rule("@some_other_decorator\nclass Foo:\n    pass", "no-dataclass");
    assert_eq!(d.len(), 0);
}

#[test]
fn pydantic_ok() {
    let d = lint_with_rule("from pydantic import BaseModel", "no-dataclass");
    assert_eq!(d.len(), 0);
}

#[test]
fn in_string() {
    let d = lint_with_rule("x = \"@dataclass\"", "no-dataclass");
    assert_eq!(d.len(), 0);
}

#[test]
fn in_comment() {
    let d = lint_with_rule("# @dataclass", "no-dataclass");
    assert_eq!(d.len(), 0);
}
