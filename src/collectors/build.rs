//! BuildScript collector - parses config files to extract build targets

use crate::config::Config;
use crate::context::BuildTarget;
use std::fs;
use std::path::Path;

/// Collect build targets from config files
pub fn collect_build_targets(config: &Config) -> Vec<BuildTarget> {
    let mut targets = Vec::new();

    let scripts_config = match &config.scripts {
        Some(sc) => sc,
        None => return targets,
    };

    let config_dir = match &scripts_config.config_dir {
        Some(dir) => dir.clone(),
        None => return targets,
    };

    let pattern = scripts_config.config_pattern.as_deref().unwrap_or("*.conf");
    let full_pattern = format!("{}/{}", config_dir, pattern);

    if let Ok(entries) = glob::glob(&full_pattern) {
        for entry in entries.flatten() {
            if let Some(target) = parse_config_file(&entry) {
                targets.push(target);
            }
        }
    }

    targets
}

/// Parse a single config file into a BuildTarget
fn parse_config_file(path: &Path) -> Option<BuildTarget> {
    let content = fs::read_to_string(path).ok()?;
    let mut target = BuildTarget::default();

    for line in content.lines() {
        let line = line.trim();
        if line.starts_with('#') || line.is_empty() {
            continue;
        }

        if let Some((key, value)) = parse_var_assignment(line) {
            match key.as_str() {
                "TARGET_NAME" => target.name = value,
                "TARGET_DESCRIPTION" => target.description = value,
                "CONTAINER_NAME" => target.container_name = value,
                "LUNCH_TARGET" => target.lunch_target = value,
                "CAN_EMULATOR" => target.can_emulator = value == "true",
                "CAN_FLASH" => target.can_flash = value == "true",
                _ => {}
            }
        }
    }

    if target.name.is_empty() {
        target.name = path
            .file_stem()
            .and_then(|s| s.to_str())
            .unwrap_or("unknown")
            .to_string();
    }

    Some(target)
}

/// Parse a variable assignment line (KEY=value)
fn parse_var_assignment(line: &str) -> Option<(String, String)> {
    let parts: Vec<&str> = line.splitn(2, '=').collect();
    if parts.len() != 2 {
        return None;
    }

    let key = parts[0].trim().to_string();
    let value = parts[1]
        .trim()
        .trim_matches('"')
        .trim_matches('\'')
        .to_string();

    Some((key, value))
}

/// Parse entry point script to extract available commands
pub fn parse_entry_point_commands(entry_point: &str) -> Vec<String> {
    let mut commands = Vec::new();

    if let Ok(content) = fs::read_to_string(entry_point) {
        for line in content.lines() {
            let line = line.trim();
            if line.contains("./") && line.contains(".sh ") {
                commands.push(line.to_string());
            }
        }
    }

    commands.sort();
    commands.dedup();
    commands.truncate(10);
    commands
}
