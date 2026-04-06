use crate::diagnostic::Diagnostic;
use crate::rules::Rule;

pub struct GuardedFunctionImport;

impl Rule for GuardedFunctionImport {
    fn name(&self) -> &'static str {
        "guarded-function-import"
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
        let lines: Vec<&[u8]> = source.split(|&b| b == b'\n').collect();
        let prev_line = lines.get(import_row - 1).copied().unwrap_or(b"");
        let trimmed = trim_ascii(prev_line);

        if trimmed.starts_with(b"#") {
            return; // guarded
        }

        diagnostics.push(make_diagnostic(node));
    }
}

fn trim_ascii(bytes: &[u8]) -> &[u8] {
    let start = bytes.iter().position(|b| !b.is_ascii_whitespace()).unwrap_or(bytes.len());
    let end = bytes.iter().rposition(|b| !b.is_ascii_whitespace()).map_or(start, |p| p + 1);
    &bytes[start..end]
}

fn make_diagnostic(node: &tree_sitter::Node) -> Diagnostic {
    Diagnostic {
        path: String::new(),
        line: node.start_position().row + 1,
        col: node.start_position().column,
        rule_id: "guarded-function-import",
        message: "Function-scope `import` requires a `# reason` comment on the line immediately above".to_string(),
    }
}
