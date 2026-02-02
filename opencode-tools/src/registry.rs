use opencode_core::error::Result;
use opencode_core::tool::{Tool, ToolContext, ToolResult};
use std::collections::HashMap;
use std::sync::Arc;

pub struct ToolRegistry {
    tools: HashMap<String, Arc<dyn Tool>>,
}

impl ToolRegistry {
    pub fn new() -> Self {
        Self {
            tools: HashMap::new(),
        }
    }

    pub fn register(&mut self, tool: Arc<dyn Tool>) {
        self.tools.insert(tool.id().to_string(), tool);
    }

    pub fn get(&self, id: &str) -> Option<&Arc<dyn Tool>> {
        self.tools.get(id)
    }

    pub fn list(&self) -> Vec<String> {
        self.tools.keys().cloned().collect()
    }

    pub async fn execute(
        &self,
        tool_id: &str,
        args: serde_json::Value,
        ctx: &ToolContext,
    ) -> Result<ToolResult> {
        let tool = self
            .get(tool_id)
            .ok_or_else(|| opencode_core::error::Error::Tool(format!("Tool not found: {}", tool_id)))?;
        tool.execute(args, ctx).await
    }
}

impl Default for ToolRegistry {
    fn default() -> Self {
        Self::new()
    }
}
