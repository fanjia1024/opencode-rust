use crate::error::Result;
use async_trait::async_trait;
use schemars::JsonSchema;
use serde_json::Value;

pub struct ToolContext {
    pub session_id: String,
    pub message_id: String,
    pub agent: String,
    pub call_id: Option<String>,
}

pub struct ToolResult {
    pub title: String,
    pub output: String,
    pub metadata: Value,
}

#[async_trait]
pub trait Tool: Send + Sync {
    fn id(&self) -> &str;
    fn description(&self) -> &str;
    fn parameters(&self) -> serde_json::Value;
    async fn execute(&self, args: Value, ctx: &ToolContext) -> Result<ToolResult>;
}
