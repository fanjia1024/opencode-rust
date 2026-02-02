#[cfg(test)]
mod tests {
    use super::super::{grep, read, write};
    use opencode_core::tool::{Tool, ToolContext};
    use serde_json::json;

    #[tokio::test]
    async fn test_read_tool() {
        let tool = read::ReadTool::new();
        let ctx = ToolContext {
            session_id: "test".to_string(),
            message_id: "test".to_string(),
            agent: "test".to_string(),
            call_id: None,
        };

        let args = json!({
            "path": "Cargo.toml"
        });

        let result = tool.execute(args, &ctx).await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_write_tool() {
        let tool = write::WriteTool::new();
        let ctx = ToolContext {
            session_id: "test".to_string(),
            message_id: "test".to_string(),
            agent: "test".to_string(),
            call_id: None,
        };

        let args = json!({
            "path": "/tmp/test_write.txt",
            "content": "test content"
        });

        let result = tool.execute(args, &ctx).await;
        assert!(result.is_ok());
        
        std::fs::remove_file("/tmp/test_write.txt").ok();
    }

    #[tokio::test]
    async fn test_grep_tool() {
        let tool = grep::GrepTool::new();
        let ctx = ToolContext {
            session_id: "test".to_string(),
            message_id: "test".to_string(),
            agent: "test".to_string(),
            call_id: None,
        };

        let args = json!({
            "pattern": "test",
            "path": "Cargo.toml",
            "recursive": false
        });

        let result = tool.execute(args, &ctx).await;
        assert!(result.is_ok());
    }
}
