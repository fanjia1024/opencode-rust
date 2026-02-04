use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

use crate::ids::SessionId;
use super::message::Message;

/// Pure data representation of a session
/// Contains only essential fields for serialization and inspection
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Session {
    pub id: SessionId,
    pub messages: Vec<Message>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

impl Session {
    pub fn new() -> Self {
        let now = Utc::now();
        Self {
            id: SessionId::new(),
            messages: Vec::new(),
            created_at: now,
            updated_at: now,
        }
    }

    /// Create a session with a specific ID (e.g. to match folder name / UI session_id).
    pub fn with_id(id: SessionId) -> Self {
        let now = Utc::now();
        Self {
            id,
            messages: Vec::new(),
            created_at: now,
            updated_at: now,
        }
    }

    /// Add a message to the session
    pub fn push_message(&mut self, message: Message) {
        self.messages.push(message);
        self.updated_at = Utc::now();
    }

    /// Check if the session has no messages
    pub fn is_empty(&self) -> bool {
        self.messages.is_empty()
    }

    /// Get the number of messages in the session
    pub fn len(&self) -> usize {
        self.messages.len()
    }
}

impl Default for Session {
    fn default() -> Self {
        Self::new()
    }
}
