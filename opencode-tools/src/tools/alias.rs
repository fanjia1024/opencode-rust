//! Tool alias: exposes an existing tool under a different id/description for LLM compatibility.

use opencode_core::error::Result;
use opencode_core::tool::{Tool, ToolContext, ToolResult};
use serde_json::Value;
use std::sync::Arc;

/// Wraps a tool and exposes it under a different id and description.
pub struct AliasTool {
    inner: Arc<dyn Tool>,
    id: &'static str,
    description: &'static str,
}

impl AliasTool {
    pub fn new(inner: Arc<dyn Tool>, id: &'static str, description: &'static str) -> Arc<Self> {
        Arc::new(Self {
            inner,
            id,
            description,
        })
    }
}

#[async_trait::async_trait]
impl Tool for AliasTool {
    fn id(&self) -> &str {
        self.id
    }

    fn description(&self) -> &str {
        self.description
    }

    fn parameters(&self) -> Value {
        self.inner.parameters()
    }

    async fn execute(&self, args: Value, ctx: &ToolContext) -> Result<ToolResult> {
        self.inner.execute(args, ctx).await
    }
}
