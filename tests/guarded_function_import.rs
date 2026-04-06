mod helpers;
use helpers::lint_with_rule;

#[test]
fn module_scope_import() {
    let d = lint_with_rule("import os", "guarded-function-import");
    assert_eq!(d.len(), 0);
}

#[test]
fn module_scope_from_import() {
    let d = lint_with_rule("from os import path", "guarded-function-import");
    assert_eq!(d.len(), 0);
}

#[test]
fn function_scope_unguarded() {
    let source = "def f():\n    import os";
    let d = lint_with_rule(source, "guarded-function-import");
    assert_eq!(d.len(), 1);
    assert_eq!(d[0].line, 2);
}

#[test]
fn function_scope_from_import_unguarded() {
    let source = "def f():\n    from os import path";
    let d = lint_with_rule(source, "guarded-function-import");
    assert_eq!(d.len(), 1);
}

#[test]
fn function_scope_guarded() {
    let source = "def f():\n    # Deferred: circular import\n    import os";
    let d = lint_with_rule(source, "guarded-function-import");
    assert_eq!(d.len(), 0);
}

#[test]
fn blank_line_between_comment_and_import() {
    let source = "def f():\n    # comment\n    \n    import os";
    let d = lint_with_rule(source, "guarded-function-import");
    assert_eq!(d.len(), 1);
}

#[test]
fn inline_comment_does_not_count() {
    let source = "def f():\n    import os  # reason";
    let d = lint_with_rule(source, "guarded-function-import");
    assert_eq!(d.len(), 1);
}

#[test]
fn inside_try_unguarded() {
    let source = "def f():\n    try:\n        import fast_lib\n    except ImportError:\n        pass";
    let d = lint_with_rule(source, "guarded-function-import");
    assert_eq!(d.len(), 1);
}

#[test]
fn inside_try_guarded() {
    let source = "def f():\n    try:\n        # Optional fast path\n        import fast_lib\n    except ImportError:\n        pass";
    let d = lint_with_rule(source, "guarded-function-import");
    assert_eq!(d.len(), 0);
}

#[test]
fn inside_if() {
    let source = "def f():\n    if cond:\n        import x";
    let d = lint_with_rule(source, "guarded-function-import");
    assert_eq!(d.len(), 1);
}

#[test]
fn inside_with() {
    let source = "def f():\n    with ctx:\n        import x";
    let d = lint_with_rule(source, "guarded-function-import");
    assert_eq!(d.len(), 1);
}

#[test]
fn class_method() {
    let source = "class C:\n    def m(self):\n        import x";
    let d = lint_with_rule(source, "guarded-function-import");
    assert_eq!(d.len(), 1);
}

#[test]
fn nested_function() {
    let source = "def f():\n    def g():\n        import x";
    let d = lint_with_rule(source, "guarded-function-import");
    assert_eq!(d.len(), 1);
}

#[test]
fn class_body() {
    let source = "class C:\n    import x";
    let d = lint_with_rule(source, "guarded-function-import");
    assert_eq!(d.len(), 1);
}

#[test]
fn consecutive_second_unguarded() {
    let source = "def f():\n    # reason\n    import a\n    import b";
    let d = lint_with_rule(source, "guarded-function-import");
    assert_eq!(d.len(), 1);
    assert_eq!(d[0].line, 4);
}

#[test]
fn consecutive_both_guarded() {
    let source = "def f():\n    # reason a\n    import a\n    # reason b\n    import b";
    let d = lint_with_rule(source, "guarded-function-import");
    assert_eq!(d.len(), 0);
}

#[test]
fn async_function() {
    let source = "async def f():\n    import x";
    let d = lint_with_rule(source, "guarded-function-import");
    assert_eq!(d.len(), 1);
}

#[test]
fn module_level_if_import() {
    let source = "if True:\n    import os";
    let d = lint_with_rule(source, "guarded-function-import");
    assert_eq!(d.len(), 0);
}
