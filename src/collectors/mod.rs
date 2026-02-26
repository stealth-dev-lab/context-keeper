//! Context collectors module

mod build;
mod container;
mod git;
mod history;
mod adb;
mod workstate;

pub use build::*;
pub use container::*;
pub use git::*;
pub use history::*;
pub use adb::*;
pub use workstate::*;

use crate::config::Config;
use crate::context::Context;

/// Collect all context data
pub fn collect_context(config: &Config) -> Context {
    let mut ctx = Context::default();

    if let Some(project) = &config.project {
        ctx.project_name = project.name.clone().unwrap_or_default();
        ctx.project_type = project.project_type.clone().unwrap_or_default();
    }

    ctx.targets = collect_build_targets(config);
    ctx.containers = collect_containers(config);

    if let Some(scripts) = &config.scripts {
        if let Some(entry) = &scripts.entry_point {
            ctx.available_commands = parse_entry_point_commands(entry);
        }
    }

    if let Some(hints) = &config.hints {
        ctx.hints = hints.default.clone().unwrap_or_default();
    }

    ctx.command_history = collect_command_history(config);
    ctx.git_repos = collect_git_repos(config);
    ctx.adb_devices = collect_adb_devices();
    ctx.work_state = load_work_state_with_hooks();

    ctx
}
