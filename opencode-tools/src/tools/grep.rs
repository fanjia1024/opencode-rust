use opencode_core::error::{Error, Result};
use opencode_core::tool::{Tool, ToolContext, ToolResult};
use regex::Regex;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::sync::Arc;
use tokio::fs;
use walkdir::WalkDir;

#[derive(Debug, Serialize, Deserialize, JsonSchema)]
pub struct GrepArgs {
    pub pattern: String,
    pub path: String,
    pub recursive: Option<bool>,
}

pub struct GrepTool;

impl GrepTool {
    pub fn new() -> Arc<Self> {
        Arc::new(Self)
    }
}

#[async_trait::async_trait]
impl Tool for GrepTool {
    fn id(&self) -> &str {
        "grep"
    }

    fn description(&self) -> &str {
        "Search for a pattern in files"
    }

    fn parameters(&self) -> serde_json::Value {
        serde_json::json!({"type": "object", "properties": {}, "required": []})
    }

    async fn execute(&self, args: Value, _ctx: &ToolContext) -> Result<ToolResult> {
        let args: GrepArgs = serde_json::from_value(args)
            .map_err(|e| Error::Validation(format!("Invalid arguments: {}", e)))?;

        let regex = Regex::new(&args.pattern)
            .map_err(|e| Error::Validation(format!("Invalid regex pattern: {}", e)))?;

        let recursive = args.recursive.unwrap_or(false);
        let mut matches = Vec::new();

        if recursive {
            for entry in WalkDir::new(&args.path).into_iter() {
                match entry {
                    Ok(e) => {
                        if e.file_type().is_file() {
                            if let Ok(content) = fs::read_to_string(e.path()).await {
                                for (line_num, line) in content.lines().enumerate() {
                                    if regex.is_match(line) {
                                        matches.push(format!("{}:{}:{}", e.path().display(), line_num + 1, line));
                                    }
                                }
                            }
                        }
                    }
                    Err(e) => {
                        tracing::warn!("Error reading file: {}", e);
                    }
                }
            }
        } else {
            let content = fs::read_to_string(&args.path)
                .await
                .map_err(|e| Error::Tool(format!("Failed to read file {}: {}", args.path, e)))?;
            
            for (line_num, line) in content.lines().enumerate() {
                if regex.is_match(line) {
                    matches.push(format!("{}:{}:{}", args.path, line_num + 1, line));
                }
            }
        }

        Ok(ToolResult {
            title: format!("Grep '{}' in {}", args.pattern, args.path),
            output: matches.join("\n"),
            metadata: serde_json::json!({
                "pattern": args.pattern,
                "path": args.path,
                "matches": matches.len()
            }),
        })
    }
}
