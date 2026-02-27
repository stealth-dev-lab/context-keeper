//! History collector - tracks relevant commands via hook-captured logs

use super::traits::Collector;
use crate::config::Config;
use crate::context::{Context, HistoryEntry};
use regex::Regex;
use std::fs;
use std::io::{self, BufRead};
use std::path::Path;

/// Command history collector
#[derive(Debug, Default)]
pub struct HistoryCollector;

impl Collector for HistoryCollector {
    fn name(&self) -> &'static str {
        "history"
    }

    fn is_enabled(&self, config: &Config) -> bool {
        config
            .history
            .as_ref()
            .and_then(|h| h.enabled)
            .unwrap_or(true)
    }

    fn collect(&self, config: &Config, ctx: &mut Context) {
        ctx.command_history = collect_command_history(config);
    }
}

/// Collect command history from log file
fn collect_command_history(config: &Config) -> Vec<HistoryEntry> {
    let history_config = match &config.history {
        Some(hc) if hc.enabled.unwrap_or(true) => hc,
        _ => return Vec::new(),
    };

    let log_file = history_config.log_file.clone().unwrap_or_else(|| {
        let home = std::env::var("HOME").unwrap_or_else(|_| ".".to_string());
        format!("{}/.contextkeeper/command-history.jsonl", home)
    });

    let max_entries = history_config.max_entries.unwrap_or(20);

    let default_patterns = vec![
        r"lunch\s+\S+".to_string(),
        r"source\s+.*envsetup".to_string(),
        r"export\s+\w+=".to_string(),
        r"m\s+\S+".to_string(),
        r"mm\b".to_string(),
        r"mma\b".to_string(),
    ];

    let patterns = history_config.patterns.clone().unwrap_or(default_patterns);

    let compiled_patterns: Vec<Regex> =
        patterns.iter().filter_map(|p| Regex::new(p).ok()).collect();

    let mut entries = Vec::new();
    let path = Path::new(&log_file);

    if !path.exists() {
        return entries;
    }

    if let Ok(file) = fs::File::open(path) {
        let reader = io::BufReader::new(file);

        for line in reader.lines().map_while(Result::ok) {
            if let Ok(json) = serde_json::from_str::<serde_json::Value>(&line) {
                let command = json["command"].as_str().unwrap_or("");
                let matches_pattern = compiled_patterns.is_empty()
                    || compiled_patterns.iter().any(|re| re.is_match(command));

                if matches_pattern && !command.is_empty() {
                    entries.push(HistoryEntry {
                        timestamp: json["timestamp"].as_str().unwrap_or("").to_string(),
                        command: command.to_string(),
                    });
                }
            }
        }
    }

    if entries.len() > max_entries {
        entries.drain(0..entries.len() - max_entries);
    }

    entries
}
