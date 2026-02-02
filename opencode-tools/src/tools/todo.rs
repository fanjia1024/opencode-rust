use opencode_core::error::Result;
use opencode_core::tool::{Tool, ToolContext, ToolResult};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::sync::Arc;

#[derive(Debug, Serialize, Deserialize, JsonSchema)]
pub struct TodoArgs {
    pub action: String,
    pub content: Option<String>,
}

pub struct TodoTool;

impl TodoTool {
    pub fn new() -> Arc<Self> {
        Arc::new(Self)
    }
}

#[async_trait::async_trait]
impl Tool for TodoTool {
    fn id(&self) -> &str {
        "todo"
    }

    fn description(&self) -> &str {
        "Read or write todo items in the current session"
    }

    fn parameters(&self) -> serde_json::Value {
        serde_json::json!({"type": "object", "properties": {}, "required": []})
    }

    async fn execute(&self, args: Value, ctx: &ToolContext) -> Result<ToolResult> {
        let args: TodoArgs = serde_json::from_value(args)
            .map_err(|e| opencode_core::error::Error::Validation(format!("Invalid arguments: {}", e)))?;

        match args.action.as_str() {
            "read" => {
                Ok(ToolResult {
                    title: "Todo List".to_string(),
                    output: "Todo items will be displayed here".to_string(),
                    metadata: serde_json::json!({
                        "action": "read",
                        "session_id": ctx.session_id
                    }),
                })
            }
            "write" => {
                let content = args.content.unwrap_or_else(|| "".to_string());
                Ok(ToolResult {
                    title: "Todo Added".to_string(),
                    output: format!("Added todo: {}", content),
                    metadata: serde_json::json!({
                        "action": "write",
                        "content": content,
                        "session_id": ctx.session_id
                    }),
                })
            }
            _ => Err(opencode_core::error::Error::Validation(
                format!("Unknown action: {}. Must be 'read' or 'write'", args.action)
            ))
        }
    }
}
