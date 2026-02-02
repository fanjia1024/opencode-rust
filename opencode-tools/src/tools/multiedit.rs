use opencode_core::error::{Error, Result};
use opencode_core::tool::{Tool, ToolContext, ToolResult};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::sync::Arc;
use tokio::fs;

#[derive(Debug, Serialize, Deserialize, JsonSchema)]
pub struct MultiEditArgs {
    pub file_path: String,
    pub edits: Vec<EditOperation>,
}

#[derive(Debug, Serialize, Deserialize, JsonSchema)]
pub struct EditOperation {
    pub file_path: String,
    pub old_string: String,
    pub new_string: String,
    pub replace_all: Option<bool>,
}

pub struct MultiEditTool;

impl MultiEditTool {
    pub fn new() -> Arc<Self> {
        Arc::new(Self)
    }
}

#[async_trait::async_trait]
impl Tool for MultiEditTool {
    fn id(&self) -> &str {
        "multiedit"
    }

    fn description(&self) -> &str {
        "Apply multiple edits to a file by replacing strings"
    }

    fn parameters(&self) -> serde_json::Value {
        serde_json::json!({"type": "object", "properties": {}, "required": []})
    }

    async fn execute(&self, args: Value, _ctx: &ToolContext) -> Result<ToolResult> {
        let args: MultiEditArgs = serde_json::from_value(args)
            .map_err(|e| Error::Validation(format!("Invalid arguments: {}", e)))?;

        let content = fs::read_to_string(&args.file_path)
            .await
            .map_err(|e| Error::Tool(format!("Failed to read file {}: {}", args.file_path, e)))?;

        let mut modified_content = content;
        let mut results = Vec::new();

        for edit in &args.edits {
            let replace_all = edit.replace_all.unwrap_or(false);
            let count = if replace_all {
                modified_content.matches(&edit.old_string).count()
            } else {
                1
            };

            if replace_all {
                modified_content = modified_content.replace(&edit.old_string, &edit.new_string);
            } else {
                modified_content = modified_content.replacen(&edit.old_string, &edit.new_string, 1);
            }

            results.push(serde_json::json!({
                "file_path": edit.file_path,
                "replacements": count
            }));
        }

        fs::write(&args.file_path, modified_content.as_bytes())
            .await
            .map_err(|e| Error::Tool(format!("Failed to write file {}: {}", args.file_path, e)))?;

        Ok(ToolResult {
            title: format!("MultiEdit {}", args.file_path),
            output: format!("Successfully applied {} edit(s) to {}", args.edits.len(), args.file_path),
            metadata: serde_json::json!({
                "file_path": args.file_path,
                "edits": args.edits.len(),
                "results": results
            }),
        })
    }
}
