# ContextKeeper Architecture

## Overview

ContextKeeper は MCP (Model Context Protocol) サーバーとして動作し、AI エージェントに開発環境のコンテキストを提供します。

## System Architecture

```mermaid
graph TB
    subgraph "Claude Code"
        CC[Claude Code CLI]
        HOOKS[Hooks System]
    end

    subgraph "ContextKeeper"
        MCP[MCP Server]
        CLI[CLI Interface]

        subgraph "Collectors"
            BUILD[BuildScript]
            CONTAINER[Container]
            GIT[Git]
            HISTORY[History]
            ADB[ADB/Fastboot]
            WORKSTATE[WorkState]
        end

        subgraph "Formatters"
            MINIMAL[Minimal ~200 tokens]
            NORMAL[Normal ~400 tokens]
            FULL[Full ~1000 tokens]
        end

        CONFIG[Config Reader]
        CONTEXT[Context Aggregator]
    end

    subgraph "External Systems"
        PODMAN[Podman/Docker]
        GITCMD[Git Command]
        ADBCMD[ADB/Fastboot]
        FS[File System]
    end

    CC <-->|MCP Protocol| MCP
    HOOKS -->|Shell Scripts| CLI

    MCP --> CONFIG
    MCP --> CONTEXT
    CLI --> CONFIG
    CLI --> CONTEXT

    CONFIG --> FS
    CONTEXT --> BUILD
    CONTEXT --> CONTAINER
    CONTEXT --> GIT
    CONTEXT --> HISTORY
    CONTEXT --> ADB
    CONTEXT --> WORKSTATE

    BUILD --> FS
    CONTAINER --> PODMAN
    GIT --> GITCMD
    HISTORY --> FS
    ADB --> ADBCMD
    WORKSTATE --> FS

    CONTEXT --> MINIMAL
    CONTEXT --> NORMAL
    CONTEXT --> FULL
```

## Data Flow

### 1. Context Collection Flow

```mermaid
sequenceDiagram
    participant CC as Claude Code
    participant MCP as MCP Server
    participant CFG as Config
    participant CTX as Context
    participant COL as Collectors
    participant FMT as Formatter

    CC->>MCP: get_dev_context(level="normal")
    MCP->>CFG: read_config()
    CFG-->>MCP: Config

    MCP->>CTX: collect_context(config)

    par Parallel Collection
        CTX->>COL: collect_build_targets()
        CTX->>COL: collect_containers()
        CTX->>COL: collect_git_repos()
        CTX->>COL: collect_command_history()
        CTX->>COL: collect_adb_devices()
        CTX->>COL: load_work_state()
    end

    COL-->>CTX: Context Data

    MCP->>FMT: format_context_markdown(ctx, "normal")
    FMT-->>MCP: Markdown String

    MCP-->>CC: CallToolResult
```

### 2. Work State Save Flow

```mermaid
sequenceDiagram
    participant CC as Claude Code
    participant MCP as MCP Server
    participant WS as WorkState
    participant FS as File System

    CC->>MCP: save_work_state(task_summary, files, notes)

    alt files not provided
        MCP->>WS: collect_working_files()
        WS->>FS: git diff (find modified files)
        FS-->>WS: file list
    end

    MCP->>WS: save_work_state_to_file(state)
    WS->>FS: write ~/.contextkeeper/work-state.json

    MCP-->>CC: "Work state saved successfully"
```

### 3. Hook Integration Flow

```mermaid
sequenceDiagram
    participant CC as Claude Code
    participant HOOK as Hook Script
    participant FS as File System
    participant CK as ContextKeeper

    Note over CC: User executes Bash command
    CC->>HOOK: PostToolUse (Bash)
    HOOK->>FS: Append to command-history.jsonl

    Note over CC: User updates TodoWrite
    CC->>HOOK: PostToolUse (TodoWrite)
    HOOK->>FS: Write current-todos.json

    Note over CC: Context compression imminent
    CC->>HOOK: PreCompact
    HOOK->>CK: context-keeper --save-state
    CK->>FS: Write work-state.json

    Note over CC: After compression
    CC->>CK: get_dev_context("minimal")
    CK->>FS: Read work-state.json
    CK-->>CC: Recovered context
```

## Module Structure

```mermaid
graph LR
    subgraph "Entry Points"
        MAIN[main.rs]
    end

    subgraph "Core"
        CONFIG[config.rs]
        CONTEXT[context.rs]
    end

    subgraph "collectors/"
        MOD_C[mod.rs]
        BUILD[build.rs]
        CONTAINER[container.rs]
        GIT[git.rs]
        HISTORY[history.rs]
        ADB[adb.rs]
        WORKSTATE[workstate.rs]
    end

    subgraph "formatters/"
        MOD_F[mod.rs]
        MINIMAL[minimal.rs]
        NORMAL[normal.rs]
        FULL[full.rs]
    end

    subgraph "mcp/"
        MOD_M[mod.rs]
        TOOLS[tools.rs]
    end

    subgraph "cli/"
        MOD_CLI[mod.rs]
        INIT[init.rs]
        CTX_CLI[context.rs]
    end

    MAIN --> CONFIG
    MAIN --> MOD_C
    MAIN --> MOD_F
    MAIN --> MOD_M
    MAIN --> MOD_CLI

    MOD_C --> BUILD
    MOD_C --> CONTAINER
    MOD_C --> GIT
    MOD_C --> HISTORY
    MOD_C --> ADB
    MOD_C --> WORKSTATE

    MOD_F --> MINIMAL
    MOD_F --> NORMAL
    MOD_F --> FULL

    MOD_M --> TOOLS
    MOD_CLI --> INIT
    MOD_CLI --> CTX_CLI

    TOOLS --> CONFIG
    TOOLS --> MOD_C
    TOOLS --> MOD_F
```

## Collector Details

### BuildScript Collector

```mermaid
flowchart TD
    START[Start] --> CHECK{config.scripts exists?}
    CHECK -->|No| EMPTY[Return empty]
    CHECK -->|Yes| DIR[Get config_dir]
    DIR --> GLOB[Glob pattern matching]
    GLOB --> PARSE[Parse each .conf file]
    PARSE --> EXTRACT[Extract variables]
    EXTRACT --> TARGET[Create BuildTarget]
    TARGET --> COLLECT[Collect all targets]
    COLLECT --> RETURN[Return Vec<BuildTarget>]
```

**Parsed Variables:**
- `TARGET_NAME`
- `TARGET_DESCRIPTION`
- `CONTAINER_NAME`
- `LUNCH_TARGET`
- `CAN_EMULATOR`
- `CAN_FLASH`

### Container Collector

```mermaid
flowchart TD
    START[Start] --> RUNTIME{Get runtime}
    RUNTIME -->|podman| PODMAN[podman ps --format]
    RUNTIME -->|docker| DOCKER[docker ps --format]
    PODMAN --> PARSE[Parse output]
    DOCKER --> PARSE
    PARSE --> INFO[Create ContainerInfo]
    INFO --> RETURN[Return Vec<ContainerInfo>]
```

### Git Collector

```mermaid
flowchart TD
    START[Start] --> ROOT{Is root a git repo?}
    ROOT -->|Yes| SINGLE[Collect single repo info]
    ROOT -->|No| SCAN[Scan subdirectories]
    SCAN --> FIND[Find .git directories]
    FIND --> EACH[For each repo]
    EACH --> BRANCH[git branch --show-current]
    EACH --> STATUS[git status --porcelain]
    EACH --> LOG[git log -1]
    BRANCH --> INFO[Create GitInfo]
    STATUS --> INFO
    LOG --> INFO
    INFO --> RETURN[Return Vec<GitInfo>]
    SINGLE --> RETURN
```

## Output Format Levels

### Minimal (~200 tokens)

圧縮後の復帰用。必要最小限の情報のみ。

```markdown
# Context Recovery (Minimal)

**Hint:** Build commands must run inside container.
**Task:** Implementing feature X
**Files:** src/main.rs, src/lib.rs
**Changed repos:** project (2M)
**Device:** ABC123 (adb)

---
*Run `get_dev_context` with level="normal" for more details.*
```

### Normal (~400 tokens)

通常使用。バランスの取れた情報量。

```markdown
# Development Context

## Saved Work State
- **Task:** Implementing feature X
- **Working files:** src/main.rs, src/lib.rs

## AI Hints
> Build commands must run inside container.

## Git Status (changes only)
| Repository | Branch | Status |
|------------|--------|--------|
| project | main | 2M |

## Active Containers
- aosp-build (Up 3 hours)
```

### Full (~1000 tokens)

完全な情報。デバッグやセットアップ確認用。

```markdown
# Development Context (Full)

## Project
- **Name:** My Project
- **Type:** aosp

## Saved Work State
...

## AI Hints (Important)
...

## Available Build Targets
| Target | Description | Container | Lunch Target |
...

## Active Containers
...

## Recent Relevant Commands
| Time | Command |
...

## Git Status
| Repository | Branch | Status | Last Commit |
...

## Connected Devices
| Serial | State | Type |
...
```

## File Storage

```
~/.contextkeeper/
├── command-history.jsonl    # Captured commands (by log-commands.sh)
├── current-todos.json       # Current todos (by save-todos.sh)
├── recent-files.json        # Recently edited files (by track-files.sh)
└── work-state.json          # Saved work state
```

### command-history.jsonl

```json
{"timestamp": "2026-02-27T10:30:00Z", "command": "lunch sdk_car_dev-trunk_staging-userdebug"}
{"timestamp": "2026-02-27T10:31:00Z", "command": "source build/envsetup.sh"}
```

### work-state.json

```json
{
  "saved_at": "2026-02-27T10:30:00Z",
  "trigger": "manual",
  "task_summary": "Implementing rate limiter",
  "working_files": ["src/main.rs", "src/lib.rs"],
  "notes": "Token bucket implementation in progress",
  "todos": [
    {"content": "Add unit tests", "status": "pending"},
    {"content": "Update documentation", "status": "in_progress"}
  ]
}
```

## MCP Protocol

ContextKeeper は [Model Context Protocol](https://modelcontextprotocol.io/) を実装しています。

### Tools

| Tool | Parameters | Description |
|------|------------|-------------|
| `get_dev_context` | `level?: "minimal" \| "normal" \| "full"` | 開発コンテキストを取得 |
| `save_work_state` | `task_summary: string, working_files?: string[], notes?: string, todos?: string` | 作業状態を保存 |

### Transport

- **Type:** stdio
- **Protocol:** JSON-RPC 2.0 over stdin/stdout
