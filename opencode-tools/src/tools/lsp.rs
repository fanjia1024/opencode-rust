use opencode_core::error::{Error, Result};
use opencode_core::tool::{Tool, ToolContext, ToolResult};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::sync::Arc;
use tokio::process::Command;

#[derive(Debug, Serialize, Deserialize, JsonSchema)]
pub struct LspArgs {
    pub command: String,
    pub file_path: Option<String>,
    pub line: Option<u32>,
    pub column: Option<u32>,
}

pub struct LspTool;

impl LspTool {
    pub fn new() -> Arc<Self> {
        Arc::new(Self)
    }
}

#[async_trait::async_trait]
impl Tool for LspTool {
    fn id(&self) -> &str {
        "lsp"
    }

    fn description(&self) -> &str {
        "Interact with Language Server Protocol (LSP) for code intelligence"
    }

    fn parameters(&self) -> serde_json::Value {
        serde_json::json!({"type": "object", "properties": {}, "required": []})
    }

    async fn execute(&self, args: Value, _ctx: &ToolContext) -> Result<ToolResult> {
        let args: LspArgs = serde_json::from_value(args)
            .map_err(|e| Error::Validation(format!("Invalid arguments: {}", e)))?;

        match args.command.as_str() {
            "hover" => {
                if let (Some(file_path), Some(line), Some(column)) = (args.file_path, args.line, args.column) {
                    Ok(ToolResult {
                        title: "LSP Hover".to_string(),
                        output: format!("Hover info for {}:{}:{}", file_path, line, column),
                        metadata: serde_json::json!({
                            "command": "hover",
                            "file": file_path,
                            "line": line,
                            "column": column
                        }),
                    })
                } else {
                    Err(Error::Validation("hover command requires file_path, line, and column".to_string()))
                }
            }
            "definition" => {
                if let (Some(file_path), Some(line), Some(column)) = (args.file_path, args.line, args.column) {
                    Ok(ToolResult {
                        title: "LSP Definition".to_string(),
                        output: format!("Definition for {}:{}:{}", file_path, line, column),
                        metadata: serde_json::json!({
                            "command": "definition",
                            "file": file_path,
                            "line": line,
                            "column": column
                        }),
                    })
                } else {
                    Err(Error::Validation("definition command requires file_path, line, and column".to_string()))
                }
            }
            "references" => {
                if let (Some(file_path), Some(line), Some(column)) = (args.file_path, args.line, args.column) {
                    Ok(ToolResult {
                        title: "LSP References".to_string(),
                        output: format!("References for {}:{}:{}", file_path, line, column),
                        metadata: serde_json::json!({
                            "command": "references",
                            "file": file_path,
                            "line": line,
                            "column": column
                        }),
                    })
                } else {
                    Err(Error::Validation("references command requires file_path, line, and column".to_string()))
                }
            }
            _ => Err(Error::Validation(format!("Unknown LSP command: {}", args.command)))
        }
    }
}
