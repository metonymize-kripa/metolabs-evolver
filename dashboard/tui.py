import argparse
import toml
from pathlib import Path
from git import Repo
from textual.app import App, ComposeResult
from textual.containers import Container, VerticalScroll
from textual.widgets import Header, Footer, Static, ListView, ListItem, Label
from rich.syntax import Syntax

class CodeWindow(Static):
    """Display code with syntax highlighting."""
    def show_code(self, code, gen_id, filename):
        # Lexer detection for Rust, Python, and common config formats
        lexer = self._detect_lexer(filename)
        
        self.update(Syntax(code, lexer, theme="monokai", line_numbers=True, word_wrap=True))
        self.parent.border_title = f"{filename} ({gen_id})"
    
    def _detect_lexer(self, filename: str) -> str:
        """Detect the appropriate lexer based on file extension."""
        if filename.endswith(".rs"):
            return "rust"
        elif filename.endswith(".py"):
            return "python"
        elif filename.endswith((".toml", ".tml")):
            return "toml"
        elif filename.endswith((".yaml", ".yml")):
            return "yaml"
        elif filename.endswith(".json"):
            return "json"
        elif filename.endswith((".md", ".markdown")):
            return "markdown"
        else:
            return "text"  # Fallback for unknown types

class GenerationItem(ListItem):
    """A selectable item representing a generation."""
    def __init__(self, commit, generation_num):
        self.commit_hex = commit.hexsha
        msg = commit.message.split('\n')[0].strip()
        self.message = msg
        self.generation_num = generation_num
        super().__init__()

    def compose(self) -> ComposeResult:
        icon = "🧬" 
        if "Genesis" in self.message: icon = "🥚"
        yield Label(f"{icon} Gen {self.generation_num}: {self.message[:30]}...")

class EvolutionApp(App):
    """The generic TUI for any Evolve project."""
    CSS = """
    Screen { layout: horizontal; }
    #sidebar-container { width: 35; dock: left; border-right: vkey $accent; background: $surface; }
    #code-container { width: 1fr; height: 100%; border: solid $accent; }
    ListView { height: 100%; }
    ListItem { padding: 1; border-bottom: solid $primary-background-darken-1; }
    ListItem:hover { background: $primary-background-darken-2; }
    """

    BINDINGS = [("q", "quit", "Quit"), ("r", "refresh_repo", "Refresh")]

    def __init__(self, target_dir):
        super().__init__()
        # FORCE ABSOLUTE PATH to solve "Git Inception" issues
        self.target_path = Path(target_dir).resolve()
        self.repo = None
        self.primary_file = "src/lib.rs" # Default fallback
        self.files_list = []

    def on_mount(self):
        self.title = f"🧬 Evolve Lab: {self.target_path.name}"
        self.load_config()
        self.load_repo()

    def load_config(self):
        """CRITICAL: Reads Evolve.toml to determine what file to show."""
        config_path = self.target_path / "Evolve.toml"
        
        if config_path.exists():
            try:
                # Load the TOML
                data = toml.load(str(config_path))
                
                # Extract file list
                self.files_list = data.get("evolution", {}).get("files", [])
                
                # Check for explicit primary_file setting
                explicit_primary = data.get("evolution", {}).get("primary_file")
                
                if explicit_primary:
                    # Use the explicitly specified primary file
                    self.primary_file = explicit_primary
                    self.notify(f"Config loaded. Tracking: {self.primary_file} (explicit)")
                elif self.files_list:
                    # Use the first file in the list (no language preference)
                    self.primary_file = self.files_list[0]
                    self.notify(f"Config loaded. Tracking: {self.primary_file}")
                else:
                    self.notify("Evolve.toml has no 'files' list!", severity="warning")
            except Exception as e:
                self.notify(f"Error parsing Evolve.toml: {e}", severity="error")
        else:
            self.notify(f"No Evolve.toml found at {config_path}", severity="error")

    def load_repo(self):
        try:
            self.repo = Repo(self.target_path)
            self.refresh_history()
        except Exception as e:
            self.query_one(CodeWindow).update(f"Error loading git repo at {self.target_path}:\n{e}")

    def refresh_history(self):
        try:
            self.repo.git.clear_cache()
            commits = list(self.repo.iter_commits())[::-1] 
        except:
            commits = []

        list_view = self.query_one(ListView)
        list_view.clear()
        
        for i, commit in enumerate(commits):
            list_view.append(GenerationItem(commit, i))
            
        if commits:
            list_view.index = len(commits) - 1
            self.show_commit(commits[-1], len(commits)-1)

    def compose(self) -> ComposeResult:
        yield Header()
        yield Container(ListView(id="sidebar"), id="sidebar-container")
        yield VerticalScroll(CodeWindow(id="code-view"), id="code-container")
        yield Footer()

    def on_list_view_selected(self, event: ListView.Selected):
        item = event.item
        commit = self.repo.commit(item.commit_hex)
        self.show_commit(commit, item.generation_num)

    def show_commit(self, commit, gen_num):
        try:
            # Look for the dynamically selected file in the tree
            target_blob = commit.tree / self.primary_file
            content = target_blob.data_stream.read().decode('utf-8')
            self.query_one(CodeWindow).show_code(content, f"Gen {gen_num}", self.primary_file)
        except KeyError:
            self.query_one(CodeWindow).update(f"// File '{self.primary_file}' not found in Generation {gen_num}")
            self.query_one(CodeWindow).parent.border_title = f"{self.primary_file} (Missing)"
        except Exception as e:
            self.query_one(CodeWindow).update(f"Error reading file: {e}")

    def action_refresh_repo(self):
        self.load_config() # Reload config in case you changed Evolve.toml mid-run
        self.refresh_history()
        self.notify("Repository & Config Refreshed")

if __name__ == "__main__":
    parser = argparse.ArgumentParser(description="Evolve Dashboard")
    parser.add_argument("--target", required=True, help="Path to the target repository")
    args = parser.parse_args()

    # Ensure toml is installed
    try:
        import toml
    except ImportError:
        print("Error: 'toml' library missing. Run: uv add toml")
        exit(1)

    app = EvolutionApp(args.target)
    app.run()
