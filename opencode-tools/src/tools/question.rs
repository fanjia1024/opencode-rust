use opencode_core::error::Result;
use opencode_core::tool::{Tool, ToolContext, ToolResult};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::sync::Arc;

#[derive(Debug, Serialize, Deserialize, JsonSchema)]
pub struct QuestionArgs {
    pub question: String,
}

pub struct QuestionTool;

impl QuestionTool {
    pub fn new() -> Arc<Self> {
        Arc::new(Self)
    }
}

#[async_trait::async_trait]
impl Tool for QuestionTool {
    fn id(&self) -> &str {
        "question"
    }

    fn description(&self) -> &str {
        "Ask a question to the user"
    }

    fn parameters(&self) -> serde_json::Value {
        serde_json::json!({"type": "object", "properties": {}, "required": []})
    }

    async fn execute(&self, args: Value, ctx: &ToolContext) -> Result<ToolResult> {
        let args: QuestionArgs = serde_json::from_value(args)
            .map_err(|e| opencode_core::error::Error::Validation(format!("Invalid arguments: {}", e)))?;

        Ok(ToolResult {
            title: "Question".to_string(),
            output: format!("Question: {}\n(Waiting for user response)", args.question),
            metadata: serde_json::json!({
                "question": args.question,
                "session_id": ctx.session_id,
                "message_id": ctx.message_id
            }),
        })
    }
}
