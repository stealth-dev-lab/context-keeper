//! Normal formatter (~400 tokens) - balanced info

use crate::context::Context;
use super::{format_git_status, format_work_state};

/// Normal format (~400 tokens) - balanced info
pub fn format_normal(ctx: &Context) -> String {
    let mut out = String::new();

    out.push_str("# Development Context\n\n");

    // Work state
    if let Some(ws) = &ctx.work_state {
        out.push_str(&format_work_state(ws));
    }

    // AI Hints
    if !ctx.hints.is_empty() {
        out.push_str("## AI Hints\n");
        out.push_str(&format!("> {}\n\n", ctx.hints));
    }

    // Git Status (dirty repos only)
    let dirty_repos: Vec<_> = ctx.git_repos.iter().filter(|r| r.is_dirty).collect();
    if !dirty_repos.is_empty() {
        out.push_str("## Git Status (changes only)\n\n");
        out.push_str("| Repository | Branch | Status |\n");
        out.push_str("|------------|--------|--------|\n");
        for git in dirty_repos {
            out.push_str(&format!(
                "| {} | {} | {} |\n",
                git.repo_path,
                git.branch,
                format_git_status(git)
            ));
        }
        out.push('\n');
    }

    // Active containers
    if !ctx.containers.is_empty() {
        out.push_str("## Active Containers\n");
        for container in &ctx.containers {
            out.push_str(&format!("- {} ({})\n", container.name, container.status));
        }
        out.push('\n');
    }

    // Connected devices
    if !ctx.adb_devices.is_empty() {
        out.push_str("## Connected Devices\n");
        for device in &ctx.adb_devices {
            out.push_str(&format!(
                "- {} ({}, {})\n",
                device.serial, device.state, device.device_type
            ));
        }
        out.push('\n');
    }

    out.push_str("---\n");
    out.push_str("*Run `get_dev_context` with level=\"full\" for complete information.*\n");

    out
}
