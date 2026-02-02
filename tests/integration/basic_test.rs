#[cfg(test)]
mod tests {
    use opencode_core::session::Session;
    use opencode_tools::ToolRegistry;
    use opencode_tools::tools::register_all_tools;
    use uuid::Uuid;

    #[tokio::test]
    async fn test_session_creation() {
        let session = Session::new(
            Uuid::new_v4().to_string(),
            "test-project".to_string(),
            "/tmp".to_string(),
        );
        assert!(!session.id.is_empty());
        assert_eq!(session.messages.len(), 0);
    }

    #[tokio::test]
    async fn test_tool_registry() {
        let mut registry = ToolRegistry::new();
        register_all_tools(&mut registry);
        
        assert!(registry.get("read").is_some());
        assert!(registry.get("write").is_some());
        assert!(registry.get("ls").is_some());
        assert!(registry.get("grep").is_some());
        assert!(registry.get("glob").is_some());
    }
}
