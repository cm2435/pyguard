use crate::diagnostic::Diagnostic;
use crate::rules::Rule;
use crate::rules::no_bare_except::find_exception_type;

const BROAD_TYPES: &[&str] = &["Exception", "BaseException"];

/// Flags `except Exception:` and `except BaseException:`,
/// including when they appear inside tuples like `except (Exception, ValueError):`.
pub struct NoBroadExcept;

impl Rule for NoBroadExcept {
    fn name(&self) -> &'static str {
        "no-broad-except"
    }

    fn node_kinds(&self) -> &'static [&'static str] {
        &["except_clause"]
    }

    fn check(
        &self,
        node: &tree_sitter::Node,
        source: &[u8],
        _ancestors: &[tree_sitter::Node],
        diagnostics: &mut Vec<Diagnostic>,
    ) {
        let Some(type_text) = find_exception_type(node, source) else {
            return; // bare except -- handled by no-bare-except
        };

        // Check if the type itself is broad
        if BROAD_TYPES.contains(&type_text) {
            diagnostics.push(make_diagnostic(node, type_text));
            return;
        }

        // Check tuple members: "(Exception, ValueError)" or "(ValueError, BaseException)"
        if type_text.starts_with('(') {
            for member in type_text
                .trim_start_matches('(')
                .trim_end_matches(')')
                .split(',')
            {
                let trimmed = member.trim();
                if BROAD_TYPES.contains(&trimmed) {
                    diagnostics.push(make_diagnostic(node, trimmed));
                    return;
                }
            }
        }
    }
}

fn make_diagnostic(node: &tree_sitter::Node, broad_type: &str) -> Diagnostic {
    Diagnostic {
        path: String::new(),
        line: node.start_position().row + 1,
        col: node.start_position().column,
        rule_id: "no-broad-except",
        message: format!(
            "`except {broad_type}` is too broad; catch specific exception types"
        ),
    }
}
