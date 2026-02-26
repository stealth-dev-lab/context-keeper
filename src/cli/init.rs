//! Init wizard for project setup

use std::fs;
use std::io::{self, Write};
use std::path::Path;

/// Detect project type based on directory contents
fn detect_project_type() -> Option<&'static str> {
    // Check for AOSP
    if Path::new("build/envsetup.sh").exists() || Path::new("build/make/envsetup.sh").exists() {
        return Some("aosp");
    }

    // Check for ROS/ROS2
    if Path::new("package.xml").exists() {
        return Some("ros");
    }
    if Path::new("src").is_dir() {
        // Check for colcon/catkin workspace
        if let Ok(entries) = fs::read_dir("src") {
            for entry in entries.flatten() {
                let pkg_xml = entry.path().join("package.xml");
                if pkg_xml.exists() {
                    return Some("ros");
                }
            }
        }
    }

    // Check for Yocto
    if Path::new("meta").is_dir() || Path::new("poky").is_dir() {
        return Some("yocto");
    }
    if let Ok(entries) = fs::read_dir(".") {
        for entry in entries.flatten() {
            let name = entry.file_name();
            if name.to_string_lossy().starts_with("meta-") {
                return Some("yocto");
            }
        }
    }

    None
}

/// Detect available container runtime
fn detect_container_runtime() -> Option<&'static str> {
    // Check podman first (preferred for rootless)
    if std::process::Command::new("podman")
        .arg("--version")
        .output()
        .map(|o| o.status.success())
        .unwrap_or(false)
    {
        return Some("podman");
    }

    // Check docker
    if std::process::Command::new("docker")
        .arg("--version")
        .output()
        .map(|o| o.status.success())
        .unwrap_or(false)
    {
        return Some("docker");
    }

    None
}

/// Get current directory name as default project name
fn get_default_project_name() -> String {
    std::env::current_dir()
        .ok()
        .and_then(|p| p.file_name().map(|n| n.to_string_lossy().to_string()))
        .unwrap_or_else(|| "my-project".to_string())
}

/// Prompt user for input with default value
fn prompt(question: &str, default: &str) -> String {
    if default.is_empty() {
        print!("{}: ", question);
    } else {
        print!("{} [{}]: ", question, default);
    }
    io::stdout().flush().unwrap();

    let mut input = String::new();
    io::stdin().read_line(&mut input).unwrap();
    let input = input.trim();

    if input.is_empty() {
        default.to_string()
    } else {
        input.to_string()
    }
}

/// Prompt for yes/no with default
fn prompt_yes_no(question: &str, default: bool) -> bool {
    let default_str = if default { "Y/n" } else { "y/N" };
    print!("{} [{}]: ", question, default_str);
    io::stdout().flush().unwrap();

    let mut input = String::new();
    io::stdin().read_line(&mut input).unwrap();
    let input = input.trim().to_lowercase();

    match input.as_str() {
        "y" | "yes" => true,
        "n" | "no" => false,
        _ => default,
    }
}

/// Generate default history patterns based on project type
fn get_default_history_patterns(project_type: &str) -> Vec<&'static str> {
    match project_type {
        "aosp" => vec![
            r"lunch\s+\S+",
            r"source.*envsetup",
            r"export\s+\w+=",
            r"m\s+\S+",
            r"mm\b",
            r"mma\b",
        ],
        "ros" => vec![
            r"source.*setup\.bash",
            r"colcon\s+build",
            r"catkin_make",
            r"ros2\s+run",
            r"ros2\s+launch",
            r"roslaunch",
        ],
        "yocto" => vec![
            r"source.*oe-init",
            r"bitbake\s+\S+",
            r"MACHINE=",
            r"devtool\s+\S+",
        ],
        _ => vec![r"export\s+\w+=", r"source\s+"],
    }
}

/// Run the init wizard
pub fn run_init_wizard() -> io::Result<()> {
    println!("\n🔧 ContextKeeper Setup Wizard\n");

    // Check if config already exists
    if Path::new("contextkeeper.toml").exists()
        && !prompt_yes_no("contextkeeper.toml already exists. Overwrite?", false)
    {
        println!("Aborted.");
        return Ok(());
    }

    // Project name
    let default_name = get_default_project_name();
    let project_name = prompt("Project name", &default_name);

    // Project type
    let detected_type = detect_project_type();
    let type_hint = detected_type
        .map(|t| format!("detected: {}", t))
        .unwrap_or_else(|| "aosp/ros/yocto/custom".to_string());
    let project_type = prompt(
        &format!("Project type ({})", type_hint),
        detected_type.unwrap_or("custom"),
    );

    // Container runtime
    let detected_runtime = detect_container_runtime();
    let runtime_hint = detected_runtime
        .map(|r| format!("detected: {}", r))
        .unwrap_or_else(|| "podman/docker/none".to_string());
    let container_runtime = prompt(
        &format!("Container runtime ({})", runtime_hint),
        detected_runtime.unwrap_or("none"),
    );

    // Build scripts (optional)
    let entry_point = prompt("Build script entry point (optional)", "");
    let config_dir = if !entry_point.is_empty() {
        prompt("Config directory (optional)", "")
    } else {
        String::new()
    };

    // AI hints
    let default_hint = if container_runtime != "none" {
        "Build commands must be executed inside the container."
    } else {
        ""
    };
    let ai_hint = prompt("AI hint for this project", default_hint);

    // Generate TOML
    let mut toml_content = String::new();

    toml_content.push_str("# ContextKeeper Configuration\n");
    toml_content.push_str("# https://github.com/stealth-dev-lab/context-keeper\n\n");

    toml_content.push_str("[project]\n");
    toml_content.push_str(&format!("name = \"{}\"\n", project_name));
    toml_content.push_str(&format!("type = \"{}\"\n", project_type));
    toml_content.push('\n');

    if !entry_point.is_empty() {
        toml_content.push_str("[scripts]\n");
        toml_content.push_str(&format!("entry_point = \"{}\"\n", entry_point));
        if !config_dir.is_empty() {
            toml_content.push_str(&format!("config_dir = \"{}\"\n", config_dir));
            toml_content.push_str("config_pattern = \"*.conf\"\n");
        }
        toml_content.push('\n');
    }

    if container_runtime != "none" {
        toml_content.push_str("[containers]\n");
        toml_content.push_str(&format!("runtime = \"{}\"\n", container_runtime));
        toml_content.push('\n');
    }

    if !ai_hint.is_empty() {
        toml_content.push_str("[hints]\n");
        toml_content.push_str(&format!("default = \"{}\"\n", ai_hint));
        toml_content.push('\n');
    }

    // History config with type-appropriate patterns
    toml_content.push_str("[history]\n");
    toml_content.push_str("enabled = true\n");
    toml_content.push_str("patterns = [\n");
    for pattern in get_default_history_patterns(&project_type) {
        // Escape backslashes for TOML
        let escaped = pattern.replace('\\', "\\\\");
        toml_content.push_str(&format!("    \"{}\",\n", escaped));
    }
    toml_content.push_str("]\n");
    toml_content.push_str("max_entries = 20\n");
    toml_content.push('\n');

    // Git config
    toml_content.push_str("[git]\n");
    toml_content.push_str("auto_detect = true\n");
    toml_content.push_str("scan_depth = 2\n");

    // Write file
    fs::write("contextkeeper.toml", &toml_content)?;

    println!("\n✅ Created contextkeeper.toml");
    println!("\nNext steps:");
    println!("  1. Review and customize contextkeeper.toml");
    println!("  2. Test with: context-keeper --context");
    println!("  3. Add to Claude Code: ./install.sh (or see README)");

    Ok(())
}
