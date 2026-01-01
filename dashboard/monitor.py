import time
import os
from git import Repo
from rich.console import Console
from rich.layout import Layout
from rich.panel import Panel
from rich.syntax import Syntax
from rich.live import Live
from rich.table import Table
from rich.text import Text

# CONFIGURATION
#REPO_PATH = "../slow-fib"  # Relative to dashboard/
REPO_PATH = "../gravity-eater"  # Relative to dashboard/

console = Console()

def get_layout():
    layout = Layout()
    layout.split(
        Layout(name="header", size=3),
        Layout(name="main", ratio=1),
        Layout(name="footer", size=3),
    )
    layout["main"].split_row(
        Layout(name="left"),
        Layout(name="right"),
    )
    return layout

def get_file_content(repo, commit_id, filename="src/lib.rs"):
    try:
        # Access the file at a specific commit
        target_file = repo.commit(commit_id).tree / filename
        content = target_file.data_stream.read().decode('utf-8')
        return content
    except:
        return "// File not found in this version"

def generate_dashboard(layout, repo):
    # 1. Fetch Git Data
    try:
        commits = list(repo.iter_commits())
        head = commits[0]
        root = commits[-1]
    except Exception as e:
        return # wait for repo init

    # 2. Header
    title = Text("🧬 Project Evolve: Live Genome Tracker", style="bold magenta")
    layout["header"].update(Panel(title, style="on black"))

    # 3. Left Pane (Ancestor)
    ancestor_code = get_file_content(repo, root.hexsha)
    syntax_left = Syntax(ancestor_code, "rust", theme="monokai", line_numbers=True)
    layout["left"].update(
        Panel(syntax_left, title=f"🦕 Ancestor (SHA: {root.hexsha[:7]})", style="red")
    )

    # 4. Right Pane (Survivor)
    survivor_code = get_file_content(repo, head.hexsha)
    # Detect Iterative vs Recursive simply by string matching for now
    style_color = "green" if "for " in survivor_code or "while " in survivor_code else "yellow"
    
    syntax_right = Syntax(survivor_code, "rust", theme="monokai", line_numbers=True)
    layout["right"].update(
        Panel(syntax_right, title=f"🚀 Survivor (SHA: {head.hexsha[:7]})", style=style_color)
    )

    # 5. Footer (Metrics)
    metrics_table = Table.grid(expand=True)
    metrics_table.add_column(justify="center", ratio=1)
    metrics_table.add_column(justify="center", ratio=1)
    metrics_table.add_column(justify="center", ratio=1)
    
    metrics_table.add_row(
        f"[bold]Generations:[/bold] {len(commits)}",
        f"[bold]Latest Msg:[/bold] {head.message.strip()}",
        "[bold green]System Active[/bold green]"
    )
    layout["footer"].update(Panel(metrics_table, style="blue"))

def run():
    layout = get_layout()
    try:
        repo = Repo(REPO_PATH)
    except:
        console.print(f"[red]Error: Could not find git repo at {REPO_PATH}[/red]")
        return

    # The Live Context Manager handles the flicker-free rendering
    with Live(layout, refresh_per_second=2, screen=True):
        while True:
            # Re-open repo to ensure we catch external git changes
            # (GitPython caches objects, so we need to be careful, 
            # but for a simple viewer, re-reading commit list is usually enough)
            repo.git.clear_cache() 
            generate_dashboard(layout, repo)
            time.sleep(0.5)

if __name__ == "__main__":
    run()
