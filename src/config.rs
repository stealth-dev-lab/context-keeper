//! Configuration loading and structures

use serde::Deserialize;
use std::fs;
use std::path::Path;

/// Main configuration structure
#[derive(Debug, Deserialize, Default)]
pub struct Config {
    pub project: Option<ProjectConfig>,
    pub scripts: Option<ScriptsConfig>,
    pub containers: Option<ContainersConfig>,
    pub hints: Option<HintsConfig>,
    pub history: Option<HistoryConfig>,
    pub git: Option<GitConfig>,
    pub adb: Option<AdbConfig>,
}

#[derive(Debug, Deserialize)]
pub struct ProjectConfig {
    pub name: Option<String>,
    #[serde(rename = "type")]
    pub project_type: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct ScriptsConfig {
    pub entry_point: Option<String>,
    pub config_dir: Option<String>,
    pub config_pattern: Option<String>,
    #[allow(dead_code)]
    pub extract_vars: Option<Vec<String>>,
}

#[derive(Debug, Deserialize)]
pub struct ContainersConfig {
    pub runtime: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct HintsConfig {
    pub default: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct HistoryConfig {
    pub enabled: Option<bool>,
    pub log_file: Option<String>,
    pub patterns: Option<Vec<String>>,
    pub max_entries: Option<usize>,
}

#[derive(Debug, Deserialize)]
pub struct GitConfig {
    /// Explicit list of repository paths to check (relative to project root)
    pub paths: Option<Vec<String>>,
    /// Auto-detect git repositories in subdirectories
    pub auto_detect: Option<bool>,
    /// Max depth for auto-detection (default: 2)
    pub scan_depth: Option<usize>,
}

#[derive(Debug, Deserialize)]
pub struct AdbConfig {
    /// Enable/disable ADB device collection
    pub enabled: Option<bool>,
}

/// Read configuration from file
pub fn read_config() -> Config {
    let paths = [
        "contextkeeper.toml",
        "context-keeper.toml",
        ".contextkeeper.toml",
    ];

    for path in paths {
        if Path::new(path).exists() {
            if let Ok(content) = fs::read_to_string(path) {
                if let Ok(config) = toml::from_str(&content) {
                    return config;
                }
            }
        }
    }

    Config::default()
}
