use opencode_core::error::{Error, Result};
use opencode_core::tool::{Tool, ToolContext, ToolResult};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::sync::Arc;
use tokio::fs;

#[derive(Debug, Serialize, Deserialize, JsonSchema)]
pub struct WriteArgs {
    pub path: String,
    pub content: String,
}

pub struct WriteTool;

impl WriteTool {
    pub fn new() -> Arc<Self> {
        Arc::new(Self)
    }
}

#[async_trait::async_trait]
impl Tool for WriteTool {
    fn id(&self) -> &str {
        "write"
    }

    fn description(&self) -> &str {
        "Write content to a file"
    }

    fn parameters(&self) -> serde_json::Value {
        serde_json::json!({"type": "object", "properties": {}, "required": []})
    }

    async fn execute(&self, args: Value, _ctx: &ToolContext) -> Result<ToolResult> {
        let args: WriteArgs = serde_json::from_value(args)
            .map_err(|e| Error::Validation(format!("Invalid arguments: {}", e)))?;

        fs::write(&args.path, args.content.as_bytes())
            .await
            .map_err(|e| Error::Tool(format!("Failed to write file {}: {}", args.path, e)))?;

        Ok(ToolResult {
            title: format!("Write {}", args.path),
            output: format!("Successfully wrote {} bytes to {}", args.content.len(), args.path),
            metadata: serde_json::json!({
                "path": args.path,
                "size": args.content.len()
            }),
        })
    }
}
