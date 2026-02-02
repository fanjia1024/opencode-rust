#[cfg(test)]
mod tests {
    use opencode_core::tool::ToolContext;
    use opencode_tools::ToolRegistry;
    use opencode_tools::tools::register_all_tools;
    use serde_json::json;

    #[tokio::test]
    async fn test_read_tool_execution() {
        let mut registry = ToolRegistry::new();
        register_all_tools(&mut registry);
        
        let ctx = ToolContext {
            session_id: "test".to_string(),
            message_id: "test".to_string(),
            agent: "test".to_string(),
            call_id: None,
        };

        let args = json!({
            "path": "Cargo.toml"
        });

        let result = registry.execute("read", args, &ctx).await;
        assert!(result.is_ok());
        let result = result.unwrap();
        assert!(!result.output.is_empty());
    }

    #[tokio::test]
    async fn test_write_tool_execution() {
        let mut registry = ToolRegistry::new();
        register_all_tools(&mut registry);
        
        let ctx = ToolContext {
            session_id: "test".to_string(),
            message_id: "test".to_string(),
            agent: "test".to_string(),
            call_id: None,
        };

        let test_file = "/tmp/test_write_tool.txt";
        let args = json!({
            "path": test_file,
            "content": "test content"
        });

        let result = registry.execute("write", args, &ctx).await;
        assert!(result.is_ok());
        
        let content = std::fs::read_to_string(test_file).unwrap();
        assert_eq!(content, "test content");
        
        std::fs::remove_file(test_file).ok();
    }

    #[tokio::test]
    async fn test_grep_tool_execution() {
        let mut registry = ToolRegistry::new();
        register_all_tools(&mut registry);
        
        let ctx = ToolContext {
            session_id: "test".to_string(),
            message_id: "test".to_string(),
            agent: "test".to_string(),
            call_id: None,
        };

        let args = json!({
            "pattern": "version",
            "path": "Cargo.toml",
            "recursive": false
        });

        let result = registry.execute("grep", args, &ctx).await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_multiedit_tool_execution() {
        let mut registry = ToolRegistry::new();
        register_all_tools(&mut registry);
        
        let test_file = "/tmp/test_multiedit.txt";
        std::fs::write(test_file, "line1\nline2\nline3").unwrap();
        
        let ctx = ToolContext {
            session_id: "test".to_string(),
            message_id: "test".to_string(),
            agent: "test".to_string(),
            call_id: None,
        };

        let args = json!({
            "file_path": test_file,
            "edits": [
                {
                    "file_path": test_file,
                    "old_string": "line2",
                    "new_string": "line2_modified",
                    "replace_all": false
                }
            ]
        });

        let result = registry.execute("multiedit", args, &ctx).await;
        assert!(result.is_ok());
        
        let content = std::fs::read_to_string(test_file).unwrap();
        assert!(content.contains("line2_modified"));
        
        std::fs::remove_file(test_file).ok();
    }
}
