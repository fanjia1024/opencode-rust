use opencode_core::error::{Error, Result};
use opencode_core::tool::{Tool, ToolContext, ToolResult};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::sync::Arc;

#[derive(Debug, Serialize, Deserialize, JsonSchema)]
pub struct CodeSearchArgs {
    pub query: String,
    pub tokens_num: Option<u32>,
}

pub struct CodeSearchTool {
    client: reqwest::Client,
}

impl CodeSearchTool {
    pub fn new() -> Arc<Self> {
        Arc::new(Self {
            client: reqwest::Client::new(),
        })
    }
}

#[async_trait::async_trait]
impl Tool for CodeSearchTool {
    fn id(&self) -> &str {
        "codesearch"
    }

    fn description(&self) -> &str {
        "Search for relevant code context, APIs, libraries, and SDKs"
    }

    fn parameters(&self) -> serde_json::Value {
        serde_json::json!({"type": "object", "properties": {}, "required": []})
    }

    async fn execute(&self, args: Value, _ctx: &ToolContext) -> Result<ToolResult> {
        let args: CodeSearchArgs = serde_json::from_value(args)
            .map_err(|e| Error::Validation(format!("Invalid arguments: {}", e)))?;

        let tokens_num = args.tokens_num.unwrap_or(5000).min(50000).max(1000);

        let request = serde_json::json!({
            "jsonrpc": "2.0",
            "id": 1,
            "method": "mcp_code_search",
            "params": {
                "name": "code_search",
                "arguments": {
                    "query": args.query,
                    "tokensNum": tokens_num
                }
            }
        });

        let response = self
            .client
            .post("https://mcp.exa.ai/mcp")
            .json(&request)
            .send()
            .await
            .map_err(|e| Error::Tool(format!("Failed to search code: {}", e)))?;

        if !response.status().is_success() {
            return Err(Error::Tool(format!("Code search failed with status: {}", response.status())));
        }

        let result: serde_json::Value = response
            .json()
            .await
            .map_err(|e| Error::Tool(format!("Failed to parse response: {}", e)))?;

        let content = result
            .get("result")
            .and_then(|r| r.get("content"))
            .and_then(|c| c.as_array())
            .map(|arr| {
                arr.iter()
                    .filter_map(|item| item.get("text").and_then(|t| t.as_str()))
                    .collect::<Vec<_>>()
                    .join("\n\n")
            })
            .unwrap_or_else(|| "No results found".to_string());

        Ok(ToolResult {
            title: format!("Code Search: {}", args.query),
            output: content,
            metadata: serde_json::json!({
                "query": args.query,
                "tokens_num": tokens_num
            }),
        })
    }
}
