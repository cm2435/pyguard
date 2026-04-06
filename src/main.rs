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
use pyguard::rules;

#[derive(Parser)]
#[command(name = "pyguard", version, about = "Fast Python linter for LLM-generated anti-patterns")]
struct Cli {
    /// Files or directories to lint (walks recursively for .py files)
    #[arg(required = true)]
    paths: Vec<PathBuf>,

    /// Suppress output; only set exit code
    #[arg(short, long)]
    quiet: bool,

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

    if let Err(e) = run(&cli) {
        eprintln!("pyguard: {e:#}");
        process::exit(2);
    }
}

fn run(cli: &Cli) -> Result<()> {
    let config = config::discover_config(
        cli.paths
            .first()
            .map(|p| p.as_path())
            .unwrap_or_else(|| std::path::Path::new(".")),
    );

    let all_rules = rules::all_rules();
    let active_rules: Vec<&dyn rules::Rule> = all_rules
        .iter()
        .filter(|r| !config.exclude.contains(&r.name().to_string()))
        .map(|r| r.as_ref())
        .collect();

    let files = collect_python_files(&cli.paths)?;

    if files.is_empty() {
        return Ok(());
    }

    let violation_count = AtomicUsize::new(0);

    let all_diagnostics: Vec<Vec<Diagnostic>> = files
        .par_iter()
        .filter_map(|path| {
            let source = match std::fs::read_to_string(path) {
                Ok(s) => s,
                Err(_) => return None,
            };

            let path_str = path.to_string_lossy();
            let diagnostics =
                engine::lint_source_with_rules(&source, &path_str, &active_rules);

            if diagnostics.is_empty() {
                None
            } else {
                violation_count.fetch_add(diagnostics.len(), Ordering::Relaxed);
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
                            "message": d.message,
                        })
                    })
                    .collect();
                eprintln!("{}", serde_json::to_string_pretty(&entries).unwrap());
            }
        }

        let total = violation_count.load(Ordering::Relaxed);
        if total > 0 {
            eprintln!(
                "\nFound {total} violation{} across {} file{}.",
                if total == 1 { "" } else { "s" },
                all_diagnostics.len(),
                if all_diagnostics.len() == 1 { "" } else { "s" },
            );
        }
    }

    if violation_count.load(Ordering::Relaxed) > 0 {
        process::exit(1);
    }

    Ok(())
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
