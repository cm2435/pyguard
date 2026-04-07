use std::path::PathBuf;
use std::process;
use std::sync::atomic::{AtomicUsize, Ordering};

use anyhow::{Context, Result};
use clap::Parser;
use ignore::WalkBuilder;
use rayon::prelude::*;

use pyguard::config;
use pyguard::diagnostic::Diagnostic;
use pyguard::engine;
use pyguard::rules::Severity;

#[derive(Parser)]
#[command(name = "pyguard", version, about = "Fast Python linter for LLM-generated anti-patterns")]
struct Cli {
    /// Files or directories to lint (walks recursively for .py files)
    #[arg(required = true)]
    paths: Vec<PathBuf>,

    /// Suppress output; only set exit code
    #[arg(short, long)]
    quiet: bool,

    /// Only fail (exit 1) on errors, not warnings
    #[arg(long)]
    warn_only: bool,

    /// Output format
    #[arg(long, default_value = "text")]
    format: OutputFormat,
}

#[derive(Clone, clap::ValueEnum)]
enum OutputFormat {
    Text,
    Json,
}

fn main() {
    let cli = Cli::parse();

    let code = match run(&cli) {
        Ok(has_violations) => {
            if has_violations { 1 } else { 0 }
        }
        Err(e) => {
            eprintln!("pyguard: {e:#}");
            2
        }
    };

    process::exit(code);
}

fn run(cli: &Cli) -> Result<bool> {
    let config = config::discover_config(
        &std::env::current_dir().unwrap_or_else(|_| {
            cli.paths
                .first()
                .cloned()
                .unwrap_or_else(|| PathBuf::from("."))
        }),
    );

    let files = collect_python_files(&cli.paths)?;

    if files.is_empty() {
        return Ok(false);
    }

    let error_count = AtomicUsize::new(0);
    let warning_count = AtomicUsize::new(0);

    let all_diagnostics: Vec<Vec<Diagnostic>> = files
        .par_iter()
        .filter_map(|path| {
            let source = match std::fs::read_to_string(path) {
                Ok(s) => s,
                Err(_) => return None,
            };

            let path_str = path.to_string_lossy();
            let diagnostics =
                engine::lint_source_with_config(&source, &path_str, &config);

            if diagnostics.is_empty() {
                None
            } else {
                for d in &diagnostics {
                    match d.severity {
                        Severity::Error => { error_count.fetch_add(1, Ordering::Relaxed); }
                        Severity::Warning => { warning_count.fetch_add(1, Ordering::Relaxed); }
                    }
                }
                Some(diagnostics)
            }
        })
        .collect();

    if !cli.quiet {
        let mut flat: Vec<&Diagnostic> = all_diagnostics.iter().flat_map(|v| v.iter()).collect();
        flat.sort_by(|a, b| {
            a.path
                .cmp(&b.path)
                .then(a.line.cmp(&b.line))
                .then(a.col.cmp(&b.col))
        });

        match cli.format {
            OutputFormat::Text => {
                for d in &flat {
                    eprintln!("{d}");
                }
            }
            OutputFormat::Json => {
                let entries: Vec<serde_json::Value> = flat
                    .iter()
                    .map(|d| {
                        serde_json::json!({
                            "path": d.path,
                            "line": d.line,
                            "col": d.col,
                            "rule_id": d.rule_id,
                            "severity": match d.severity {
                                Severity::Error => "error",
                                Severity::Warning => "warning",
                            },
                            "message": d.message,
                        })
                    })
                    .collect();
                eprintln!("{}", serde_json::to_string_pretty(&entries).unwrap());
            }
        }

        let errors = error_count.load(Ordering::Relaxed);
        let warnings = warning_count.load(Ordering::Relaxed);
        let total = errors + warnings;
        if total > 0 {
            eprintln!(
                "\nFound {total} violation{} ({errors} error{}, {warnings} warning{}) across {} file{}.",
                if total == 1 { "" } else { "s" },
                if errors == 1 { "" } else { "s" },
                if warnings == 1 { "" } else { "s" },
                all_diagnostics.len(),
                if all_diagnostics.len() == 1 { "" } else { "s" },
            );
        }
    }

    if cli.warn_only {
        Ok(error_count.load(Ordering::Relaxed) > 0)
    } else {
        let total = error_count.load(Ordering::Relaxed) + warning_count.load(Ordering::Relaxed);
        Ok(total > 0)
    }
}

fn collect_python_files(paths: &[PathBuf]) -> Result<Vec<PathBuf>> {
    let mut files = Vec::new();

    for path in paths {
        if path.is_file() {
            files.push(path.clone());
            continue;
        }

        if !path.exists() {
            anyhow::bail!("path does not exist: {}", path.display());
        }

        let walker = WalkBuilder::new(path)
            .hidden(true)
            .git_ignore(true)
            .build();

        for entry in walker {
            let entry = entry.context("walking directory")?;
            let p = entry.path();
            if p.is_file() && p.extension().is_some_and(|ext| ext == "py") {
                files.push(p.to_path_buf());
            }
        }
    }

    Ok(files)
}
