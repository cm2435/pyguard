use crate::diagnostic::Diagnostic;
use crate::rules::{Rule, Severity};

pub struct GuardedFunctionImport;

impl Rule for GuardedFunctionImport {
    fn name(&self) -> &'static str {
        "guarded-function-import"
    }

    fn severity(&self) -> Severity { Severity::Warning }

    fn help(&self) -> &'static str {
        "Verify whether a real circular dependency or optional-dependency guard \
         exists before adding a `# reason` comment. Trace the import chain: does \
         module_A -> module_B -> ... -> module_A form a cycle? If a genuine cycle \
         exists, document it: `# reason: avoid import cycle between foo.bar and \
         baz.qux`. If the import is for an optional heavy dependency, document \
         that: `# reason: optional dep; imported only when feature X is used`. \
         If NO cycle or optional-dep guard exists, move the import to the top of \
         the file. Do NOT add a `# reason` comment just to silence this warning."
    }

    fn node_kinds(&self) -> &'static [&'static str] {
        &["import_statement", "import_from_statement"]
    }

    fn check(
        &self,
        node: &tree_sitter::Node,
        source: &[u8],
        ancestors: &[tree_sitter::Node],
        diagnostics: &mut Vec<Diagnostic>,
    ) {
        let in_non_module_scope = ancestors.iter().any(|a| {
            matches!(a.kind(), "function_definition" | "class_definition")
        });

        if !in_non_module_scope {
            return;
        }

        let import_row = node.start_position().row;
        if import_row == 0 {
            diagnostics.push(make_diagnostic(node));
            return;
        }

        // Check if line R-1 is a comment by examining the source text directly.
        // This is more robust than tree-sibling walking because tree-sitter-python
        // places comments at varying hierarchy levels (sibling of block vs inside block).
        let prev_line = source
            .split(|&b| b == b'\n')
            .nth(import_row - 1)
            .unwrap_or(b"");

        if prev_line.trim_ascii().starts_with(b"#") {
            return; // guarded
        }

        diagnostics.push(make_diagnostic(node));
    }
}

fn make_diagnostic(node: &tree_sitter::Node) -> Diagnostic {
    Diagnostic {
        path: String::new(),
        line: node.start_position().row + 1,
        col: node.start_position().column,
        rule_id: "guarded-function-import",
            severity: crate::rules::Severity::Error,
        message: "Function-scope `import` requires a `# reason` comment on the line immediately above".to_string(),
    }
}
