// orchestrator/src/config.rs

use serde::Deserialize;
use std::fs;
use std::path::Path;
use anyhow::{Context, Result};

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
