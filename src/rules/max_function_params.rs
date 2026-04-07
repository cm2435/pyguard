use crate::diagnostic::Diagnostic;
use crate::rules::Rule;

pub const DEFAULT_MAX_PARAMS: usize = 8;

pub struct MaxFunctionParams {
    pub max: usize,
}

impl Default for MaxFunctionParams {
    fn default() -> Self {
        Self { max: DEFAULT_MAX_PARAMS }
    }
}

impl Rule for MaxFunctionParams {
    fn name(&self) -> &'static str {
        "max-function-params"
    }

    fn node_kinds(&self) -> &'static [&'static str] {
        &["function_definition"]
    }

    fn check(
        &self,
        node: &tree_sitter::Node,
        source: &[u8],
        _ancestors: &[tree_sitter::Node],
        diagnostics: &mut Vec<Diagnostic>,
    ) {
        let Some(params) = node.child_by_field_name("parameters") else {
            return;
        };

        let mut count = 0;
        for i in 0..params.child_count() {
            let child = params.child(i).unwrap();
            let kind = child.kind();
            // Count actual parameter nodes, not punctuation
            if matches!(
                kind,
                "identifier"
                    | "typed_parameter"
                    | "default_parameter"
                    | "typed_default_parameter"
                    | "list_splat_pattern"
                    | "dictionary_splat_pattern"
            ) {
                // Skip `self` and `cls` as they're boilerplate
                if kind == "identifier" {
                    let text = child.utf8_text(source).unwrap_or("");
                    if text == "self" || text == "cls" {
                        continue;
                    }
                }
                if kind == "typed_parameter" || kind == "typed_default_parameter" {
                    if let Some(name) = child.child_by_field_name("name") {
                        let text = name.utf8_text(source).unwrap_or("");
                        if text == "self" || text == "cls" {
                            continue;
                        }
                    }
                }
                count += 1;
            }
        }

        if count > self.max {
            diagnostics.push(Diagnostic {
                path: String::new(),
                line: node.start_position().row + 1,
                col: node.start_position().column,
                rule_id: "max-function-params",
                message: format!(
                    "Function has {count} parameters (max {}); group related parameters into a dataclass or model",
                    self.max
                ),
            });
        }
    }
}
