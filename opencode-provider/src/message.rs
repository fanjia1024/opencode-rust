use crate::trait_::{Message, MessageRole};
use opencode_core::session::{Message as CoreMessage, MessageRole as CoreMessageRole};

pub fn to_provider_message(msg: &CoreMessage) -> Message {
    let role = match msg.role {
        CoreMessageRole::User => MessageRole::User,
        CoreMessageRole::Assistant => MessageRole::Assistant,
        CoreMessageRole::System => MessageRole::System,
    };
    Message {
        role,
        content: msg.content.clone(),
    }
}

pub fn from_provider_message(msg: &Message) -> CoreMessage {
    use chrono::Utc;
    use uuid::Uuid;
    
    let role = match msg.role {
        MessageRole::User => CoreMessageRole::User,
        MessageRole::Assistant => CoreMessageRole::Assistant,
        MessageRole::System => CoreMessageRole::System,
    };
    
    CoreMessage {
        id: Uuid::new_v4().to_string(),
        role,
        content: msg.content.clone(),
        timestamp: Utc::now(),
    }
}
