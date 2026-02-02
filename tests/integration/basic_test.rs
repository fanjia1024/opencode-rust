#[cfg(test)]
mod tests {
use opencode_core::session::Session;
use opencode_tools::ToolRegistry;
use opencode_tools::tools::register_all_tools;

    #[tokio::test]
    async fn test_session_creation() {
        let session = Session::new();
        assert_eq!(session.messages.len(), 0);
        assert!(session.is_empty());
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
