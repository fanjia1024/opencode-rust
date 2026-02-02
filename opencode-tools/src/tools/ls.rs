use opencode_core::error::{Error, Result};
use opencode_core::tool::{Tool, ToolContext, ToolResult};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::sync::Arc;
use tokio::fs;
use walkdir::WalkDir;

#[derive(Debug, Serialize, Deserialize, JsonSchema)]
pub struct ListArgs {
    pub path: String,
    pub recursive: Option<bool>,
}

pub struct ListTool;

impl ListTool {
    pub fn new() -> Arc<Self> {
        Arc::new(Self)
    }
}

#[async_trait::async_trait]
impl Tool for ListTool {
    fn id(&self) -> &str {
        "ls"
    }

    fn description(&self) -> &str {
        "List files and directories"
    }

    fn parameters(&self) -> serde_json::Value {
        serde_json::json!({"type": "object", "properties": {}, "required": []})
    }

    async fn execute(&self, args: Value, _ctx: &ToolContext) -> Result<ToolResult> {
        let args: ListArgs = serde_json::from_value(args)
            .map_err(|e| Error::Validation(format!("Invalid arguments: {}", e)))?;

        let recursive = args.recursive.unwrap_or(false);
        let mut entries = Vec::new();

        if recursive {
            for entry in WalkDir::new(&args.path).max_depth(3) {
                match entry {
                    Ok(e) => {
                        let path = e.path().to_string_lossy().to_string();
                        let is_dir = e.file_type().is_dir();
                        entries.push(format!("{}{}", path, if is_dir { "/" } else { "" }));
                    }
                    Err(e) => {
                        tracing::warn!("Error walking directory: {}", e);
                    }
                }
            }
        } else {
            let mut dir = fs::read_dir(&args.path)
                .await
                .map_err(|e| Error::Tool(format!("Failed to read directory {}: {}", args.path, e)))?;

            while let Some(entry) = dir.next_entry().await? {
                let path = entry.path();
                let name = path.file_name().unwrap().to_string_lossy().to_string();
                let metadata = entry.metadata().await?;
                entries.push(format!("{}{}", name, if metadata.is_dir() { "/" } else { "" }));
            }
        }

        entries.sort();

        Ok(ToolResult {
            title: format!("List {}", args.path),
            output: entries.join("\n"),
            metadata: serde_json::json!({
                "path": args.path,
                "count": entries.len()
            }),
        })
    }
}
