#[cfg(feature = "langchain")]
use opencode_core::tool::{Tool, ToolContext};
#[cfg(feature = "langchain")]
use serde_json::Value;
#[cfg(feature = "langchain")]
use std::sync::Arc;
#[cfg(feature = "langchain")]
use async_trait::async_trait;
#[cfg(feature = "langchain")]
use langchain_ai_rust::error::ToolError as LangChainToolError;

#[cfg(feature = "langchain")]
pub struct LangChainToolAdapter {
    tool: Arc<dyn Tool>,
}

#[cfg(feature = "langchain")]
impl LangChainToolAdapter {
    pub fn new(tool: Arc<dyn Tool>) -> Self {
        Self { tool }
    }
}

#[cfg(feature = "langchain")]
#[async_trait]
impl langchain_ai_rust::tools::Tool for LangChainToolAdapter {
    fn name(&self) -> String {
        self.tool.id().to_string()
    }

    fn description(&self) -> String {
        self.tool.description().to_string()
    }

    fn parameters(&self) -> Value {
        // Return the parameters schema from the tool
        self.tool.parameters()
    }

    async fn run(&self, input: Value) -> Result<String, LangChainToolError> {
        let tool = self.tool.clone();
        let ctx = ToolContext {
            session_id: "langchain".to_string(),
            message_id: "langchain".to_string(),
            agent: "langchain".to_string(),
            call_id: None,
        };

        match tool.execute(input, &ctx).await {
            Ok(result) => {
                let output = serde_json::json!({
                    "output": result.output,
                    "title": result.title,
                    "metadata": result.metadata
                });
                Ok(output.to_string())
            }
            Err(e) => Err(LangChainToolError::ExecutionError(e.to_string())),
        }
    }
}

#[cfg(not(feature = "langchain"))]
pub struct LangChainToolAdapter;

#[cfg(not(feature = "langchain"))]
impl LangChainToolAdapter {
    pub fn new(_tool: std::sync::Arc<dyn opencode_core::tool::Tool>) -> Self {
        Self
    }
}
