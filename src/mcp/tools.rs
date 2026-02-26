//! MCP Tool implementations

use rmcp::{
    handler::server::{tool::ToolRouter, wrapper::Parameters},
    model::*,
    tool, tool_handler, tool_router,
    ErrorData as McpError, ServerHandler,
};
use schemars::JsonSchema;
use serde::Deserialize;

use crate::collectors::{collect_context, collect_working_files, save_work_state_to_file};
use crate::config::read_config;
use crate::context::{TodoItem, WorkState};
use crate::formatters::format_context_markdown;

/// Parameters for get_dev_context tool
#[derive(Debug, Deserialize, JsonSchema)]
pub struct GetDevContextParams {
    /// Detail level: 'minimal' (~200 tokens), 'normal' (~400 tokens), or 'full' (~1000 tokens). Default: 'normal'
    level: Option<String>,
}

/// Parameters for save_work_state tool
#[derive(Debug, Deserialize, JsonSchema)]
pub struct SaveWorkStateParams {
    /// Brief summary of current task (required)
    task_summary: String,
    /// List of files currently being worked on (auto-detected from git diff if omitted)
    working_files: Option<Vec<String>>,
    /// Additional notes about current progress
    notes: Option<String>,
    /// Todo items as JSON array: [{"content": "...", "status": "pending|in_progress|completed"}]
    todos: Option<String>,
}

#[derive(Clone)]
pub struct ContextKeeperService {
    tool_router: ToolRouter<Self>,
}

impl Default for ContextKeeperService {
    fn default() -> Self {
        Self::new()
    }
}

#[tool_router]
impl ContextKeeperService {
    pub fn new() -> Self {
        Self {
            tool_router: Self::tool_router(),
        }
    }

    #[tool(
        description = "Get development context. Use level='minimal' after compression (~200 tokens), 'normal' for balanced info (~400 tokens), or 'full' for complete details (~1000 tokens). Default is 'normal'."
    )]
    async fn get_dev_context(
        &self,
        params: Parameters<GetDevContextParams>,
    ) -> Result<CallToolResult, McpError> {
        let config = read_config();
        let context = collect_context(&config);
        let level_str = params.0.level.as_deref().unwrap_or("normal");
        let markdown = format_context_markdown(&context, level_str);

        Ok(CallToolResult::success(vec![Content::text(markdown)]))
    }

    #[tool(
        description = "Save current work state for recovery after context compression. Call this before compression or at task milestones."
    )]
    async fn save_work_state(
        &self,
        params: Parameters<SaveWorkStateParams>,
    ) -> Result<CallToolResult, McpError> {
        let SaveWorkStateParams {
            task_summary,
            working_files,
            notes,
            todos,
        } = params.0;

        // Parse todos if provided
        let todo_items: Vec<TodoItem> = todos
            .and_then(|t| serde_json::from_str(&t).ok())
            .unwrap_or_default();

        // Auto-collect working files if not provided
        let files = working_files.unwrap_or_else(collect_working_files);

        let state = WorkState {
            saved_at: chrono::Utc::now().to_rfc3339(),
            trigger: "manual".to_string(),
            task_summary,
            working_files: files,
            notes: notes.unwrap_or_default(),
            todos: todo_items,
        };

        match save_work_state_to_file(&state) {
            Ok(_) => Ok(CallToolResult::success(vec![Content::text(format!(
                "Work state saved successfully.\n\n\
                - Task: {}\n\
                - Files: {}\n\
                - Todos: {} items\n\n\
                This state will be included in `get_dev_context` output after compression.",
                state.task_summary,
                state.working_files.len(),
                state.todos.len()
            ))])),
            Err(e) => Ok(CallToolResult::success(vec![Content::text(format!(
                "Failed to save work state: {}",
                e
            ))])),
        }
    }
}

#[tool_handler]
impl ServerHandler for ContextKeeperService {
    fn get_info(&self) -> ServerInfo {
        ServerInfo {
            protocol_version: ProtocolVersion::LATEST,
            capabilities: ServerCapabilities::builder().enable_tools().build(),
            server_info: Implementation {
                name: "context-keeper".into(),
                version: env!("CARGO_PKG_VERSION").into(),
                ..Default::default()
            },
            instructions: Some(
                "ContextKeeper provides development environment context. \
                 Call get_dev_context to retrieve build targets, containers, \
                 and recent commands."
                    .into(),
            ),
        }
    }
}
