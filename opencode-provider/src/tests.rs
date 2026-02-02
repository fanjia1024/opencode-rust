#[cfg(test)]
mod tests {
    use super::*;
    use crate::trait_::{GenerateRequest, Message, MessageRole};

    #[tokio::test]
    async fn test_message_conversion() {
        use opencode_core::session::{Message as CoreMessage, MessageRole as CoreRole};
        use chrono::Utc;
        use uuid::Uuid;

        let core_msg = CoreMessage {
            id: Uuid::new_v4().to_string(),
            role: CoreRole::User,
            content: "Hello".to_string(),
            timestamp: Utc::now(),
        };

        let provider_msg = message::to_provider_message(&core_msg);
        assert_eq!(provider_msg.content, "Hello");
        assert!(matches!(provider_msg.role, MessageRole::User));
    }
}
