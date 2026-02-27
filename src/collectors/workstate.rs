//! Work state collector - saves and loads work state for context recovery

use super::traits::Collector;
use crate::config::Config;
use crate::context::{Context, TodoItem, WorkState};
use std::fs;
use std::io::{self, Write};
use std::path::Path;

/// Work state collector
#[derive(Debug, Default)]
pub struct WorkStateCollector;

impl Collector for WorkStateCollector {
    fn name(&self) -> &'static str {
        "workstate"
    }

    fn is_enabled(&self, _config: &Config) -> bool {
        // Always enabled - core feature for context recovery
        true
    }

    fn collect(&self, _config: &Config, ctx: &mut Context) {
        ctx.work_state = load_work_state_with_hooks();
    }
}

/// Get the path to the work state file
pub fn get_work_state_path() -> String {
    let home = std::env::var("HOME").unwrap_or_else(|_| ".".to_string());
    format!("{}/.contextkeeper/work-state.json", home)
}

/// Ensure the contextkeeper directory exists
pub fn ensure_contextkeeper_dir() -> io::Result<()> {
    let home = std::env::var("HOME").unwrap_or_else(|_| ".".to_string());
    let dir = format!("{}/.contextkeeper", home);
    fs::create_dir_all(&dir)?;
    Ok(())
}

/// Save work state to file
pub fn save_work_state_to_file(state: &WorkState) -> io::Result<()> {
    ensure_contextkeeper_dir()?;
    let path = get_work_state_path();
    let json = serde_json::to_string_pretty(state).map_err(io::Error::other)?;
    let mut file = fs::File::create(&path)?;
    file.write_all(json.as_bytes())?;
    Ok(())
}

/// Load work state from file
fn load_work_state_from_file() -> Option<WorkState> {
    let path = get_work_state_path();
    if !Path::new(&path).exists() {
        return None;
    }

    fs::read_to_string(&path)
        .ok()
        .and_then(|content| serde_json::from_str(&content).ok())
}

/// Load saved todos from TodoWrite hook
fn load_saved_todos() -> Vec<TodoItem> {
    let home = std::env::var("HOME").unwrap_or_else(|_| ".".to_string());
    let path = format!("{}/.contextkeeper/current-todos.json", home);

    if !Path::new(&path).exists() {
        return Vec::new();
    }

    fs::read_to_string(&path)
        .ok()
        .and_then(|content| serde_json::from_str::<serde_json::Value>(&content).ok())
        .and_then(|json| {
            json.get("todos").and_then(|t| t.as_array()).map(|arr| {
                arr.iter()
                    .filter_map(|item| {
                        Some(TodoItem {
                            content: item.get("content")?.as_str()?.to_string(),
                            status: item.get("status")?.as_str()?.to_string(),
                        })
                    })
                    .collect()
            })
        })
        .unwrap_or_default()
}

/// Load recently edited files from Edit/Write hook
fn load_recent_files() -> Vec<String> {
    let home = std::env::var("HOME").unwrap_or_else(|_| ".".to_string());
    let path = format!("{}/.contextkeeper/recent-files.json", home);

    if !Path::new(&path).exists() {
        return Vec::new();
    }

    fs::read_to_string(&path)
        .ok()
        .and_then(|content| serde_json::from_str::<serde_json::Value>(&content).ok())
        .and_then(|json| {
            json.get("files").and_then(|f| f.as_array()).map(|arr| {
                arr.iter()
                    .filter_map(|item| item.get("path")?.as_str().map(|s| s.to_string()))
                    .collect()
            })
        })
        .unwrap_or_default()
}

/// Load or construct work state with hook-collected data
pub fn load_work_state_with_hooks() -> Option<WorkState> {
    // First try to load manually saved work state
    let mut state = load_work_state_from_file().unwrap_or_default();

    // Enhance with hook-collected data
    let hook_todos = load_saved_todos();
    let hook_files = load_recent_files();

    // If we have hook data, update the state
    if !hook_todos.is_empty() {
        state.todos = hook_todos;
    }

    if !hook_files.is_empty() && state.working_files.is_empty() {
        state.working_files = hook_files;
    }

    // Return None if state is completely empty
    if state.task_summary.is_empty()
        && state.todos.is_empty()
        && state.working_files.is_empty()
        && state.notes.is_empty()
    {
        return None;
    }

    Some(state)
}

/// Collect working files from git diff (for PreCompact hook)
pub fn collect_working_files() -> Vec<String> {
    let mut files = Vec::new();
    let cwd = std::env::current_dir()
        .map(|p| p.to_string_lossy().to_string())
        .unwrap_or_else(|_| ".".to_string());

    // Try to get modified files from all git repos
    if let Ok(output) = std::process::Command::new("bash")
        .args(["-c", &format!(
            "cd '{}' && find . -maxdepth 3 -name '.git' -type d 2>/dev/null | while read gitdir; do \
             repo=$(dirname \"$gitdir\"); \
             git -C \"$repo\" diff --name-only 2>/dev/null | sed \"s|^|$repo/|\" ; \
             done | head -20",
            cwd
        )])
        .output()
    {
        if output.status.success() {
            let stdout = String::from_utf8_lossy(&output.stdout);
            for line in stdout.lines() {
                let line = line.trim();
                if !line.is_empty() {
                    // Clean up path (remove leading ./)
                    let clean_path = line.trim_start_matches("./");
                    files.push(clean_path.to_string());
                }
            }
        }
    }

    files
}
