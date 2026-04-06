use crate::diagnostic::Diagnostic;

/// Filter out diagnostics suppressed by inline `# pyguard: ignore` comments.
pub fn filter_suppressed(diagnostics: Vec<Diagnostic>, source: &str) -> Vec<Diagnostic> {
    let lines: Vec<&str> = source.lines().collect();
    diagnostics
        .into_iter()
        .filter(|d| {
            let line = lines.get(d.line.wrapping_sub(1)).copied().unwrap_or("");
            !is_suppressed(line, d.rule_id)
        })
        .collect()
}

/// Check if a source line suppresses a given rule_id via `# pyguard: ignore`.
///
/// Supported forms:
///   # pyguard: ignore                           -- blanket, suppresses all rules
///   # pyguard: ignore[rule-id]                  -- targeted single
///   # pyguard: ignore[rule-a, rule-b]           -- targeted multi
pub fn is_suppressed(line: &str, rule_id: &str) -> bool {
    let Some(comment_start) = line.find("# pyguard: ignore") else {
        return false;
    };

    let after = &line[comment_start + "# pyguard: ignore".len()..];

    // Blanket ignore: nothing after, or only whitespace
    let after_trimmed = after.trim_start();
    if after_trimmed.is_empty() {
        return true;
    }

    // Targeted: must start with '['
    if !after_trimmed.starts_with('[') {
        return false;
    }

    let Some(bracket_end) = after_trimmed.find(']') else {
        return false;
    };

    let inside = &after_trimmed[1..bracket_end];
    inside
        .split(',')
        .any(|item| item.trim() == rule_id)
}
