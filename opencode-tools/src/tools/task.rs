use opencode_core::error::Result;
use opencode_core::tool::{Tool, ToolContext, ToolResult};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::sync::Arc;

#[derive(Debug, Serialize, Deserialize, JsonSchema)]
pub struct TaskArgs {
    pub description: String,
    pub status: Option<String>,
}

pub struct TaskTool;

impl TaskTool {
    pub fn new() -> Arc<Self> {
        Arc::new(Self)
    }
}

#[async_trait::async_trait]
impl Tool for TaskTool {
    fn id(&self) -> &str {
        "task"
    }

    fn description(&self) -> &str {
        "Manage tasks in the current session"
    }

    fn parameters(&self) -> serde_json::Value {
        serde_json::json!({"type": "object", "properties": {}, "required": []})
    }

    async fn execute(&self, args: Value, ctx: &ToolContext) -> Result<ToolResult> {
        let args: TaskArgs = serde_json::from_value(args)
            .map_err(|e| opencode_core::error::Error::Validation(format!("Invalid arguments: {}", e)))?;

        let status = args.status.as_deref().unwrap_or("pending");

        Ok(ToolResult {
            title: "Task".to_string(),
            output: format!("Task: {}\nStatus: {}", args.description, status),
            metadata: serde_json::json!({
                "description": args.description,
                "status": status,
                "session_id": ctx.session_id
            }),
        })
    }
}
