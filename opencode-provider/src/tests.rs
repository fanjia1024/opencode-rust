#[cfg(test)]
mod tests {
    use crate::message;
    use crate::trait_::MessageRole;
    use chrono::Utc;
    use opencode_core::session::{Message as CoreMessage, Role as CoreRole};

    #[tokio::test]
    async fn test_message_conversion() {
        let core_msg = CoreMessage {
            role: CoreRole::User,
            content: "Hello".to_string(),
            created_at: Utc::now(),
            meta: None,
        };

        let provider_msg = message::to_provider_message(&core_msg);
        assert_eq!(provider_msg.content, "Hello");
        assert!(matches!(provider_msg.role, MessageRole::User));
    }
}
