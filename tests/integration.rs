use pyguard::{lint_source, lint_source_with_config, lint_source_with_rules, Config, Diagnostic};
use pyguard::rules::{self, Rule};

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

fn count_rule(diagnostics: &[Diagnostic], rule_id: &str) -> usize {
    diagnostics.iter().filter(|d| d.rule_id == rule_id).count()
}

fn lint_with_rule(source: &str, rule_name: &str) -> Vec<Diagnostic> {
    let all = rules::all_rules();
    let rule: Vec<&dyn Rule> = all.iter()
        .filter(|r| r.name() == rule_name)
        .map(|r| r.as_ref())
        .collect();
    lint_source_with_rules(source, "<test>", &rule)
}

// ===========================================================================
// Rule 1: no-hasattr-getattr
// ===========================================================================

#[test]
fn r1_bare_hasattr() {
    let d = lint_with_rule(r#"hasattr(obj, "x")"#, "no-hasattr-getattr");
    assert_eq!(d.len(), 1);
    assert_eq!(d[0].line, 1);
    assert_eq!(d[0].rule_id, "no-hasattr-getattr");
}

#[test]
fn r1_bare_getattr() {
    let d = lint_with_rule(r#"getattr(obj, "x")"#, "no-hasattr-getattr");
    assert_eq!(d.len(), 1);
}

#[test]
fn r1_getattr_three_args() {
    let d = lint_with_rule(r#"getattr(obj, "x", None)"#, "no-hasattr-getattr");
    assert_eq!(d.len(), 1);
}

#[test]
fn r1_builtins_hasattr() {
    let d = lint_with_rule(r#"builtins.hasattr(obj, "x")"#, "no-hasattr-getattr");
    assert_eq!(d.len(), 1);
}

#[test]
fn r1_builtins_getattr() {
    let d = lint_with_rule(r#"builtins.getattr(obj, "x")"#, "no-hasattr-getattr");
    assert_eq!(d.len(), 1);
}

#[test]
fn r1_in_comment() {
    let d = lint_with_rule("# hasattr(obj, \"x\")", "no-hasattr-getattr");
    assert_eq!(d.len(), 0);
}

#[test]
fn r1_in_string() {
    let d = lint_with_rule("x = \"hasattr(obj, 'x')\"", "no-hasattr-getattr");
    assert_eq!(d.len(), 0);
}

#[test]
fn r1_in_multiline_string() {
    let d = lint_with_rule("x = '''hasattr(obj, 'x')'''", "no-hasattr-getattr");
    assert_eq!(d.len(), 0);
}

#[test]
fn r1_method_call() {
    let d = lint_with_rule("obj.hasattr()", "no-hasattr-getattr");
    assert_eq!(d.len(), 1);
}

#[test]
fn r1_in_expression() {
    let d = lint_with_rule("result = getattr(obj, key) if cond else default", "no-hasattr-getattr");
    assert_eq!(d.len(), 1);
}

#[test]
fn r1_multiple_in_file() {
    let source = "hasattr(a, \"x\")\ngetattr(b, \"y\")\nhasattr(c, \"z\")";
    let d = lint_with_rule(source, "no-hasattr-getattr");
    assert_eq!(d.len(), 3);
}

#[test]
fn r1_inside_function() {
    let source = "def f():\n    hasattr(self, \"x\")";
    let d = lint_with_rule(source, "no-hasattr-getattr");
    assert_eq!(d.len(), 1);
    assert_eq!(d[0].line, 2);
}

#[test]
fn r1_inside_lambda() {
    let source = "f = lambda: hasattr(x, \"y\")";
    let d = lint_with_rule(source, "no-hasattr-getattr");
    assert_eq!(d.len(), 1);
}

// ===========================================================================
// Rule 2: guarded-function-import
// ===========================================================================

#[test]
fn r2_module_scope_import() {
    let d = lint_with_rule("import os", "guarded-function-import");
    assert_eq!(d.len(), 0);
}

#[test]
fn r2_module_scope_from_import() {
    let d = lint_with_rule("from os import path", "guarded-function-import");
    assert_eq!(d.len(), 0);
}

#[test]
fn r2_function_scope_unguarded() {
    let source = "def f():\n    import os";
    let d = lint_with_rule(source, "guarded-function-import");
    assert_eq!(d.len(), 1);
    assert_eq!(d[0].line, 2);
}

#[test]
fn r2_function_scope_from_import_unguarded() {
    let source = "def f():\n    from os import path";
    let d = lint_with_rule(source, "guarded-function-import");
    assert_eq!(d.len(), 1);
}

#[test]
fn r2_function_scope_guarded() {
    let source = "def f():\n    # Deferred: circular import\n    import os";
    let d = lint_with_rule(source, "guarded-function-import");
    assert_eq!(d.len(), 0);
}

#[test]
fn r2_blank_line_between_comment_and_import() {
    // Blank line sits between the comment and the import, so NOT guarded
    let source = "def f():\n    # comment\n    \n    import os";
    let d = lint_with_rule(source, "guarded-function-import");
    assert_eq!(d.len(), 1);
}

#[test]
fn r2_inline_comment_does_not_count() {
    let source = "def f():\n    import os  # reason";
    let d = lint_with_rule(source, "guarded-function-import");
    assert_eq!(d.len(), 1);
}

#[test]
fn r2_inside_try_unguarded() {
    let source = "def f():\n    try:\n        import fast_lib\n    except ImportError:\n        pass";
    let d = lint_with_rule(source, "guarded-function-import");
    assert_eq!(d.len(), 1);
}

#[test]
fn r2_inside_try_guarded() {
    let source = "def f():\n    try:\n        # Optional fast path\n        import fast_lib\n    except ImportError:\n        pass";
    let d = lint_with_rule(source, "guarded-function-import");
    assert_eq!(d.len(), 0);
}

#[test]
fn r2_inside_if() {
    let source = "def f():\n    if cond:\n        import x";
    let d = lint_with_rule(source, "guarded-function-import");
    assert_eq!(d.len(), 1);
}

#[test]
fn r2_inside_with() {
    let source = "def f():\n    with ctx:\n        import x";
    let d = lint_with_rule(source, "guarded-function-import");
    assert_eq!(d.len(), 1);
}

#[test]
fn r2_class_method() {
    let source = "class C:\n    def m(self):\n        import x";
    let d = lint_with_rule(source, "guarded-function-import");
    assert_eq!(d.len(), 1);
}

#[test]
fn r2_nested_function() {
    let source = "def f():\n    def g():\n        import x";
    let d = lint_with_rule(source, "guarded-function-import");
    assert_eq!(d.len(), 1);
}

#[test]
fn r2_class_body() {
    let source = "class C:\n    import x";
    let d = lint_with_rule(source, "guarded-function-import");
    assert_eq!(d.len(), 1);
}

#[test]
fn r2_consecutive_second_unguarded() {
    let source = "def f():\n    # reason\n    import a\n    import b";
    let d = lint_with_rule(source, "guarded-function-import");
    assert_eq!(d.len(), 1);
    assert_eq!(d[0].line, 4);
}

#[test]
fn r2_consecutive_both_guarded() {
    let source = "def f():\n    # reason a\n    import a\n    # reason b\n    import b";
    let d = lint_with_rule(source, "guarded-function-import");
    assert_eq!(d.len(), 0);
}

#[test]
fn r2_async_function() {
    let source = "async def f():\n    import x";
    let d = lint_with_rule(source, "guarded-function-import");
    assert_eq!(d.len(), 1);
}

#[test]
fn r2_module_level_if_import() {
    let source = "if True:\n    import os";
    let d = lint_with_rule(source, "guarded-function-import");
    assert_eq!(d.len(), 0);
}

// ===========================================================================
// Rule 3: no-future-annotations
// ===========================================================================

#[test]
fn r3_basic() {
    let d = lint_with_rule("from __future__ import annotations", "no-future-annotations");
    assert_eq!(d.len(), 1);
    assert_eq!(d[0].line, 1);
}

#[test]
fn r3_division_ok() {
    let d = lint_with_rule("from __future__ import division", "no-future-annotations");
    assert_eq!(d.len(), 0);
}

#[test]
fn r3_multi_import() {
    let d = lint_with_rule("from __future__ import annotations, division", "no-future-annotations");
    assert_eq!(d.len(), 1);
}

#[test]
fn r3_parenthesized() {
    let d = lint_with_rule("from __future__ import (annotations)", "no-future-annotations");
    assert_eq!(d.len(), 1);
}

#[test]
fn r3_multiline_parenthesized() {
    let source = "from __future__ import (\n    annotations,\n)";
    let d = lint_with_rule(source, "no-future-annotations");
    assert_eq!(d.len(), 1);
}

#[test]
fn r3_plain_import() {
    let d = lint_with_rule("import __future__", "no-future-annotations");
    assert_eq!(d.len(), 0);
}

#[test]
fn r3_in_comment() {
    let d = lint_with_rule("# from __future__ import annotations", "no-future-annotations");
    assert_eq!(d.len(), 0);
}

#[test]
fn r3_in_string() {
    let d = lint_with_rule("x = \"from __future__ import annotations\"", "no-future-annotations");
    assert_eq!(d.len(), 0);
}

#[test]
fn r3_aliased() {
    let d = lint_with_rule("from __future__ import annotations as ann", "no-future-annotations");
    assert_eq!(d.len(), 1);
}

#[test]
fn r3_inside_function() {
    let source = "def f():\n    from __future__ import annotations";
    let d = lint_with_rule(source, "no-future-annotations");
    assert_eq!(d.len(), 1);
}

// ===========================================================================
// Rule 4: no-dataclass
// ===========================================================================

#[test]
fn r4_decorator_bare() {
    let source = "@dataclass\nclass Foo:\n    x: int";
    let d = lint_with_rule(source, "no-dataclass");
    assert_eq!(d.len(), 1);
    assert_eq!(d[0].line, 1);
}

#[test]
fn r4_decorator_with_args() {
    let source = "@dataclass(frozen=True)\nclass Foo:\n    x: int";
    let d = lint_with_rule(source, "no-dataclass");
    assert_eq!(d.len(), 1);
}

#[test]
fn r4_decorator_qualified() {
    let source = "@dataclasses.dataclass\nclass Foo:\n    x: int";
    let d = lint_with_rule(source, "no-dataclass");
    assert_eq!(d.len(), 1);
}

#[test]
fn r4_decorator_qualified_with_args() {
    let source = "@dataclasses.dataclass(eq=False)\nclass Foo:\n    pass";
    let d = lint_with_rule(source, "no-dataclass");
    assert_eq!(d.len(), 1);
}

#[test]
fn r4_from_import() {
    let d = lint_with_rule("from dataclasses import dataclass", "no-dataclass");
    assert_eq!(d.len(), 1);
}

#[test]
fn r4_from_import_multi() {
    let d = lint_with_rule("from dataclasses import dataclass, field", "no-dataclass");
    assert_eq!(d.len(), 1);
}

#[test]
fn r4_import_module() {
    let d = lint_with_rule("import dataclasses", "no-dataclass");
    assert_eq!(d.len(), 1);
}

#[test]
fn r4_unrelated_decorator() {
    let d = lint_with_rule("@some_other_decorator\nclass Foo:\n    pass", "no-dataclass");
    assert_eq!(d.len(), 0);
}

#[test]
fn r4_pydantic_ok() {
    let d = lint_with_rule("from pydantic import BaseModel", "no-dataclass");
    assert_eq!(d.len(), 0);
}

#[test]
fn r4_in_string() {
    let d = lint_with_rule("x = \"@dataclass\"", "no-dataclass");
    assert_eq!(d.len(), 0);
}

#[test]
fn r4_in_comment() {
    let d = lint_with_rule("# @dataclass", "no-dataclass");
    assert_eq!(d.len(), 0);
}

// ===========================================================================
// Rule 5: no-bare-except
// ===========================================================================

#[test]
fn r5_bare_except() {
    let source = "try:\n    pass\nexcept:\n    pass";
    let d = lint_with_rule(source, "no-bare-except");
    assert_eq!(d.len(), 1);
    assert_eq!(d[0].line, 3);
}

#[test]
fn r5_except_exception() {
    let source = "try:\n    pass\nexcept Exception:\n    pass";
    let d = lint_with_rule(source, "no-bare-except");
    assert_eq!(d.len(), 1);
}

#[test]
fn r5_except_base_exception() {
    let source = "try:\n    pass\nexcept BaseException:\n    pass";
    let d = lint_with_rule(source, "no-bare-except");
    assert_eq!(d.len(), 1);
}

#[test]
fn r5_specific_type_ok() {
    let source = "try:\n    pass\nexcept ValueError:\n    pass";
    let d = lint_with_rule(source, "no-bare-except");
    assert_eq!(d.len(), 0);
}

#[test]
fn r5_tuple_ok() {
    let source = "try:\n    pass\nexcept (KeyError, TypeError):\n    pass";
    let d = lint_with_rule(source, "no-bare-except");
    assert_eq!(d.len(), 0);
}

#[test]
fn r5_broad_with_alias() {
    let source = "try:\n    pass\nexcept Exception as e:\n    pass";
    let d = lint_with_rule(source, "no-bare-except");
    assert_eq!(d.len(), 1);
}

#[test]
fn r5_mixed_clauses() {
    let source = "try:\n    pass\nexcept ValueError:\n    pass\nexcept:\n    pass";
    let d = lint_with_rule(source, "no-bare-except");
    assert_eq!(d.len(), 1);
    assert_eq!(d[0].line, 5);
}

#[test]
fn r5_in_comment() {
    let d = lint_with_rule("# except:", "no-bare-except");
    assert_eq!(d.len(), 0);
}

// ===========================================================================
// Rule 6: no-print
// ===========================================================================

#[test]
fn r6_bare_print() {
    let d = lint_with_rule("print(\"hello\")", "no-print");
    assert_eq!(d.len(), 1);
    assert_eq!(d[0].line, 1);
}

#[test]
fn r6_print_with_args() {
    let d = lint_with_rule("print(x, y, sep=\",\")", "no-print");
    assert_eq!(d.len(), 1);
}

#[test]
fn r6_builtins_print() {
    let d = lint_with_rule("builtins.print(\"hello\")", "no-print");
    assert_eq!(d.len(), 1);
}

#[test]
fn r6_in_comment() {
    let d = lint_with_rule("# print(\"hello\")", "no-print");
    assert_eq!(d.len(), 0);
}

#[test]
fn r6_in_string() {
    let d = lint_with_rule("x = \"print('hello')\"", "no-print");
    assert_eq!(d.len(), 0);
}

#[test]
fn r6_inside_function() {
    let source = "def f():\n    print(\"debug\")";
    let d = lint_with_rule(source, "no-print");
    assert_eq!(d.len(), 1);
    assert_eq!(d[0].line, 2);
}

#[test]
fn r6_multiple() {
    let source = "print(\"a\")\nprint(\"b\")\nprint(\"c\")";
    let d = lint_with_rule(source, "no-print");
    assert_eq!(d.len(), 3);
}

#[test]
fn r6_logging_ok() {
    let d = lint_with_rule("logging.info(\"hello\")", "no-print");
    assert_eq!(d.len(), 0);
}

#[test]
fn r6_pprint_ok() {
    let d = lint_with_rule("pprint(obj)", "no-print");
    assert_eq!(d.len(), 0);
}

// ===========================================================================
// Configuration
// ===========================================================================

#[test]
fn config_default_all_active() {
    let source = "hasattr(obj, \"x\")";
    let d = lint_source_with_config(source, "<test>", &Config::default());
    assert!(count_rule(&d, "no-hasattr-getattr") >= 1);
}

#[test]
fn config_exclude_rule() {
    let source = "hasattr(obj, \"x\")";
    let config = Config { exclude: vec!["no-hasattr-getattr".to_string()] };
    let d = lint_source_with_config(source, "<test>", &config);
    assert_eq!(count_rule(&d, "no-hasattr-getattr"), 0);
}

#[test]
fn config_exclude_unrelated_rule() {
    let source = "hasattr(obj, \"x\")";
    let config = Config { exclude: vec!["no-print".to_string()] };
    let d = lint_source_with_config(source, "<test>", &config);
    assert!(count_rule(&d, "no-hasattr-getattr") >= 1);
}

#[test]
fn config_exclude_print() {
    let source = "print(\"hi\")";
    let config = Config { exclude: vec!["no-print".to_string()] };
    let d = lint_source_with_config(source, "<test>", &config);
    assert_eq!(count_rule(&d, "no-print"), 0);
}

#[test]
fn config_empty_exclude() {
    let source = "print(\"hi\")";
    let config = Config { exclude: vec![] };
    let d = lint_source_with_config(source, "<test>", &config);
    assert!(count_rule(&d, "no-print") >= 1);
}

// ===========================================================================
// Inline suppression
// ===========================================================================

#[test]
fn suppress_blanket() {
    let source = "hasattr(obj, \"x\")  # pyguard: ignore";
    let d = lint_source(source, "<test>");
    assert_eq!(count_rule(&d, "no-hasattr-getattr"), 0);
}

#[test]
fn suppress_targeted_match() {
    let source = "hasattr(obj, \"x\")  # pyguard: ignore[no-hasattr-getattr]";
    let d = lint_source(source, "<test>");
    assert_eq!(count_rule(&d, "no-hasattr-getattr"), 0);
}

#[test]
fn suppress_targeted_mismatch() {
    let source = "hasattr(obj, \"x\")  # pyguard: ignore[no-print]";
    let d = lint_source(source, "<test>");
    assert!(count_rule(&d, "no-hasattr-getattr") >= 1);
}

#[test]
fn suppress_multi_target() {
    let source = "hasattr(obj, \"x\")  # pyguard: ignore[no-hasattr-getattr, no-print]";
    let d = lint_source(source, "<test>");
    assert_eq!(count_rule(&d, "no-hasattr-getattr"), 0);
}

#[test]
fn suppress_print() {
    let source = "print(\"hi\")  # pyguard: ignore";
    let d = lint_source(source, "<test>");
    assert_eq!(count_rule(&d, "no-print"), 0);
}

#[test]
fn suppress_wrong_comment() {
    let source = "hasattr(obj, \"x\")  # some other comment";
    let d = lint_source(source, "<test>");
    assert!(count_rule(&d, "no-hasattr-getattr") >= 1);
}

#[test]
fn suppress_on_previous_line_does_not_work() {
    let source = "# pyguard: ignore\nhasattr(obj, \"x\")";
    let d = lint_source(source, "<test>");
    assert!(count_rule(&d, "no-hasattr-getattr") >= 1);
}

#[test]
fn suppress_function_import() {
    let source = "def f():\n    import os  # pyguard: ignore";
    let d = lint_source(source, "<test>");
    assert_eq!(count_rule(&d, "guarded-function-import"), 0);
}

// ===========================================================================
// Edge cases / robustness
// ===========================================================================

#[test]
fn edge_empty_file() {
    let d = lint_source("", "<test>");
    assert_eq!(d.len(), 0);
}

#[test]
fn edge_comments_only() {
    let d = lint_source("# just comments\n# more comments", "<test>");
    assert_eq!(d.len(), 0);
}

#[test]
fn edge_syntax_error_no_panic() {
    let d = lint_source("def f(\n    broken syntax", "<test>");
    // Should not panic. May or may not find violations.
    let _ = d;
}

#[test]
fn edge_binary_content_no_panic() {
    let d = lint_source("\x00\x01\x02", "<test>");
    let _ = d;
}

#[test]
fn edge_deeply_nested() {
    let mut source = String::new();
    for i in 0..50 {
        let indent = "    ".repeat(i);
        source.push_str(&format!("{}def f{}():\n", indent, i));
    }
    let deep_indent = "    ".repeat(50);
    source.push_str(&format!("{}import os", deep_indent));
    let d = lint_source(&source, "<test>");
    assert!(count_rule(&d, "guarded-function-import") >= 1);
}

#[test]
fn edge_module_level_if_import_ok() {
    let source = "if True:\n    import os";
    let d = lint_with_rule(source, "guarded-function-import");
    assert_eq!(d.len(), 0);
}

// ===========================================================================
// Cross-rule: lint_source runs all rules together
// ===========================================================================

#[test]
fn all_rules_fire_together() {
    let source = r#"from __future__ import annotations
import dataclasses
hasattr(obj, "x")
print("hello")
def f():
    import os
try:
    pass
except:
    pass
"#;
    let d = lint_source(source, "<test>");
    let ids: Vec<&str> = d.iter().map(|d| d.rule_id).collect();
    assert!(ids.contains(&"no-future-annotations"), "missing no-future-annotations");
    assert!(ids.contains(&"no-dataclass"), "missing no-dataclass");
    assert!(ids.contains(&"no-hasattr-getattr"), "missing no-hasattr-getattr");
    assert!(ids.contains(&"no-print"), "missing no-print");
    assert!(ids.contains(&"guarded-function-import"), "missing guarded-function-import");
    assert!(ids.contains(&"no-bare-except"), "missing no-bare-except");
}
