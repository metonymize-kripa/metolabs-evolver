# 🧬 Metolabs Evolver

**Local Code Evolution System**

Evolver is an experimental engine that uses AI agents to autonomously evolve and improve codebases through iterative mutation and verification. It employs a "survival of the fittest" approach, where code changes (mutations) are kept only if they pass a defined set of tests (verification).

## 📂 Project Structure

This monorepo contains three main components:

### 1. Orchestrator (Rust)
The brain of the operation. The Orchestrator manages the evolution lifecycle:
- **Bootstrap**: Sets up the target project using `cargo init` if needed.
- **Snapshot**: Creates git commits to save valid states ("Generations").
- **Mutation**: Uses AI agents (via `aider` and Ollama models like `qwen2.5-coder`) to modify code based on TDD instructions.
- **Verification**: Runs test commands to validate changes. Reverts failed generations.

**Usage:**
```bash
cd orchestrator
cargo run -- --target ../slow-fibo --architect ollama/qwen2.5-coder:32b --editor ollama/qwen2.5-coder:32b
```

### 2. Dashboard (Python/Textual)
A terminal-based user interface (TUI) for monitoring the evolution in real-time.
- **Visual History**: Browse through generations (commits).
- **Code Inspection**: View the source code state at any point in the evolution history.
- **Live Updates**: Refreshes as the Orchestrator works.

**Usage:**
```bash
cd dashboard
uv run tui.py --target ../slow-fibo
```
*(Requires `uv` for dependency management)*

### 3. Slow-Fibo (Rust)
A sample "patient" project used to demonstrate the system. It is a simple Rust application that the Orchestrator can be pointed at to practice evolution strategies.

## 🚀 Getting Started

1.  **Prerequisites**:
    - Rust (Cargo)
    - Python 3.11+ (and `uv`)
    - [Ollama](https://ollama.com/) running locally with suitable coding models.
    - [Aider](https://github.com/paul-gauthier/aider) installed and in your PATH.

2.  **Configuration**:
    Target projects must have an `Evolve.toml` file defining the goal and files to track.

3.  **Run**:
    Start the Orchestrator in one terminal and the Dashboard in another to watch your code evolve!
