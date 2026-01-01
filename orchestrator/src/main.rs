mod config;

use clap::Parser;
use anyhow::{Context, Result};
use std::process::Command;
use std::path::Path;
use std::fs;
use tracing::{error, info, warn, instrument};
use config::{load_config, EvolutionSettings};

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
    /// Path to the target repository (must contain Evolve.toml)
    #[arg(short, long, default_value = ".")]
    target: String,

    /// The model used for Planning/Checking (The Architect)
    #[arg(long, default_value = "ollama/qwen3-coder:30b")]
    architect: String,

    /// The model used for Writing Code (The Editor)
    #[arg(long, default_value = "ollama/qwen3-coder:30b")]
    editor: String,
}

fn main() -> Result<()> {
    tracing_subscriber::fmt::init();
    
    let args = Args::parse();
    let target_dir = &args.target;
    
    info!("🚀 Loading Evolution Engine for: {}", target_dir);
    info!("🧠 Architect Model: {}", args.architect);
    info!("✍️  Editor Model:    {}", args.editor);

    // 1. Validation
    if !Path::new(target_dir).join("Evolve.toml").exists() {
        error!("❌ Missing Evolve.toml in target directory.");
        anyhow::bail!("Cannot evolve a project without instructions.");
    }

    // 2. Load Config
    let config = load_config(target_dir)?;
    let settings = config.evolution;

    info!("🎯 Goal: {}", settings.instruction.lines().next().unwrap_or(""));

    // 3. Bootstrap
    bootstrap_project(target_dir, &settings)?;

    // 4. Git Sanitation
    let start_commit = ensure_git_clean_state(target_dir)?;
    info!("📌 Baseline Snapshot: {}", &start_commit[0..7]);

    // 5. Evolution Loop
    for generation in 1..=settings.max_generations {
        info!("---------------------------------------------------");
        info!("🧬 Generation {}: Mutation Cycle", generation);

        // A. Mutate (The Agent)
        match run_agent_mutation(target_dir, &settings.instruction, &settings.files, &args.architect, &args.editor) {
            Ok(_) => info!("🤖 Agent finished."),
            Err(e) => {
                error!("💀 Agent failed: {}. Reverting...", e);
                revert_to_snapshot(target_dir, &start_commit)?;
                continue;
            }
        }

        // B. Verify (The Judge)
        match verify_fitness(target_dir, &settings.test_command) {
            Ok(true) => {
                info!("✅ SUCCESS: Generation {} survived.", generation);
                return Ok(());
            }
            Ok(false) => {
                warn!("❌ FAILURE: Generation {} died. Tests failed.", generation);
                revert_to_snapshot(target_dir, &start_commit)?;
            }
            Err(e) => {
                error!("💀 Test system crashed: {}. Reverting...", e);
                revert_to_snapshot(target_dir, &start_commit)?;
            }
        }
    }

    error!("💀 Evolution failed after {} generations.", settings.max_generations);
    Ok(())
}

// --- BOOTSTRAP ENGINE ---
fn bootstrap_project(target_dir: &str, settings: &EvolutionSettings) -> Result<()> {
    let cargo_path = Path::new(target_dir).join("Cargo.toml");
    if !cargo_path.exists() {
        let needs_lib = settings.files.iter().any(|f| f.contains("lib.rs"));
        let init_type = if needs_lib { "--lib" } else { "--bin" };
        info!("🌱 Seeding new Cargo project (Type: {})...", init_type);

        let output = Command::new("cargo")
            .current_dir(target_dir)
            .args(["init", init_type])
            .output()
            .context("Failed to run cargo init")?;

        if !output.status.success() {
            anyhow::bail!("Cargo init failed: {}", String::from_utf8_lossy(&output.stderr));
        }
    }

    for filename in &settings.files {
        let file_path = Path::new(target_dir).join(filename);
        if let Some(parent) = file_path.parent() {
            if !parent.exists() {
                fs::create_dir_all(parent)?;
            }
        }
        if !file_path.exists() {
            fs::write(&file_path, "// Scaffold\n")?;
        }
    }
    Ok(())
}

// --- GIT ENGINE ---
fn ensure_git_clean_state(target_dir: &str) -> Result<String> {
    let git_dir = Path::new(target_dir).join(".git");
    if !git_dir.exists() {
        warn!("📂 No local .git found. Initializing isolated repo...");
        Command::new("git").current_dir(target_dir).arg("init").output()?;
    }
    Command::new("git").current_dir(target_dir).args(["add", "."]).output()?;
    let diff_cached = Command::new("git").current_dir(target_dir).args(["diff", "--cached", "--quiet"]).status()?;
    if !diff_cached.success() {
        Command::new("git").current_dir(target_dir).args(["commit", "-m", "Genesis"]).output()?;
    }
    let output = Command::new("git").current_dir(target_dir).args(["rev-parse", "HEAD"]).output()?;
    Ok(String::from_utf8(output.stdout)?.trim().to_string())
}

fn revert_to_snapshot(target_dir: &str, sha: &str) -> Result<()> {
    info!("⏪ Reverting to snapshot: {}", &sha[0..7]);
    Command::new("git").current_dir(target_dir).args(["reset", "--hard", sha]).output()?;
    Command::new("git").current_dir(target_dir).args(["clean", "-fd"]).output()?;
    Ok(())
}

// --- CORE AGENT LOGIC (UPDATED FOR TDD) ---

#[instrument]
fn run_agent_mutation(
    target_dir: &str, 
    instruction: &str, 
    files: &[String], 
    architect_model: &str, 
    editor_model: &str
) -> Result<()> {
    info!("🤖 Spawning Aider (Architect: {} | Editor: {})...", architect_model, editor_model);
    
    // TDD ENFORCEMENT PROTOCOL
    // We wrap the user's instruction with a strict TDD mandate.
    let tdd_instruction = format!(
        "IMPORTANT: You are an expert Engineer practicing strict Test-Driven Development (TDD).\n\
        PROTOCOL:\n\
        1. FIRST: Create or update the test file to reflect the requirements. Assertions must fail initially.\n\
        2. SECOND: Implement the minimum code necessary to pass the new tests.\n\
        3. THIRD: Ensure the library functions added are called in main to avoid omission in cargo run.\n\
        3. Do NOT delete existing valid tests.\n\
        \n\
        TASK:\n\
        {}", 
        instruction
    );

    let abs_path = std::fs::canonicalize(target_dir)
        .context("Failed to resolve absolute path of target_dir")?;

    let mut cmd = Command::new("aider");
    cmd.current_dir(&abs_path)
       .arg("--model").arg(architect_model)
       .arg("--editor-model").arg(editor_model)
       .arg("--message").arg(tdd_instruction) // Use the TDD wrapper
       .arg("--yes"); 

    for file in files {
        cmd.arg(file);
    }

    let status = cmd.status().context("Failed to run Aider")?;
    if !status.success() {
        anyhow::bail!("Aider failed.");
    }
    Ok(())
}

#[instrument]
fn verify_fitness(target_dir: &str, test_cmd: &str) -> Result<bool> {
    info!("🧪 Verifying: '{}'", test_cmd);
    let parts: Vec<&str> = test_cmd.split_whitespace().collect();
    if parts.is_empty() { return Ok(false); }
    let output = Command::new(parts[0]).current_dir(target_dir).args(&parts[1..]).output()?;
    Ok(output.status.success())
}
