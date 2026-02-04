use async_trait::async_trait;
use langchain_ai_rust::error::ToolError as LangChainToolError;
use opencode_core::tool::{Tool, ToolContext};
use serde_json::Value;
use std::sync::Arc;

use crate::deep_agent_turn::{OnToolCall, ToolCallEvent};

/// Strips a leading "path:" or "path :" (case-insensitive, optional space) from a string
/// so that LLM output like "path: /foo/bar" is normalized to "/foo/bar".
fn strip_path_prefix(s: &str) -> String {
    let s = s.trim();
    let lower = s.to_lowercase();
    let rest = if lower.starts_with("path:") {
        s.get(5..).unwrap_or(s).trim()
    } else if lower.starts_with("path :") {
        s.get(6..).unwrap_or(s).trim()
    } else {
        s
    };
    rest.to_string()
}

/// Coerces tool input from formats the LLM/framework may send (e.g. plain string)
/// into the object shape our tools expect (e.g. {"path": s} or {"command": s}).
fn normalize_tool_input(
    tool_id: &str,
    input: Value,
    workspace_path: Option<&str>,
) -> Value {
    match input {
        Value::String(s) if !s.is_empty() => match tool_id {
            "bash" => serde_json::json!({ "command": s }),
            "read" | "ls" | "list_files" => {
                serde_json::json!({ "path": strip_path_prefix(&s) })
            }
            _ => Value::String(s),
        },
        Value::Object(mut map) if ["read", "ls", "list_files"].contains(&tool_id) => {
            if let Some(Value::String(p)) = map.get("path") {
                let normalized = strip_path_prefix(p);
                if !normalized.is_empty() {
                    map.insert("path".to_string(), Value::String(normalized));
                }
            }
            let path_empty = map
                .get("path")
                .and_then(|v| v.as_str())
                .map_or(true, |p| p.is_empty());
            if path_empty {
                if let Some(wp) = workspace_path {
                    map.insert("path".to_string(), Value::String(wp.to_string()));
                }
            }
            Value::Object(map)
        }
        other => other,
    }
}

pub struct LangChainToolAdapter {
    tool: Arc<dyn Tool>,
    /// When set, used in run(); otherwise a default context is used (e.g. for init).
    context: Option<ToolContext>,
    /// When set, called after each tool run (for TUI log).
    on_tool_call: Option<OnToolCall>,
}

impl LangChainToolAdapter {
    /// Creates an adapter with a default context (session_id etc. set to "langchain").
    /// Use this when no real session context is available (e.g. init deep agent).
    pub fn new(tool: Arc<dyn Tool>) -> Self {
        Self {
            tool,
            context: None,
            on_tool_call: None,
        }
    }

    /// Creates an adapter that uses the given context for every tool execution.
    pub fn new_with_context(tool: Arc<dyn Tool>, context: ToolContext) -> Self {
        Self {
            tool,
            context: Some(context),
            on_tool_call: None,
        }
    }

    /// Creates an adapter with context and optional callback for tool events (e.g. TUI log).
    pub fn new_with_context_and_callback(
        tool: Arc<dyn Tool>,
        context: ToolContext,
        on_tool_call: Option<OnToolCall>,
    ) -> Self {
        Self {
            tool,
            context: Some(context),
            on_tool_call,
        }
    }
}

#[async_trait]
impl langchain_ai_rust::tools::Tool for LangChainToolAdapter {
    fn name(&self) -> String {
        self.tool.id().to_string()
    }

    fn description(&self) -> String {
        self.tool.description().to_string()
    }

    fn parameters(&self) -> Value {
        // Return the parameters schema from the tool
        self.tool.parameters()
    }

    async fn run(&self, input: Value) -> Result<String, LangChainToolError> {
        let tool_id = self.tool.id();
        tracing::info!(
            tool_id = %tool_id,
            input = %input,
            "deep_agent tool run start"
        );
        let tool = self.tool.clone();
        let ctx = self.context.clone().unwrap_or_else(|| ToolContext {
            session_id: "langchain".to_string(),
            message_id: "langchain".to_string(),
            agent: "langchain".to_string(),
            call_id: None,
            workspace_path: None,
        });

        let input = normalize_tool_input(tool_id, input, ctx.workspace_path.as_deref());

        let result = tool.execute(input.clone(), &ctx).await;
        if let Some(ref cb) = self.on_tool_call {
            let input_preview = input.to_string();
            let input_preview = if input_preview.len() > 120 {
                format!("{}…", &input_preview[..120])
            } else {
                input_preview
            };
            let event = match &result {
                Ok(r) => ToolCallEvent {
                    tool_id: tool_id.to_string(),
                    input_preview,
                    output_len: Some(r.output.len()),
                    error: None,
                },
                Err(e) => ToolCallEvent {
                    tool_id: tool_id.to_string(),
                    input_preview,
                    output_len: None,
                    error: Some(e.to_string()),
                },
            };
            cb(event);
        }
        match result {
            Ok(result) => {
                tracing::info!(
                    tool_id = %tool_id,
                    output_len = result.output.len(),
                    "deep_agent tool run ok"
                );
                let output = serde_json::json!({
                    "output": result.output,
                    "title": result.title,
                    "metadata": result.metadata
                });
                Ok(output.to_string())
            }
            Err(e) => {
                tracing::error!(
                    tool_id = %tool_id,
                    error = %e,
                    "deep_agent tool run err"
                );
                Err(LangChainToolError::ExecutionError(e.to_string()))
            }
        }
    }
}
