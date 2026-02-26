# ContextKeeper

**AI-Native Development Context Engine** - Helps AI agents remember your build environment after context compression.

## The Problem

When working with AI coding assistants on complex projects:

- AI forgets your build targets after context compression
- You repeatedly explain which container to use
- Environment variables and lunch targets get lost
- "Run this in the container" instructions disappear

## The Solution

ContextKeeper provides a **dynamic, queryable summary** of your development environment via MCP (Model Context Protocol).

Unlike static documentation, ContextKeeper collects **current state** at query time:
- Which containers are actually running right now
- What `lunch` target was used in the last session
- Build targets and their configurations
- Git status across multiple repositories

## Target Users

ContextKeeper is designed for developers working with **complex build environments**:

- **AOSP / Android Platform** - Multiple lunch targets, containerized builds
- **ROS / ROS2** - Workspace configurations, launch files
- **Yocto / Embedded Linux** - BitBake targets, layers
- **Multi-container development** - Docker/Podman based workflows

## Features

| Collector | Type | Description |
|-----------|------|-------------|
| **BuildScript** | Static | Parses config files to extract build targets |
| **Container** | Dynamic | Detects running Podman/Docker containers |
| **History** | Dynamic | Tracks relevant commands via Claude Code Hooks |
| **Git** | Dynamic | Multi-repository status (branch, changes, last commit) |
| **ADB/Fastboot** | Dynamic | Connected Android devices |
| **WorkState** | Persistent | Saves/restores work state across compressions |

### Context Compression Recovery

ContextKeeper provides **hierarchical output levels** to minimize token usage:

| Level | Tokens | Content |
|-------|--------|---------|
| `minimal` | ~200 | Hint + task + working files + dirty repos |
| `normal` | ~400 | + containers + AI hints |
| `full` | ~1000 | Complete information including all repos |

## Quick Start

### 1. Build from Source

**Using Container (Recommended)**

```bash
git clone https://github.com/stealth-dev-lab/context-keeper
cd context-keeper

# Build container
podman build -t context-keeper-build -f Containerfile .

# Build binary
podman run --rm -v $(pwd):/app:Z context-keeper-build cargo build --release

# Copy binary to your PATH
cp target/release/context-keeper ~/.local/bin/
```

> **Note:** Rust nightly is required due to rmcp dependencies. The Containerfile handles this automatically.

### 2. Initialize Your Project

```bash
cd /your/project
context-keeper init
```

### 3. Test Locally

```bash
context-keeper --context          # Normal level
context-keeper --context minimal  # After compression
context-keeper --context full     # Complete details
```

### 4. Setup with Claude Code

Add MCP server to `~/.claude.json`:

```json
{
  "projects": {
    "/your/project": {
      "mcpServers": {
        "context-keeper": {
          "type": "stdio",
          "command": "/path/to/context-keeper",
          "args": [],
          "env": {}
        }
      }
    }
  }
}
```

(Optional) Enable hooks in `~/.claude/settings.json`:

```json
{
  "hooks": {
    "PostToolUse": [
      {
        "matcher": "Bash",
        "hooks": [
          {"type": "command", "command": "/path/to/hooks/log-commands.sh"}
        ]
      },
      {
        "matcher": "TodoWrite",
        "hooks": [
          {"type": "command", "command": "/path/to/hooks/save-todos.sh"}
        ]
      }
    ]
  }
}
```

## Configuration

Create `contextkeeper.toml` in your project root:

```toml
[project]
name = "My Project"
type = "aosp"  # aosp, ros, yocto, custom

[containers]
runtime = "podman"  # or "docker"

[hints]
default = "Build commands must be executed inside the container."

[history]
enabled = true
patterns = [
    "lunch\\s+\\S+",
    "source.*envsetup",
]
max_entries = 20

[git]
auto_detect = true
scan_depth = 2
```

## Project Structure

```
context-keeper/
├── src/
│   ├── main.rs              # Entry point
│   ├── config.rs            # Configuration loading
│   ├── context.rs           # Data structures
│   ├── collectors/          # Context collectors
│   │   ├── build.rs         # Build target parsing
│   │   ├── container.rs     # Docker/Podman detection
│   │   ├── git.rs           # Multi-repo git status
│   │   ├── history.rs       # Command history
│   │   ├── adb.rs           # Android device detection
│   │   └── workstate.rs     # Work state persistence
│   ├── formatters/          # Output formatters
│   │   ├── minimal.rs       # ~200 tokens
│   │   ├── normal.rs        # ~400 tokens
│   │   └── full.rs          # ~1000 tokens
│   ├── mcp/                 # MCP server
│   │   └── tools.rs         # Tool implementations
│   └── cli/                 # CLI commands
│       ├── init.rs          # Setup wizard
│       └── context.rs       # Context output
├── hooks/                   # Claude Code hooks
├── Containerfile            # Build environment
└── Cargo.toml
```

## MCP Tools

| Tool | Description |
|------|-------------|
| `get_dev_context(level)` | Returns development context. Level: `minimal`, `normal` (default), `full` |
| `save_work_state(...)` | Save current work state for recovery after compression |

## CLI Usage

```bash
# Initialize project (interactive wizard)
context-keeper init

# Output context as Markdown
context-keeper --context          # Normal level
context-keeper --context minimal  # Minimal level
context-keeper --context full     # Full level

# Save work state (for PreCompact hook)
context-keeper --save-state "Current task description"

# Run as MCP server (default)
context-keeper
```

## Hooks

| Hook | Purpose |
|------|---------|
| `log-commands.sh` | Captures relevant Bash commands |
| `save-todos.sh` | Saves todos for recovery |
| `track-files.sh` | Tracks edited files |
| `pre-compact-save.sh` | Saves state before compression |
| `post-compact-reminder.sh` | Reminds to restore context |

## License

MIT

## Contributing

Contributions welcome! Please open an issue first to discuss changes.

---

Part of [stealth-dev-lab](https://github.com/stealth-dev-lab).
