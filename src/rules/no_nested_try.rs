use crate::diagnostic::Diagnostic;
use crate::rules::Rule;

pub struct NoNestedTry;

impl Rule for NoNestedTry {
    fn name(&self) -> &'static str {
        "no-nested-try"
    }

    fn node_kinds(&self) -> &'static [&'static str] {
        &["try_statement"]
    }

    fn check(
        &self,
        node: &tree_sitter::Node,
        _source: &[u8],
        ancestors: &[tree_sitter::Node],
        diagnostics: &mut Vec<Diagnostic>,
    ) {
        let nested = ancestors.iter().any(|a| a.kind() == "try_statement");
        if !nested {
            return;
        }

        diagnostics.push(Diagnostic {
            path: String::new(),
            line: node.start_position().row + 1,
            col: node.start_position().column,
            rule_id: "no-nested-try",
            message: "Avoid nested `try` blocks; extract the inner try into a separate function"
                .to_string(),
        });
    }
}
