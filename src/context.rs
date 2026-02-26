//! Context data structures

use serde::{Deserialize, Serialize};

/// Build target information
#[derive(Debug, Default, Clone)]
pub struct BuildTarget {
    pub name: String,
    pub description: String,
    pub container_name: String,
    pub lunch_target: String,
    pub can_emulator: bool,
    pub can_flash: bool,
}

/// Container information
#[derive(Debug, Default, Clone)]
pub struct ContainerInfo {
    pub name: String,
    pub status: String,
    pub runtime: String,
}

/// Command history entry
#[derive(Debug, Clone)]
pub struct HistoryEntry {
    pub timestamp: String,
    pub command: String,
}

/// Git repository information
#[derive(Debug, Default, Clone)]
pub struct GitInfo {
    pub repo_path: String,
    pub branch: String,
    pub is_dirty: bool,
    pub modified_files: usize,
    pub untracked_files: usize,
    pub last_commit_short: String,
}

/// ADB/Fastboot device information
#[derive(Debug, Clone)]
pub struct AdbDevice {
    pub serial: String,
    pub state: String,
    pub device_type: String, // "adb" or "fastboot"
}

/// Todo item for work state
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct TodoItem {
    pub content: String,
    pub status: String, // "pending", "in_progress", "completed"
}

/// Saved work state for recovery after context compression
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct WorkState {
    pub saved_at: String,
    pub trigger: String, // "manual", "pre_compact", "auto"
    pub task_summary: String,
    pub working_files: Vec<String>,
    pub notes: String,
    pub todos: Vec<TodoItem>,
}

/// Aggregated development context
#[derive(Debug, Default, Clone)]
pub struct Context {
    pub project_name: String,
    pub project_type: String,
    pub targets: Vec<BuildTarget>,
    pub containers: Vec<ContainerInfo>,
    pub available_commands: Vec<String>,
    pub hints: String,
    pub command_history: Vec<HistoryEntry>,
    pub git_repos: Vec<GitInfo>,
    pub adb_devices: Vec<AdbDevice>,
    pub work_state: Option<WorkState>,
}
