// orchestrator/src/config.rs

use anyhow::{Context, Result};
use serde::Deserialize;
use std::fs;
use std::path::Path;

#[derive(Deserialize, Debug)]
pub struct EvolutionConfig {
    pub evolution: EvolutionSettings,
}

#[derive(Deserialize, Debug)]
pub struct EvolutionSettings {
    pub instruction: String,

    #[serde(default = "default_test_cmd")]
    pub test_command: String,

    #[serde(default = "default_generations")]
    pub max_generations: u32,

    // NEW: Allow specifying exactly which files Aider should see
    #[serde(default = "default_files")]
    pub files: Vec<String>,

    // NEW: Project type detection (optional)
    #[serde(default)]
    pub project_type: Option<String>, // "rust", "python", "javascript", "go", etc.

    // NEW: Optional bootstrap command to initialize project
    #[serde(default)]
    pub bootstrap_command: Option<String>, // e.g., "cargo init --lib", "npm init -y", "poetry init"

    // NEW: Optional file extension for scaffolding
    #[serde(default)]
    pub file_extension: Option<String>, // e.g., ".rs", ".py", ".js"

    // NEW: Optional scaffold template content
    #[serde(default)]
    pub scaffold_content: Option<String>, // Custom content for new files

    // NEW: Optional primary file for dashboard display
    #[serde(default)]
    pub primary_file: Option<String>, // e.g., "src/main.py", "index.js"
}

fn default_test_cmd() -> String {
    "cargo nextest run".to_string()
}

fn default_generations() -> u32 {
    5
}

// Default fallback if user doesn't specify files
fn default_files() -> Vec<String> {
    vec!["src/lib.rs".to_string()]
}

pub fn load_config(target_dir: &str) -> Result<EvolutionConfig> {
    let path = Path::new(target_dir).join("Evolve.toml");
    let content = fs::read_to_string(&path)
        .with_context(|| format!("Could not find Evolve.toml in {}", target_dir))?;

    let config: EvolutionConfig = toml::from_str(&content)?;
    Ok(config)
}
