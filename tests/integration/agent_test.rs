#[cfg(test)]
mod tests {
    use opencode_core::agent::{AgentManager, BuildAgent, Context};
    use opencode_core::session::Session;
    use opencode_provider::OpenAIProvider;
    use opencode_tools::ToolRegistry;
    use opencode_tools::tools::register_all_tools;
    use uuid::Uuid;

    #[tokio::test]
    #[ignore] // Requires API key
    async fn test_agent_with_provider() {
        let api_key = std::env::var("OPENAI_API_KEY").unwrap_or_default();
        if api_key.is_empty() {
            return;
        }

        let provider = OpenAIProvider::new(api_key, None).unwrap();
        let manager = AgentManager::new();
        let mut session = Session::new();
        let ctx = Context {
            session_id: session.id.to_string(),
            message_id: Uuid::new_v4().to_string(),
            agent: "build".to_string(),
        };

        let mut tool_registry = ToolRegistry::new();
        register_all_tools(&mut tool_registry);
        let tools: Vec<std::sync::Arc<dyn opencode_core::tool::Tool>> = vec![];

        let result = manager.process(&ctx, "Hello", &mut session, &provider, &tools).await;
        assert!(result.is_ok());
        assert!(session.messages.len() >= 2);
    }

    #[tokio::test]
    async fn test_agent_manager_switching() {
        let mut manager = AgentManager::new();
        
        assert_eq!(manager.current_agent, "build");
        manager.switch("plan").unwrap();
        assert_eq!(manager.current_agent, "plan");
        manager.switch("general").unwrap();
        assert_eq!(manager.current_agent, "general");
        
        assert!(manager.switch("invalid").is_err());
    }
}
