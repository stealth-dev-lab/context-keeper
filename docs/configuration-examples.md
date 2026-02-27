# Configuration Examples

ContextKeeper の設定ファイル `contextkeeper.toml` の詳細と、プロジェクトタイプ別の設定例です。

## 設定ファイルの場所

以下の順序で検索されます（最初に見つかったものを使用）:

1. `contextkeeper.toml`
2. `context-keeper.toml`
3. `.contextkeeper.toml`

## 設定項目リファレンス

### [project] - プロジェクト基本情報

```toml
[project]
name = "My Project"     # プロジェクト名（出力に表示）
type = "aosp"           # プロジェクトタイプ: aosp, ros, yocto, custom
```

`type` の効果:
- `aosp`: ADB collector が自動有効化
- その他: 特別な動作なし（将来拡張予定）

### [containers] - コンテナランタイム設定

```toml
[containers]
runtime = "podman"      # "podman" または "docker"
```

実行中のコンテナを検出するために使用されます。

### [hints] - AI へのヒント

```toml
[hints]
default = "Build commands must be executed inside the container."
```

AI エージェントに伝えたい重要な情報を記述します。
この情報は全ての出力レベル（minimal/normal/full）で表示されます。

**使用例**:
- 「ビルドはコンテナ内で実行」
- 「main ブランチへの直接 push 禁止」
- 「テストは必ず X コマンドで実行」

### [scripts] - ビルドスクリプト設定

```toml
[scripts]
config_dir = "./build-configs"    # 設定ファイルのディレクトリ
config_pattern = "*.conf"         # ファイルパターン
entry_point = "./scripts/build.sh" # エントリポイントスクリプト
```

`config_dir` 内の `.conf` ファイルから以下の変数を抽出:
- `TARGET_NAME` - ターゲット名
- `TARGET_DESCRIPTION` - 説明
- `CONTAINER_NAME` - 使用するコンテナ
- `LUNCH_TARGET` - AOSP lunch ターゲット
- `CAN_EMULATOR` - エミュレータ対応 (true/false)
- `CAN_FLASH` - 実機フラッシュ対応 (true/false)

### [history] - コマンド履歴設定

```toml
[history]
enabled = true                              # 履歴収集の有効/無効
log_file = "~/.contextkeeper/command-history.jsonl"  # ログファイルパス
max_entries = 20                            # 表示する最大エントリ数
patterns = [                                # 収集対象のコマンドパターン（正規表現）
    "lunch\\s+\\S+",
    "source.*envsetup",
    "export\\s+\\w+=",
]
```

履歴は Claude Code Hooks 経由で収集されます（後述）。

### [git] - Git リポジトリ設定

```toml
[git]
auto_detect = true      # サブディレクトリの自動検出
scan_depth = 2          # 検索する深さ（デフォルト: 2）
paths = [               # 明示的なパス指定（auto_detect と併用可）
    "subproject-a",
    "libs/core",
]
```

### [adb] - Android デバイス設定

```toml
[adb]
enabled = true          # ADB/Fastboot デバイス検出の有効/無効
```

`project.type = "aosp"` の場合は自動で有効化されます。

---

## プロジェクトタイプ別の設定例

### AOSP / Android Platform 開発

```toml
# contextkeeper.toml - AOSP Project

[project]
name = "AOSP Development"
type = "aosp"

[containers]
runtime = "podman"

[hints]
default = """
Build commands must run inside the aosp-build container.
Use 'podman exec -it aosp-build bash' to enter the container.
After entering, run 'source build/envsetup.sh' and 'lunch <target>'.
"""

[scripts]
config_dir = "./build-configs"
config_pattern = "*.conf"

[history]
enabled = true
patterns = [
    "lunch\\s+\\S+",
    "source.*envsetup",
    "^m\\s",
    "^mm\\b",
    "^mma\\b",
    "adb\\s+(shell|push|pull|install)",
    "fastboot\\s+flash",
]
max_entries = 30

[git]
auto_detect = false     # AOSP は巨大なので手動指定推奨
paths = [
    "packages/apps/MyApp",
    "frameworks/base",
]

[adb]
enabled = true          # type = "aosp" なら省略可
```

**ビルド設定ファイル例** (`build-configs/pixel7a.conf`):

```bash
TARGET_NAME="pixel7a"
TARGET_DESCRIPTION="Pixel 7a development build"
CONTAINER_NAME="aosp-build"
LUNCH_TARGET="aosp_lynx-trunk_staging-userdebug"
CAN_EMULATOR="false"
CAN_FLASH="true"
```

### ROS / ROS2 開発

```toml
# contextkeeper.toml - ROS2 Project

[project]
name = "Robot Navigation"
type = "ros"

[containers]
runtime = "docker"

[hints]
default = """
Source the workspace before building: 'source install/setup.bash'
Launch files are in src/*/launch/
Use 'colcon build --symlink-install' for development.
"""

[history]
enabled = true
patterns = [
    "colcon\\s+build",
    "ros2\\s+(launch|run|topic|service)",
    "source.*setup\\.bash",
]
max_entries = 20

[git]
auto_detect = true
scan_depth = 2

[adb]
enabled = false
```

### Yocto / Embedded Linux 開発

```toml
# contextkeeper.toml - Yocto Project

[project]
name = "Embedded Linux"
type = "yocto"

[containers]
runtime = "podman"

[hints]
default = """
Initialize build environment: 'source oe-init-build-env'
Build with: 'bitbake core-image-minimal'
Layers are in meta-*/
"""

[scripts]
config_dir = "./build-configs"
config_pattern = "*.conf"

[history]
enabled = true
patterns = [
    "bitbake\\s+\\S+",
    "source.*oe-init-build-env",
    "devtool\\s+\\S+",
]
max_entries = 20

[git]
auto_detect = true
scan_depth = 1          # meta-* レイヤーは浅い階層

[adb]
enabled = false
```

### マルチリポジトリワークスペース

```toml
# contextkeeper.toml - Multi-repo Workspace (like stealth-dev-lab)

[project]
name = "stealth-dev-lab"
type = "custom"

[containers]
runtime = "podman"

[hints]
default = """
This is a multi-project workspace. Each subdirectory is a separate repository.
- context-keeper: MCP server for AI context
- nix-zen: Nix-based development environment
- stealth-gateway: Tailscale-based remote access
"""

[git]
auto_detect = true
scan_depth = 1          # 各サブディレクトリが独立したリポジトリ

[history]
enabled = true
patterns = [
    "git\\s+(push|pull|commit|checkout)",
    "cargo\\s+(build|test|run)",
    "podman\\s+(build|run|exec)",
    "nix\\s+\\S+",
    "home-manager\\s+switch",
]
max_entries = 30

[adb]
enabled = false
```

### シンプルな Rust プロジェクト

```toml
# contextkeeper.toml - Simple Rust Project

[project]
name = "My Rust App"
type = "custom"

[hints]
default = "Run tests with 'cargo test'. Build with 'cargo build --release'."

[git]
auto_detect = true
scan_depth = 1

[history]
enabled = true
patterns = [
    "cargo\\s+(build|test|run|clippy|fmt)",
]
max_entries = 10

# containers と adb は使わないので省略
```

---

## Claude Code Hooks との連携

ContextKeeper は Claude Code Hooks と連携してコマンド履歴や Todo を自動収集できます。

### フックのセットアップ

`~/.claude/settings.json` に以下を追加:

```json
{
  "hooks": {
    "PostToolUse": [
      {
        "matcher": "Bash",
        "hooks": [
          {
            "type": "command",
            "command": "/path/to/hooks/log-commands.sh"
          }
        ]
      },
      {
        "matcher": "TodoWrite",
        "hooks": [
          {
            "type": "command",
            "command": "/path/to/hooks/save-todos.sh"
          }
        ]
      }
    ],
    "PreCompact": [
      {
        "hooks": [
          {
            "type": "command",
            "command": "context-keeper --save-state 'Auto-saved before compression'"
          }
        ]
      }
    ]
  }
}
```

### log-commands.sh

```bash
#!/bin/bash
# Bash コマンドを履歴に記録
HISTORY_FILE="$HOME/.contextkeeper/command-history.jsonl"
mkdir -p "$(dirname "$HISTORY_FILE")"

# stdin から tool_input を読み取り
INPUT=$(cat)
COMMAND=$(echo "$INPUT" | jq -r '.tool_input.command // empty')

if [ -n "$COMMAND" ]; then
    TIMESTAMP=$(date -Iseconds)
    echo "{\"timestamp\": \"$TIMESTAMP\", \"command\": \"$COMMAND\"}" >> "$HISTORY_FILE"
fi
```

### save-todos.sh

```bash
#!/bin/bash
# TodoWrite の内容を保存
TODOS_FILE="$HOME/.contextkeeper/current-todos.json"
mkdir -p "$(dirname "$TODOS_FILE")"

# stdin から tool_input を読み取り、そのまま保存
cat > "$TODOS_FILE"
```

---

## 出力レベルと表示内容

| レベル | トークン数 | 表示内容 |
|--------|-----------|----------|
| `minimal` | ~200 | hints, task, working files, dirty repos |
| `normal` | ~400 | + containers, AI hints |
| `full` | ~1000 | + all git repos, command history, devices |

コンテキスト圧縮後は `minimal` で復帰し、必要に応じて `normal` や `full` を使用します。

---

## トラブルシューティング

### 設定が読み込まれない

```bash
# 設定ファイルの存在確認
ls -la contextkeeper.toml context-keeper.toml .contextkeeper.toml

# TOML 構文エラーの確認
cat contextkeeper.toml | python3 -c "import sys, tomllib; tomllib.loads(sys.stdin.read())"
```

### コンテナが検出されない

```bash
# runtime が正しいか確認
podman ps  # または docker ps

# contextkeeper.toml の runtime 設定を確認
grep runtime contextkeeper.toml
```

### 履歴が収集されない

1. `~/.contextkeeper/command-history.jsonl` が存在するか確認
2. Claude Code Hooks が正しく設定されているか確認
3. `history.patterns` がコマンドにマッチしているか確認
