use chrono::Utc;
use opencode_core::session::{Message as CoreMessage, Role as CoreRole};
use crate::trait_::{Message, MessageRole};

pub fn to_provider_message(msg: &CoreMessage) -> Message {
    let role = match msg.role {
        CoreRole::User => MessageRole::User,
        CoreRole::Assistant => MessageRole::Assistant,
        CoreRole::System => MessageRole::System,
        CoreRole::Tool => MessageRole::System,
    };
    Message {
        role,
        content: msg.content.clone(),
    }
}

pub fn from_provider_message(msg: &Message) -> CoreMessage {
    let role = match msg.role {
        MessageRole::User => CoreRole::User,
        MessageRole::Assistant => CoreRole::Assistant,
        MessageRole::System => CoreRole::System,
    };

    CoreMessage {
        role,
        content: msg.content.clone(),
        created_at: Utc::now(),
        meta: None,
    }
}
