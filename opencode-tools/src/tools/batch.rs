use opencode_core::error::{Error, Result};
use opencode_core::tool::{Tool, ToolContext, ToolResult};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::sync::Arc;

#[derive(Debug, Serialize, Deserialize, JsonSchema)]
pub struct BatchArgs {
    pub commands: Vec<Command>,
}

#[derive(Debug, Serialize, Deserialize, JsonSchema)]
pub struct Command {
    pub tool: String,
    pub args: Value,
}

pub struct BatchTool;

impl BatchTool {
    pub fn new() -> Arc<Self> {
        Arc::new(Self)
    }
}

#[async_trait::async_trait]
impl Tool for BatchTool {
    fn id(&self) -> &str {
        "batch"
    }

    fn description(&self) -> &str {
        "Execute multiple tool commands in sequence"
    }

    fn parameters(&self) -> serde_json::Value {
        serde_json::json!({
            "type": "object",
            "properties": {
                "commands": {
                    "type": "array",
                    "items": {
                        "type": "object",
                        "properties": {
                            "tool": {"type": "string"},
                            "args": {"type": "object"}
                        }
                    }
                }
            },
            "required": ["commands"]
        })
    }

    async fn execute(&self, args: Value, ctx: &ToolContext) -> Result<ToolResult> {
        let args: BatchArgs = serde_json::from_value(args)
            .map_err(|e| Error::Validation(format!("Invalid arguments: {}", e)))?;

        let mut results = Vec::new();
        let mut success_count = 0;

        let mut temp_registry = crate::registry::ToolRegistry::new();
        crate::tools::register_all_tools(&mut temp_registry);
        
        for cmd in &args.commands {
            match temp_registry.execute(&cmd.tool, cmd.args.clone(), ctx).await {
                Ok(result) => {
                    results.push(serde_json::json!({
                        "tool": cmd.tool,
                        "success": true,
                        "title": result.title
                    }));
                    success_count += 1;
                }
                Err(e) => {
                    results.push(serde_json::json!({
                        "tool": cmd.tool,
                        "success": false,
                        "error": e.to_string()
                    }));
                }
            }
        }

        Ok(ToolResult {
            title: format!("Batch: {}/{} succeeded", success_count, args.commands.len()),
            output: serde_json::to_string_pretty(&results).unwrap_or_default(),
            metadata: serde_json::json!({
                "total": args.commands.len(),
                "success": success_count,
                "failed": args.commands.len() - success_count
            }),
        })
    }
}
