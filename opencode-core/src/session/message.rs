use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

use super::role::Role;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Message {
    pub role: Role,
    pub content: String,
    pub created_at: DateTime<Utc>,

    /// Optional metadata for inspection/debugging only.
    /// Never used for control flow.
    #[serde(default)]
    pub meta: Option<MessageMeta>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MessageMeta {
    pub tool_name: Option<String>,
    pub tool_call_id: Option<String>,
}
