use opencode_core::error::{Error, Result};
use opencode_core::tool::{Tool, ToolContext, ToolResult};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::sync::Arc;

#[derive(Debug, Serialize, Deserialize, JsonSchema)]
pub struct WebFetchArgs {
    pub url: String,
}

pub struct WebFetchTool {
    client: reqwest::Client,
}

impl WebFetchTool {
    pub fn new() -> Arc<Self> {
        Arc::new(Self {
            client: reqwest::Client::new(),
        })
    }
}

#[async_trait::async_trait]
impl Tool for WebFetchTool {
    fn id(&self) -> &str {
        "webfetch"
    }

    fn description(&self) -> &str {
        "Fetch content from a URL"
    }

    fn parameters(&self) -> serde_json::Value {
        serde_json::json!({"type": "object", "properties": {}, "required": []})
    }

    async fn execute(&self, args: Value, _ctx: &ToolContext) -> Result<ToolResult> {
        let args: WebFetchArgs = serde_json::from_value(args)
            .map_err(|e| Error::Validation(format!("Invalid arguments: {}", e)))?;

        let response = self
            .client
            .get(&args.url)
            .send()
            .await
            .map_err(|e| Error::Tool(format!("Failed to fetch URL: {}", e)))?;

        if !response.status().is_success() {
            return Err(Error::Tool(format!(
                "HTTP error: {}",
                response.status()
            )));
        }

        let content = response
            .text()
            .await
            .map_err(|e| Error::Tool(format!("Failed to read response: {}", e)))?;

        let size = content.len();
        Ok(ToolResult {
            title: format!("Fetch {}", args.url),
            output: content.clone(),
            metadata: serde_json::json!({
                "url": args.url,
                "size": size
            }),
        })
    }
}
