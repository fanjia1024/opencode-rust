use opencode_core::error::{Error, Result};
use opencode_core::tool::{Tool, ToolContext, ToolResult};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::sync::Arc;

#[derive(Debug, Serialize, Deserialize, JsonSchema)]
pub struct WebSearchArgs {
    pub query: String,
    pub max_results: Option<usize>,
}

pub struct WebSearchTool {
    client: reqwest::Client,
}

impl WebSearchTool {
    pub fn new() -> Arc<Self> {
        Arc::new(Self {
            client: reqwest::Client::new(),
        })
    }
}

#[async_trait::async_trait]
impl Tool for WebSearchTool {
    fn id(&self) -> &str {
        "websearch"
    }

    fn description(&self) -> &str {
        "Search the web for information"
    }

    fn parameters(&self) -> serde_json::Value {
        serde_json::json!({"type": "object", "properties": {}, "required": []})
    }

    async fn execute(&self, args: Value, _ctx: &ToolContext) -> Result<ToolResult> {
        let args: WebSearchArgs = serde_json::from_value(args)
            .map_err(|e| Error::Validation(format!("Invalid arguments: {}", e)))?;

        let max_results = args.max_results.unwrap_or(10);
        
        let search_url = format!("https://www.google.com/search?q={}", 
            urlencoding::encode(&args.query));

        let response = self
            .client
            .get(&search_url)
            .header("User-Agent", "Mozilla/5.0")
            .send()
            .await
            .map_err(|e| Error::Tool(format!("Failed to search: {}", e)))?;

        if !response.status().is_success() {
            return Err(Error::Tool(format!("Search failed with status: {}", response.status())));
        }

        let content = response
            .text()
            .await
            .map_err(|e| Error::Tool(format!("Failed to read response: {}", e)))?;

        Ok(ToolResult {
            title: format!("Web Search: {}", args.query),
            output: format!("Search results for: {}\n\n(Request completed; results are not parsed. For production use, integrate a search API such as Tavily or SerpAPI.)", args.query),
            metadata: serde_json::json!({
                "query": args.query,
                "max_results": max_results
            }),
        })
    }
}
