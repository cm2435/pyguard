use std::path::Path;

use anyhow::Result;
use serde::Deserialize;

#[derive(Debug, Default, Clone, Deserialize)]
pub struct Config {
    #[serde(default)]
    pub exclude: Vec<String>,
}

#[derive(Deserialize)]
struct PyprojectToml {
    tool: Option<ToolTable>,
}

#[derive(Deserialize)]
struct ToolTable {
    pyguard: Option<Config>,
}

/// Walk upward from `start_dir` looking for a pyproject.toml with [tool.pyguard].
/// Returns Config::default() if nothing is found.
pub fn discover_config(start_dir: &Path) -> Config {
    let mut dir = if start_dir.is_file() {
        start_dir.parent().unwrap_or(start_dir)
    } else {
        start_dir
    };

    loop {
        let candidate = dir.join("pyproject.toml");
        if candidate.is_file() {
            if let Ok(config) = try_load_config(&candidate) {
                return config;
            }
        }
        match dir.parent() {
            Some(parent) => dir = parent,
            None => return Config::default(),
        }
    }
}

fn try_load_config(path: &Path) -> Result<Config> {
    let content = std::fs::read_to_string(path)?;
    let pyproject: PyprojectToml = toml::from_str(&content)?;
    Ok(pyproject
        .tool
        .and_then(|t| t.pyguard)
        .unwrap_or_default())
}
