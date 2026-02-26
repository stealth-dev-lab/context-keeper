//! Git collector - multi-repository status

use crate::config::Config;
use crate::context::GitInfo;
use std::fs;
use std::path::Path;

/// Collect git info from a single repository path
fn collect_git_info_for_path(repo_path: &str) -> Option<GitInfo> {
    // Check if this path is a git repository
    let is_git = std::process::Command::new("git")
        .args(["-C", repo_path, "rev-parse", "--is-inside-work-tree"])
        .output()
        .map(|o| o.status.success())
        .unwrap_or(false);

    if !is_git {
        return None;
    }

    let mut info = GitInfo {
        repo_path: repo_path.to_string(),
        ..Default::default()
    };

    // Get current branch
    if let Ok(output) = std::process::Command::new("git")
        .args(["-C", repo_path, "branch", "--show-current"])
        .output()
    {
        if output.status.success() {
            info.branch = String::from_utf8_lossy(&output.stdout).trim().to_string();
        }
    }

    // If branch is empty, try to get detached HEAD info
    if info.branch.is_empty() {
        if let Ok(output) = std::process::Command::new("git")
            .args(["-C", repo_path, "describe", "--always", "--dirty"])
            .output()
        {
            if output.status.success() {
                info.branch = format!("({})", String::from_utf8_lossy(&output.stdout).trim());
            }
        }
    }

    // Get status (modified and untracked counts)
    if let Ok(output) = std::process::Command::new("git")
        .args(["-C", repo_path, "status", "--porcelain"])
        .output()
    {
        if output.status.success() {
            let status = String::from_utf8_lossy(&output.stdout);
            for line in status.lines() {
                if line.starts_with(" M") || line.starts_with("M ") || line.starts_with("MM") {
                    info.modified_files += 1;
                } else if line.starts_with("??") {
                    info.untracked_files += 1;
                } else if !line.trim().is_empty() {
                    info.modified_files += 1; // Other changes (added, deleted, etc.)
                }
            }
            info.is_dirty = info.modified_files > 0 || info.untracked_files > 0;
        }
    }

    // Get last commit short hash and message
    if let Ok(output) = std::process::Command::new("git")
        .args(["-C", repo_path, "log", "-1", "--format=%h %s"])
        .output()
    {
        if output.status.success() {
            let commit_info = String::from_utf8_lossy(&output.stdout).trim().to_string();
            if commit_info.len() > 50 {
                info.last_commit_short =
                    format!("{}...", &commit_info.chars().take(47).collect::<String>());
            } else {
                info.last_commit_short = commit_info;
            }
        }
    }

    Some(info)
}

/// Auto-detect git repositories in subdirectories
fn find_git_repos(base_path: &str, max_depth: usize) -> Vec<String> {
    let mut repos = Vec::new();
    find_git_repos_recursive(base_path, base_path, 0, max_depth, &mut repos);
    repos.sort();
    repos
}

fn find_git_repos_recursive(
    base_path: &str,
    current_path: &str,
    depth: usize,
    max_depth: usize,
    repos: &mut Vec<String>,
) {
    if depth > max_depth {
        return;
    }

    let current = Path::new(current_path);

    // Check if current directory is a git repo
    let git_dir = current.join(".git");
    if git_dir.exists() {
        // Use relative path from base
        if let Ok(relative) = current.strip_prefix(base_path) {
            let rel_str = relative.to_string_lossy().to_string();
            if !rel_str.is_empty() {
                repos.push(rel_str);
            }
        }
        return; // Don't recurse into git repos
    }

    // Recurse into subdirectories
    if let Ok(entries) = fs::read_dir(current) {
        for entry in entries.flatten() {
            let path = entry.path();
            if path.is_dir() {
                let name = path.file_name().and_then(|n| n.to_str()).unwrap_or("");
                // Skip hidden directories and common non-repo directories
                if name.starts_with('.')
                    || name == "node_modules"
                    || name == "target"
                    || name == "out"
                {
                    continue;
                }
                find_git_repos_recursive(
                    base_path,
                    path.to_str().unwrap_or(""),
                    depth + 1,
                    max_depth,
                    repos,
                );
            }
        }
    }
}

/// Collect git info from multiple repositories based on config
pub fn collect_git_repos(config: &Config) -> Vec<GitInfo> {
    let mut repos = Vec::new();
    let cwd = std::env::current_dir()
        .map(|p| p.to_string_lossy().to_string())
        .unwrap_or_else(|_| ".".to_string());

    // First, check if current directory itself is a git repo
    if let Some(info) = collect_git_info_for_path(&cwd) {
        let mut info = info;
        info.repo_path = ".".to_string();
        repos.push(info);
        return repos; // If root is a git repo, don't scan subdirectories
    }

    // Get paths from config or auto-detect
    let git_config = config.git.as_ref();
    let auto_detect = git_config.and_then(|g| g.auto_detect).unwrap_or(true);
    let explicit_paths = git_config.and_then(|g| g.paths.clone());
    let scan_depth = git_config.and_then(|g| g.scan_depth).unwrap_or(2);

    let paths_to_check: Vec<String> = if let Some(paths) = explicit_paths {
        paths
    } else if auto_detect {
        find_git_repos(&cwd, scan_depth)
    } else {
        Vec::new()
    };

    // Collect info from each path
    for path in paths_to_check {
        let full_path = if Path::new(&path).is_absolute() {
            path.clone()
        } else {
            format!("{}/{}", cwd, path)
        };

        if let Some(mut info) = collect_git_info_for_path(&full_path) {
            info.repo_path = path;
            repos.push(info);
        }
    }

    // Sort by path for consistent output
    repos.sort_by(|a, b| a.repo_path.cmp(&b.repo_path));

    // Limit to reasonable number
    repos.truncate(10);

    repos
}
