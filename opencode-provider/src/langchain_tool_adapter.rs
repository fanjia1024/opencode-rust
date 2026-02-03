use async_trait::async_trait;
use langchain_ai_rust::error::ToolError as LangChainToolError;
use opencode_core::tool::{Tool, ToolContext};
use serde_json::Value;
use std::sync::Arc;

pub struct LangChainToolAdapter {
    tool: Arc<dyn Tool>,
}

impl LangChainToolAdapter {
    pub fn new(tool: Arc<dyn Tool>) -> Self {
        Self { tool }
    }
}

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
