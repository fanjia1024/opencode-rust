use opencode_core::error::{Error, Result};
use opencode_core::tool::{Tool, ToolContext, ToolResult};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::sync::Arc;
use tokio::fs;

#[derive(Debug, Serialize, Deserialize, JsonSchema)]
pub struct ReadArgs {
    pub path: String,
}

pub struct ReadTool;

impl ReadTool {
    pub fn new() -> Arc<Self> {
        Arc::new(Self)
    }
}

#[async_trait::async_trait]
impl Tool for ReadTool {
    fn id(&self) -> &str {
        "read"
    }

    fn description(&self) -> &str {
        "Read the contents of a file"
    }

    fn parameters(&self) -> serde_json::Value {
        serde_json::json!({
            "type": "object",
            "properties": {
                "path": {
                    "type": "string",
                    "description": "The path to the file to read"
                }
            },
            "required": ["path"]
        })
    }

    async fn execute(&self, args: Value, _ctx: &ToolContext) -> Result<ToolResult> {
        let args: ReadArgs = serde_json::from_value(args)
            .map_err(|e| Error::Validation(format!("Invalid arguments: {}", e)))?;

        let content = fs::read_to_string(&args.path)
            .await
            .map_err(|e| Error::Tool(format!("Failed to read file {}: {}", args.path, e)))?;

        let size = content.len();
        Ok(ToolResult {
            title: format!("Read {}", args.path),
            output: content.clone(),
            metadata: serde_json::json!({
                "path": args.path,
                "size": size
            }),
        })
    }
}
