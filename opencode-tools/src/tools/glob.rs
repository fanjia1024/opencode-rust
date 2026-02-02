use globset::{Glob, GlobMatcher};
use opencode_core::error::{Error, Result};
use opencode_core::tool::{Tool, ToolContext, ToolResult};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::sync::Arc;
use walkdir::WalkDir;

#[derive(Debug, Serialize, Deserialize, JsonSchema)]
pub struct GlobArgs {
    pub pattern: String,
    pub base_path: Option<String>,
}

pub struct GlobTool;

impl GlobTool {
    pub fn new() -> Arc<Self> {
        Arc::new(Self)
    }
}

#[async_trait::async_trait]
impl Tool for GlobTool {
    fn id(&self) -> &str {
        "glob"
    }

    fn description(&self) -> &str {
        "Find files matching a glob pattern"
    }

    fn parameters(&self) -> serde_json::Value {
        serde_json::json!({"type": "object", "properties": {}, "required": []})
    }

    async fn execute(&self, args: Value, _ctx: &ToolContext) -> Result<ToolResult> {
        let args: GlobArgs = serde_json::from_value(args)
            .map_err(|e| Error::Validation(format!("Invalid arguments: {}", e)))?;

        let glob = Glob::new(&args.pattern)
            .map_err(|e| Error::Validation(format!("Invalid glob pattern: {}", e)))?;
        let matcher = glob.compile_matcher();

        let base_path = args.base_path.unwrap_or_else(|| ".".to_string());
        let mut matches = Vec::new();

        for entry in WalkDir::new(&base_path).into_iter() {
            match entry {
                Ok(e) => {
                    let path_str = e.path().to_string_lossy().to_string();
                    if matcher.is_match(&path_str) {
                        matches.push(path_str);
                    }
                }
                Err(e) => {
                    tracing::warn!("Error walking directory: {}", e);
                }
            }
        }

        matches.sort();

        Ok(ToolResult {
            title: format!("Glob '{}'", args.pattern),
            output: matches.join("\n"),
            metadata: serde_json::json!({
                "pattern": args.pattern,
                "matches": matches.len()
            }),
        })
    }
}
