//! Minimal formatter (~200 tokens) - for recovery after compression

use crate::context::Context;
use super::format_git_status;

/// Minimal format (~200 tokens) - for recovery after compression
pub fn format_minimal(ctx: &Context) -> String {
    let mut out = String::new();

    out.push_str("# Context Recovery (Minimal)\n\n");

    // AI hints (critical for remembering build environment)
    if !ctx.hints.is_empty() {
        out.push_str(&format!("**Hint:** {}\n\n", ctx.hints));
    }

    // Work state is most important for recovery
    if let Some(ws) = &ctx.work_state {
        if !ws.task_summary.is_empty() {
            out.push_str(&format!("**Task:** {}\n", ws.task_summary));
        }
        if !ws.working_files.is_empty() {
            let files: Vec<&str> = ws.working_files.iter().map(|s| s.as_str()).collect();
            out.push_str(&format!("**Files:** {}\n", files.join(", ")));
        }
        if !ws.notes.is_empty() {
            out.push_str(&format!("**Notes:** {}\n", ws.notes));
        }
        out.push('\n');
    }

    // Show only dirty repos
    let dirty_repos: Vec<_> = ctx.git_repos.iter().filter(|r| r.is_dirty).collect();
    if !dirty_repos.is_empty() {
        out.push_str("**Changed repos:** ");
        let repo_strs: Vec<String> = dirty_repos
            .iter()
            .map(|r| format!("{} ({})", r.repo_path, format_git_status(r)))
            .collect();
        out.push_str(&repo_strs.join(", "));
        out.push('\n');
    }

    // Device (one line)
    if !ctx.adb_devices.is_empty() {
        let device = &ctx.adb_devices[0];
        out.push_str(&format!(
            "**Device:** {} ({})\n",
            device.serial, device.device_type
        ));
    }

    out.push_str("\n---\n");
    out.push_str("*Run `get_dev_context` with level=\"normal\" or \"full\" for more details.*\n");

    out
}
