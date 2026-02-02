use opencode_core::error::{Error, Result};
use opencode_core::tool::{Tool, ToolContext, ToolResult};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::sync::Arc;
use tokio::process::Command;

#[derive(Debug, Serialize, Deserialize, JsonSchema)]
pub struct BashArgs {
    pub command: String,
}

pub struct BashTool;

impl BashTool {
    pub fn new() -> Arc<Self> {
        Arc::new(Self)
    }
}

#[async_trait::async_trait]
impl Tool for BashTool {
    fn id(&self) -> &str {
        "bash"
    }

    fn description(&self) -> &str {
        "Execute a bash command"
    }

    fn parameters(&self) -> serde_json::Value {
        serde_json::json!({
            "type": "object",
            "properties": {
                "command": {
                    "type": "string",
                    "description": "The bash command to execute"
                }
            },
            "required": ["command"]
        })
    }

    async fn execute(&self, args: Value, _ctx: &ToolContext) -> Result<ToolResult> {
        let args: BashArgs = serde_json::from_value(args)
            .map_err(|e| Error::Validation(format!("Invalid arguments: {}", e)))?;

        let output = Command::new("sh")
            .arg("-c")
            .arg(&args.command)
            .output()
            .await
            .map_err(|e| Error::Tool(format!("Failed to execute command: {}", e)))?;

        let stdout = String::from_utf8_lossy(&output.stdout);
        let stderr = String::from_utf8_lossy(&output.stderr);

        let output_text = if !stderr.is_empty() {
            format!("STDOUT:\n{}\n\nSTDERR:\n{}", stdout, stderr)
        } else {
            stdout.to_string()
        };

        Ok(ToolResult {
            title: format!("Bash: {}", args.command),
            output: output_text,
            metadata: serde_json::json!({
                "command": args.command,
                "exit_code": output.status.code(),
                "success": output.status.success()
            }),
        })
    }
}
