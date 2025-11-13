# fini

A fast, interactive CLI todo list tool written in Rust. Track tasks with optional links (URLs, issue trackers) and manage them efficiently from your terminal.

## Features

- **Task Management**: Create, edit, complete, and delete tasks
- **Status Tracking**: Three states - Todo, In Progress, and Done
- **Link Support**: Attach URLs or issue tracker links to tasks
- **Interactive Mode**: Full-featured TUI for managing tasks
- **Archive System**: Archive completed tasks
- **Copy to Clipboard**: Copy tasks (with links) to clipboard

## Installation

### From Source

```bash
git clone https://github.com/sirbrillig/fini
cd fini
cargo build --release
```

The binary will be available at `target/release/fini`. You can add it to your PATH or install it with:

```bash
cargo install --path .
```

## Usage

### Add Tasks

```bash
# Add a task (will prompt for optional link)
fini add "Implement user authentication"
fini a Fix bug in parser # or use the alias
```

### List Tasks

```bash
# Show all current tasks
fini list
fini l  # using alias
```

Output format:
```
  1. ☐ Task title
  2. … Task in progress https://github.com/user/repo/issues/123
  3. ✔ Completed task
```

### Work on Tasks

```bash
# Mark task(s) as in-progress (toggles with Todo)
fini work 1 2 3
fini w 1       # using alias
fini begin 1   # alternative alias
fini b 1       # short alias
```

### Complete Tasks

```bash
# Mark task(s) as done (toggles with Todo)
fini done 1 2
fini check 1  # alternative alias
fini c 1      # short alias
```

### Edit Tasks

```bash
# Edit task title and optionally the link
fini edit 2
fini e 2  # using alias
```

Opens your default editor (via `$EDITOR` or `$VISUAL`) to modify the task.

### Delete Tasks

```bash
# Permanently delete a task
fini delete 1
fini d 1  # using alias
```

### Copy Tasks

```bash
# Copy task(s) to clipboard (includes links)
fini copy 1 2 3
fini y 1 2     # using alias
fini yank 1    # alternative alias

# Copy tasks completed on a specific date
fini date 2024-12-15
```

### Archive Management

```bash
# Archive all completed tasks (and copy in-progress tasks)
# In-progress tasks are archived as a snapshot but remain active as Todo
fini clear

# View all archived tasks
fini archived

# Delete all archived tasks permanently
fini cycle
```

### Interactive Mode

```bash
# Enter interactive mode with a menu-driven interface
fini interactive
fini i  # using alias
```

Interactive mode provides:
- Visual task selection
- Multi-select for batch operations
- Confirmation prompts for destructive actions
- Continuous workflow without re-running commands

## Data Storage

Tasks are stored in JSON format at a platform-specific location:

- **Linux**: `~/.local/share/fini_data.json`
- **macOS**: `~/Library/Application Support/fini_data.json`
- **Windows**: `%APPDATA%\fini_data.json`

The data file is human-readable and can be backed up or version controlled.

## Development

### Prerequisites

- Rust 1.70+ (uses 2024 edition)

### Building

```bash
cargo build           # Debug build
cargo build --release # Optimized release build
```

### Testing

```bash
cargo test                    # Run all tests
cargo test <test_name>        # Run specific test
cargo test -- --nocapture     # Show output during tests
```

### Running

```bash
# Run directly with cargo
cargo run -- list
cargo run -- add "New task"
cargo run -- work 1
```

## License

MIT License - see the [LICENSE](LICENSE) file for details.
