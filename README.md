# fini

A fast, interactive CLI todo list tool written in Rust. Track tasks with optional links (URLs, issue trackers) and manage them efficiently from your terminal.

## Features

- **Task Management**: Create, edit, complete, and delete tasks
- **Status Tracking**: Three states - Todo, In Progress, and Done
- **Link Support**: Attach URLs or issue tracker links to tasks
- **Interactive Mode**: Full-featured TUI for managing tasks
- **Archive System**: Archive completed tasks with `clear`
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

Running `fini` with no arguments launches interactive mode — this is the primary way to use the tool.

```bash
fini
```

All other subcommands are available for scripting or quick one-off operations from the command line.

### Add Tasks

```bash
# Add a task (will prompt for optional link)
fini add "Implement user authentication"
fini a "Fix bug in parser"  # using alias
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
fini begin 1 2 3
fini b 1       # using alias
```

### Complete Tasks

```bash
# Mark task(s) as done (toggles with Todo)
fini check 1 2
fini c 1      # using alias
```

### Star Tasks

```bash
# Star or un-star task(s) to mark as important
fini star 1 2 3
fini s 1       # using alias
```

### Edit Tasks

```bash
# Edit a task's title and link via interactive prompts
fini edit 2
fini e 2  # using alias
```

### Open Task Links

```bash
# Open a task's link in your web browser
fini open 1
fini o 1  # using alias
```

### Delete Tasks

```bash
# Permanently delete task(s)
fini delete 1
fini d 1  # using alias
```

### Copy Tasks

```bash
# Copy task(s) to clipboard (includes links)
fini copy 1 2 3
fini y 1 2     # using alias
fini yank 1    # alternative alias

# Copy task(s) to clipboard as Markdown links
fini copy-markdown 1 2 3

# Copy all completed, begun, or archived tasks on or after a date
fini copy-after-date 2024-12-15
```

### Archive Management

```bash
# Archive all done tasks (also archives a snapshot of in-progress tasks,
# which remain active as Todo)
fini clear

# View all archived tasks
fini archived

# List all completed, begun, or archived tasks on or after a date
fini list-after-date 2024-12-15
```

### Interactive Mode

```bash
# Enter interactive mode (default when no command is given)
fini
fini interactive
fini i  # using alias
```

Interactive mode provides:
- Visual task selection
- Multi-select for batch operations
- Confirmation prompts for destructive actions
- Continuous workflow without re-running commands

## Data Storage

Tasks are stored as Markdown files in a platform-specific directory:

- **Linux**: `~/.local/share/fini/tasks/`
- **macOS**: `~/Library/Application Support/fini/tasks/`
- **Windows**: `%APPDATA%\fini\tasks\`

Active tasks are stored in `tasks.md`. Archived tasks are stored in monthly files named `archived-YYYY-MM.md`, one per calendar month.

All files are human-readable and can be backed up or version controlled.

### Active tasks file format (`tasks.md`)

fini uses a custom Markdown-adjacent format for task status markers. It extends standard `- [ ]` checkbox syntax with additional status values:

| Marker   | Status      |
|----------|-------------|
| `- [ ]`  | Todo        |
| `- [~]`  | In Progress |
| `- [x]`  | Done        |

A `⭐️ ` prefix after the marker indicates a starred task. Links use standard Markdown link syntax with the title as the link text.

Tasks are grouped under `## YYYY-MM-DD` date headers (reflecting when they became active or done). Undated active tasks appear under an `## Active Tasks` header at the top.

```markdown
## Active Tasks
- [ ] A plain todo task
- [~] ⭐️ A starred in-progress task
- [~] [Task linked to an issue](https://github.com/user/repo/issues/1)
- [x] A completed task

## 2026-03-01
- [x] A task completed on this date
```

### Archive file format (`archived-YYYY-MM.md`)

Archive files use plain Markdown lists grouped under `## YYYY-MM-DD` date headers. All entries in an archive file are implicitly archived, so no status marker is used. Links use standard Markdown link syntax.

```markdown
## 2026-02-14
- A plain archived task
- [Task with a link](https://github.com/user/repo/issues/2)

## 2026-02-28
- Another archived task
```

## Configuration

fini can be configured by creating a TOML file named `fini_config.toml` in the data directory:

- **Linux**: `~/.local/share/fini/fini_config.toml`
- **macOS**: `~/Library/Application Support/fini/fini_config.toml`
- **Windows**: `%APPDATA%\fini\fini_config.toml`

All settings are optional and fall back to defaults if omitted.

### `list_link_format`

Controls how task links are displayed by `fini list`. Default: `newline`.

| Value           | Description                                              |
|-----------------|----------------------------------------------------------|
| `Newline`       | Link printed on a new line below the task title          |
| `Adjacent`      | Link printed after the task title on the same line       |
| `AdjacentColor` | Same as `Adjacent` but link is dimmed                    |
| `Hyperlink`     | Link rendered as an OSC 8 terminal hyperlink             |
| `Markdown`      | Task title becomes a Markdown link: `[title](url)`       |
| `None`          | Links not shown                                          |

### `prompter`

Controls the input method used for interactive prompts. Default: `inquire`.

| Value     | Description                        |
|-----------|------------------------------------|
| `inquire` | Default interactive prompt UI      |
| `vim`     | Vim-style line editing             |

### Example config

```toml
list_link_format = "Hyperlink"
prompter = "vim"
```

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
cargo run -- begin 1
```

## License

MIT License - see the [LICENSE](LICENSE) file for details.
