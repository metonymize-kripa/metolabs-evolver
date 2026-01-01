mod config;

use anyhow::{Context, Result};
use clap::Parser;
use config::{EvolutionSettings, load_config};
use std::fs;
use std::path::Path;
use std::process::Command;
use tracing::{error, info, instrument, warn};

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

    info!(
        "🎯 Goal: {}",
        settings.instruction.lines().next().unwrap_or("")
    );

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
        match run_agent_mutation(
            target_dir,
            &settings.instruction,
            &settings.files,
            &args.architect,
            &args.editor,
        ) {
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

    error!(
        "💀 Evolution failed after {} generations.",
        settings.max_generations
    );
    Ok(())
}

// --- BOOTSTRAP ENGINE ---
fn bootstrap_project(target_dir: &str, settings: &EvolutionSettings) -> Result<()> {
    // Check if bootstrap command is provided in config
    if let Some(bootstrap_cmd) = &settings.bootstrap_command {
        info!("🌱 Running bootstrap command: {}", bootstrap_cmd);

        let parts: Vec<&str> = bootstrap_cmd.split_whitespace().collect();
        if !parts.is_empty() {
            let output = Command::new(parts[0])
                .current_dir(target_dir)
                .args(&parts[1..])
                .output()
                .context("Failed to run bootstrap command")?;

            if !output.status.success() {
                warn!(
                    "Bootstrap command failed: {}",
                    String::from_utf8_lossy(&output.stderr)
                );
                warn!("Continuing anyway...");
            }
        }
    } else {
        // Fallback: Try to detect project type and bootstrap accordingly
        let cargo_path = Path::new(target_dir).join("Cargo.toml");
        let pyproject = Path::new(target_dir).join("pyproject.toml");

        if !cargo_path.exists() && !pyproject.exists() {
            // Try to infer from project_type or files
            let project_type = settings
                .project_type
                .as_deref()
                .or_else(|| infer_project_type(&settings.files));

            match project_type {
                Some("rust") => {
                    let needs_lib = settings.files.iter().any(|f| f.contains("lib.rs"));
                    let init_type = if needs_lib { "--lib" } else { "--bin" };
                    info!("🌱 Seeding new Cargo project (Type: {})...", init_type);

                    let output = Command::new("cargo")
                        .current_dir(target_dir)
                        .args(["init", init_type])
                        .output()
                        .context("Failed to run cargo init")?;

                    if !output.status.success() {
                        anyhow::bail!(
                            "Cargo init failed: {}",
                            String::from_utf8_lossy(&output.stderr)
                        );
                    }
                }
                Some("python") => {
                    info!("🌱 Python project detected. Skipping auto-bootstrap.");
                    info!(
                        "💡 Tip: Add 'bootstrap_command' to Evolve.toml if needed (e.g., 'poetry init -n')"
                    );
                }
                _ => {
                    info!("🌱 Unknown project type. Skipping bootstrap.");
                    info!(
                        "💡 Tip: Add 'bootstrap_command' to Evolve.toml for custom initialization."
                    );
                }
            }
        }
    }

    // Ensure all tracked files exist (scaffold if needed)
    for filename in &settings.files {
        let file_path = Path::new(target_dir).join(filename);
        if let Some(parent) = file_path.parent() {
            if !parent.exists() {
                fs::create_dir_all(parent)?;
            }
        }
        if !file_path.exists() {
            // Use custom scaffold content if provided, otherwise infer from extension
            let content = if let Some(scaffold) = &settings.scaffold_content {
                scaffold.clone()
            } else {
                infer_scaffold_content(filename, settings.file_extension.as_deref())
            };

            info!("📝 Creating scaffold file: {}", filename);
            fs::write(&file_path, content)?;
        }
    }
    Ok(())
}

// Helper function to infer project type from file extensions (Rust or Python only)
fn infer_project_type(files: &[String]) -> Option<&'static str> {
    for file in files {
        if file.ends_with(".rs") {
            return Some("rust");
        } else if file.ends_with(".py") {
            return Some("python");
        }
    }
    None
}

// Helper function to generate appropriate scaffold content (Rust or Python only)
fn infer_scaffold_content(filename: &str, explicit_ext: Option<&str>) -> String {
    // Check explicit extension first
    if let Some(ext) = explicit_ext {
        return match ext {
            ".rs" => "// TODO: Implement\n".to_string(),
            ".py" => "# TODO: Implement\n".to_string(),
            _ => "// TODO: Implement\n".to_string(),
        };
    }

    // Infer from filename
    if filename.ends_with(".rs") {
        "// TODO: Implement\n".to_string()
    } else if filename.ends_with(".py") {
        "# TODO: Implement\n".to_string()
    } else {
        "// TODO: Implement\n".to_string()
    }
}

// --- GIT ENGINE ---
fn ensure_git_clean_state(target_dir: &str) -> Result<String> {
    let git_dir = Path::new(target_dir).join(".git");
    if !git_dir.exists() {
        warn!("📂 No local .git found. Initializing isolated repo...");
        Command::new("git")
            .current_dir(target_dir)
            .arg("init")
            .output()?;
    }
    Command::new("git")
        .current_dir(target_dir)
        .args(["add", "."])
        .output()?;
    let diff_cached = Command::new("git")
        .current_dir(target_dir)
        .args(["diff", "--cached", "--quiet"])
        .status()?;
    if !diff_cached.success() {
        Command::new("git")
            .current_dir(target_dir)
            .args(["commit", "-m", "Genesis"])
            .output()?;
    }
    let output = Command::new("git")
        .current_dir(target_dir)
        .args(["rev-parse", "HEAD"])
        .output()?;
    Ok(String::from_utf8(output.stdout)?.trim().to_string())
}

fn revert_to_snapshot(target_dir: &str, sha: &str) -> Result<()> {
    info!("⏪ Reverting to snapshot: {}", &sha[0..7]);
    Command::new("git")
        .current_dir(target_dir)
        .args(["reset", "--hard", sha])
        .output()?;
    Command::new("git")
        .current_dir(target_dir)
        .args(["clean", "-fd"])
        .output()?;
    Ok(())
}

// --- CORE AGENT LOGIC (UPDATED FOR TDD) ---

#[instrument]
fn run_agent_mutation(
    target_dir: &str,
    instruction: &str,
    files: &[String],
    architect_model: &str,
    editor_model: &str,
) -> Result<()> {
    info!(
        "🤖 Spawning Aider (Architect: {} | Editor: {})...",
        architect_model, editor_model
    );

    // TDD ENFORCEMENT PROTOCOL
    // We wrap the user's instruction with a strict TDD mandate.
    let tdd_instruction = format!(
        "IMPORTANT: You are an expert Engineer practicing strict Test-Driven Development (TDD).\n\
        PROTOCOL:\n\
        1. FIRST: Create or update the test file to reflect the requirements. Assertions must fail initially.\n\
        2. SECOND: Implement the minimum code necessary to pass the new tests.\n\
        3. THIRD: Refactor if needed while keeping tests green.\n\
        4. Do NOT delete existing valid tests.\n\
        \n\
        TASK:\n\
        {}",
        instruction
    );

    let abs_path = std::fs::canonicalize(target_dir)
        .context("Failed to resolve absolute path of target_dir")?;

    let mut cmd = Command::new("aider");
    cmd.current_dir(&abs_path)
        .arg("--model")
        .arg(architect_model)
        .arg("--editor-model")
        .arg(editor_model)
        .arg("--message")
        .arg(tdd_instruction) // Use the TDD wrapper
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
    if parts.is_empty() {
        return Ok(false);
    }
    let output = Command::new(parts[0])
        .current_dir(target_dir)
        .args(&parts[1..])
        .output()?;
    Ok(output.status.success())
}
