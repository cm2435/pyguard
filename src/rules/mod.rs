pub mod guarded_function_import;
pub mod max_function_params;
pub mod no_assert;
pub mod no_bare_except;
pub mod no_boolean_positional;
pub mod no_dataclass;
pub mod no_future_annotations;
pub mod no_hasattr_getattr;
pub mod no_nested_try;
pub mod no_pass_except;
pub mod no_print;
pub mod no_str_empty_default;
pub mod no_todo_comment;
pub mod no_typing_any;

use crate::diagnostic::Diagnostic;

pub trait Rule: Send + Sync {
    /// Unique rule identifier, e.g. "no-hasattr-getattr".
    fn name(&self) -> &'static str;

    /// CST node kinds this rule wants to inspect.
    fn node_kinds(&self) -> &'static [&'static str];

    /// Inspect a single node. Push to `diagnostics` if violated.
    fn check(
        &self,
        node: &tree_sitter::Node,
        source: &[u8],
        ancestors: &[tree_sitter::Node],
        diagnostics: &mut Vec<Diagnostic>,
    );
}

pub fn all_rules() -> Vec<Box<dyn Rule>> {
    vec![
        Box::new(no_hasattr_getattr::NoHasattrGetattr),
        Box::new(guarded_function_import::GuardedFunctionImport),
        Box::new(no_future_annotations::NoFutureAnnotations),
        Box::new(no_dataclass::NoDataclass),
        Box::new(no_bare_except::NoBareExcept),
        Box::new(no_print::NoPrint),
        Box::new(no_todo_comment::NoTodoComment),
        Box::new(no_str_empty_default::NoStrEmptyDefault),
        Box::new(no_typing_any::NoTypingAny),
        Box::new(no_assert::NoAssert),
        Box::new(no_nested_try::NoNestedTry),
        Box::new(no_pass_except::NoPassExcept),
        Box::new(max_function_params::MaxFunctionParams::default()),
        Box::new(no_boolean_positional::NoBooleanPositional),
    ]
}
