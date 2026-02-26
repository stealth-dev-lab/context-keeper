//! ContextKeeper - AI-Native Development Context Engine
//!
//! Helps AI agents remember your build environment after context compression.

mod config;
mod context;
mod collectors;
mod formatters;
mod mcp;
mod cli;

use rmcp::{transport::stdio, ServiceExt};

use crate::collectors::{collect_working_files, save_work_state_to_file};
use crate::context::WorkState;
use crate::mcp::ContextKeeperService;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args: Vec<String> = std::env::args().collect();

    // Init wizard mode
    // Usage: context-keeper init
    if args.iter().any(|arg| arg == "init") {
        cli::run_init_wizard()?;
        return Ok(());
    }

    // CLI mode: output context directly
    // Usage: context-keeper --context [minimal|normal|full]
    if args.iter().any(|arg| arg == "--context" || arg == "-c") {
        // Check for level argument
        let level = args
            .iter()
            .position(|arg| arg == "--context" || arg == "-c")
            .and_then(|i| args.get(i + 1))
            .map(|s| s.as_str())
            .unwrap_or("normal");

        cli::run_context_command(level);
        return Ok(());
    }

    // Save state mode (for PreCompact hook)
    // Usage: context-keeper --save-state "task description"
    if let Some(pos) = args.iter().position(|arg| arg == "--save-state") {
        let task_summary = args.get(pos + 1).cloned().unwrap_or_default();
        let files = collect_working_files();

        let state = WorkState {
            saved_at: chrono::Utc::now().to_rfc3339(),
            trigger: "pre_compact".to_string(),
            task_summary,
            working_files: files,
            notes: String::new(),
            todos: Vec::new(),
        };

        match save_work_state_to_file(&state) {
            Ok(_) => println!(
                "Work state saved: {} files tracked",
                state.working_files.len()
            ),
            Err(e) => eprintln!("Failed to save work state: {}", e),
        }
        return Ok(());
    }

    // MCP Server mode (default)
    let service = ContextKeeperService::new();
    let server = service.serve(stdio()).await?;
    server.waiting().await?;

    Ok(())
}
