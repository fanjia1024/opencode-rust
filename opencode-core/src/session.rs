use crate::error::Result;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::VecDeque;
use std::path::PathBuf;
use tokio::fs;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum MessageRole {
    User,
    Assistant,
    System,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Message {
    pub id: String,
    pub role: MessageRole,
    pub content: String,
    pub timestamp: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Session {
    pub id: String,
    pub project_id: String,
    pub directory: String,
    pub title: String,
    pub messages: VecDeque<Message>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

impl Session {
    pub fn new(id: String, project_id: String, directory: String) -> Self {
        let now = Utc::now();
        Self {
            id,
            project_id,
            directory,
            title: format!("New session - {}", now.to_rfc3339()),
            messages: VecDeque::new(),
            created_at: now,
            updated_at: now,
        }
    }

    pub fn add_message(&mut self, message: Message) {
        self.messages.push_back(message);
        self.updated_at = Utc::now();
    }

    pub fn compact(&mut self) -> Result<()> {
        if self.messages.len() > 100 {
            let keep = self.messages.len() - 50;
            self.messages.drain(..keep);
        }
        Ok(())
    }

    pub async fn save(&self, base_dir: &PathBuf) -> Result<()> {
        let session_dir = base_dir.join(&self.id);
        fs::create_dir_all(&session_dir).await?;
        
        let session_file = session_dir.join("session.json");
        let content = serde_json::to_string_pretty(self)?;
        fs::write(&session_file, content).await?;
        
        Ok(())
    }

    pub async fn load(id: &str, base_dir: &PathBuf) -> Result<Self> {
        let session_file = base_dir.join(id).join("session.json");
        let content = fs::read_to_string(&session_file).await?;
        let session: Session = serde_json::from_str(&content)?;
        Ok(session)
    }
}
