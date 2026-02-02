use opencode_core::error::{Error, Result};
use opencode_core::tool::{Tool, ToolContext, ToolResult};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::sync::Arc;
use tokio::fs;

#[derive(Debug, Serialize, Deserialize, JsonSchema)]
pub struct PatchArgs {
    pub patch_text: String,
}

pub struct PatchTool;

impl PatchTool {
    pub fn new() -> Arc<Self> {
        Arc::new(Self)
    }
}

#[async_trait::async_trait]
impl Tool for PatchTool {
    fn id(&self) -> &str {
        "patch"
    }

    fn description(&self) -> &str {
        "Apply a patch to modify files. Supports unified diff format."
    }

    fn parameters(&self) -> serde_json::Value {
        serde_json::json!({"type": "object", "properties": {}, "required": []})
    }

    async fn execute(&self, args: Value, _ctx: &ToolContext) -> Result<ToolResult> {
        let args: PatchArgs = serde_json::from_value(args)
            .map_err(|e| Error::Validation(format!("Invalid arguments: {}", e)))?;

        if args.patch_text.is_empty() {
            return Err(Error::Validation("patch_text is required".to_string()));
        }

        let lines: Vec<&str> = args.patch_text.lines().collect();
        let mut file_changes = Vec::new();
        let mut i = 0;

        while i < lines.len() {
            if lines[i].starts_with("---") && i + 1 < lines.len() && lines[i + 1].starts_with("+++") {
                let old_file = lines[i].strip_prefix("--- ").unwrap_or("").trim();
                let new_file = lines[i + 1].strip_prefix("+++ ").unwrap_or("").trim();
                i += 2;

                let mut old_content = String::new();
                let mut new_content = String::new();
                let mut in_hunk = false;

                while i < lines.len() && !lines[i].starts_with("---") {
                    if lines[i].starts_with("@@") {
                        in_hunk = true;
                        i += 1;
                        continue;
                    }
                    if in_hunk {
                        if lines[i].starts_with('-') && !lines[i].starts_with("--") {
                            old_content.push_str(&lines[i][1..]);
                            old_content.push('\n');
                        } else if lines[i].starts_with('+') && !lines[i].starts_with("++") {
                            new_content.push_str(&lines[i][1..]);
                            new_content.push('\n');
                        } else if !lines[i].starts_with("\\") {
                            old_content.push_str(&lines[i]);
                            old_content.push('\n');
                            new_content.push_str(&lines[i]);
                            new_content.push('\n');
                        }
                    }
                    i += 1;
                }

                if !old_file.is_empty() && old_file != "/dev/null" {
                    file_changes.push((old_file.to_string(), old_content, new_content));
                }
            } else {
                i += 1;
            }
        }

        let mut applied = 0;
        for (file_path, _old_content, new_content) in &file_changes {
            if let Ok(_) = fs::write(file_path, new_content.as_bytes()).await {
                applied += 1;
            }
        }

        Ok(ToolResult {
            title: format!("Apply Patch ({} files)", applied),
            output: format!("Applied patch to {} file(s)", applied),
            metadata: serde_json::json!({
                "files_changed": applied,
                "total_files": file_changes.len()
            }),
        })
    }
}
