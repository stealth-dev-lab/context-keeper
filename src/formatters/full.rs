//! Full formatter (~1000 tokens) - complete information

use crate::context::Context;
use super::{format_git_status, format_work_state};

/// Full format (~1000 tokens) - complete information
pub fn format_full(ctx: &Context) -> String {
    let mut out = String::new();

    out.push_str("# Development Context (Full)\n\n");

    // Project info
    if !ctx.project_name.is_empty() {
        out.push_str("## Project\n");
        out.push_str(&format!("- **Name:** {}\n", ctx.project_name));
        if !ctx.project_type.is_empty() {
            out.push_str(&format!("- **Type:** {}\n", ctx.project_type));
        }
        out.push('\n');
    }

    // Work state
    if let Some(ws) = &ctx.work_state {
        out.push_str(&format_work_state(ws));
    }

    // AI Hints
    if !ctx.hints.is_empty() {
        out.push_str("## AI Hints (Important)\n");
        out.push_str(&format!("> {}\n\n", ctx.hints));
    }

    // Build targets
    if !ctx.targets.is_empty() {
        out.push_str("## Available Build Targets\n\n");
        out.push_str("| Target | Description | Container | Lunch Target |\n");
        out.push_str("|--------|-------------|-----------|---------------|\n");
        for target in &ctx.targets {
            out.push_str(&format!(
                "| {} | {} | {} | {} |\n",
                target.name, target.description, target.container_name, target.lunch_target
            ));
        }
        out.push('\n');

        out.push_str("### Target Capabilities\n");
        for target in &ctx.targets {
            let caps: Vec<&str> = [
                if target.can_emulator {
                    Some("emulator")
                } else {
                    None
                },
                if target.can_flash {
                    Some("flash")
                } else {
                    None
                },
            ]
            .into_iter()
            .flatten()
            .collect();

            if !caps.is_empty() {
                out.push_str(&format!("- **{}:** {}\n", target.name, caps.join(", ")));
            }
        }
        out.push('\n');
    }

    // Containers
    if !ctx.containers.is_empty() {
        out.push_str("## Active Containers\n");
        for container in &ctx.containers {
            out.push_str(&format!(
                "- **{}** ({}): {}\n",
                container.name, container.runtime, container.status
            ));
        }
        out.push('\n');
    }

    // Example commands
    if !ctx.available_commands.is_empty() {
        out.push_str("## Example Commands\n");
        out.push_str("```bash\n");
        for cmd in &ctx.available_commands {
            out.push_str(&format!("{}\n", cmd));
        }
        out.push_str("```\n");
    }

    // Command history
    if !ctx.command_history.is_empty() {
        out.push_str("## Recent Relevant Commands\n");
        out.push_str(
            "These commands were executed in previous sessions (useful after context compression):\n\n",
        );
        out.push_str("| Time | Command |\n");
        out.push_str("|------|--------|\n");
        for entry in &ctx.command_history {
            let cmd_display = if entry.command.chars().count() > 80 {
                let truncated: String = entry.command.chars().take(77).collect();
                format!("{}...", truncated)
            } else {
                entry.command.clone()
            };
            let cmd_escaped = cmd_display.replace('|', "\\|");
            out.push_str(&format!("| {} | `{}` |\n", entry.timestamp, cmd_escaped));
        }
        out.push('\n');
    }

    // Git information (ALL repositories)
    if !ctx.git_repos.is_empty() {
        out.push_str("## Git Status\n\n");
        out.push_str("| Repository | Branch | Status | Last Commit |\n");
        out.push_str("|------------|--------|--------|-------------|\n");

        for git in &ctx.git_repos {
            let commit = git.last_commit_short.replace('|', "\\|");
            out.push_str(&format!(
                "| {} | {} | {} | {} |\n",
                git.repo_path,
                git.branch,
                format_git_status(git),
                commit
            ));
        }
        out.push('\n');
    }

    // ADB/Fastboot devices
    if !ctx.adb_devices.is_empty() {
        out.push_str("## Connected Devices\n");
        out.push_str("| Serial | State | Type |\n");
        out.push_str("|--------|-------|------|\n");
        for device in &ctx.adb_devices {
            out.push_str(&format!(
                "| {} | {} | {} |\n",
                device.serial, device.state, device.device_type
            ));
        }
        out.push('\n');
    }

    out
}
