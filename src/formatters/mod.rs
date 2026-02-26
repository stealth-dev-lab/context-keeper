//! Output formatters module

mod minimal;
mod normal;
mod full;

pub use minimal::format_minimal;
pub use normal::format_normal;
pub use full::format_full;

use crate::context::{Context, GitInfo, WorkState};

/// Main formatter dispatcher
pub fn format_context_markdown(ctx: &Context, level: &str) -> String {
    match level {
        "minimal" => format_minimal(ctx),
        "normal" => format_normal(ctx),
        "full" => format_full(ctx),
        _ => format_normal(ctx), // Default to normal
    }
}

/// Helper: format git status string
pub fn format_git_status(git: &GitInfo) -> String {
    if git.is_dirty {
        if git.modified_files > 0 && git.untracked_files > 0 {
            format!("{}M {}U", git.modified_files, git.untracked_files)
        } else if git.modified_files > 0 {
            format!("{}M", git.modified_files)
        } else {
            format!("{}U", git.untracked_files)
        }
    } else {
        "clean".to_string()
    }
}

/// Helper: format work state section
pub fn format_work_state(work_state: &WorkState) -> String {
    let mut out = String::new();
    out.push_str("## Saved Work State\n");
    out.push_str(&format!("- **Saved at:** {}\n", work_state.saved_at));

    if !work_state.task_summary.is_empty() {
        out.push_str(&format!("- **Task:** {}\n", work_state.task_summary));
    }

    if !work_state.working_files.is_empty() {
        out.push_str("- **Working files:**\n");
        for file in &work_state.working_files {
            out.push_str(&format!("  - {}\n", file));
        }
    }

    if !work_state.notes.is_empty() {
        out.push_str(&format!("- **Notes:** {}\n", work_state.notes));
    }

    if !work_state.todos.is_empty() {
        out.push_str("- **Todos:**\n");
        for todo in &work_state.todos {
            let checkbox = match todo.status.as_str() {
                "completed" => "[x]",
                "in_progress" => "[>]",
                _ => "[ ]",
            };
            out.push_str(&format!("  - {} {}\n", checkbox, todo.content));
        }
    }

    out.push('\n');
    out
}
