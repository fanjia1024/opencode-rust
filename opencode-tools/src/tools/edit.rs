use opencode_core::error::{Error, Result};
use opencode_core::tool::{Tool, ToolContext, ToolResult};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::sync::Arc;
use tokio::fs;

#[derive(Debug, Serialize, Deserialize, JsonSchema)]
pub struct EditArgs {
    pub path: String,
    pub edits: Vec<Edit>,
}

#[derive(Debug, Serialize, Deserialize, JsonSchema)]
pub struct Edit {
    pub start_line: usize,
    pub end_line: usize,
    pub content: String,
}

pub struct EditTool;

impl EditTool {
    pub fn new() -> Arc<Self> {
        Arc::new(Self)
    }
}

#[async_trait::async_trait]
impl Tool for EditTool {
    fn id(&self) -> &str {
        "edit"
    }

    fn description(&self) -> &str {
        "Edit a file by replacing lines"
    }

    fn parameters(&self) -> serde_json::Value {
        serde_json::json!({"type": "object", "properties": {}, "required": []})
    }

    async fn execute(&self, args: Value, _ctx: &ToolContext) -> Result<ToolResult> {
        let args: EditArgs = serde_json::from_value(args)
            .map_err(|e| Error::Validation(format!("Invalid arguments: {}", e)))?;

        let content = fs::read_to_string(&args.path)
            .await
            .map_err(|e| Error::Tool(format!("Failed to read file {}: {}", args.path, e)))?;

        let mut lines: Vec<String> = content.lines().map(|s| s.to_string()).collect();
        
        for edit in args.edits.iter().rev() {
            if edit.start_line > lines.len() || edit.end_line > lines.len() {
                return Err(Error::Tool(format!(
                    "Line numbers out of range: {} - {} (file has {} lines)",
                    edit.start_line,
                    edit.end_line,
                    lines.len()
                )));
            }

            let new_lines: Vec<String> = edit.content.lines().map(|s| s.to_string()).collect();
            lines.splice(edit.start_line..edit.end_line, new_lines);
        }

        let new_content = lines.join("\n");
        fs::write(&args.path, new_content.as_bytes())
            .await
            .map_err(|e| Error::Tool(format!("Failed to write file {}: {}", args.path, e)))?;

        Ok(ToolResult {
            title: format!("Edit {}", args.path),
            output: format!("Successfully edited {} with {} edit(s)", args.path, args.edits.len()),
            metadata: serde_json::json!({
                "path": args.path,
                "edits": args.edits.len()
            }),
        })
    }
}
