use opencode_core::error::Result;
use opencode_core::session::{Role, Session};
use serde_json;

fn session_title_for_export(session: &Session) -> String {
    const MAX: usize = 40;
    for msg in &session.messages {
        let s = msg.content.trim();
        if !s.is_empty() {
            return if s.len() <= MAX {
                s.to_string()
            } else {
                format!("{}...", &s[..MAX.saturating_sub(3)])
            };
        }
    }
    "Session".to_string()
}

pub fn export_transcript(session: &Session, format: &str) -> Result<String> {
    match format {
        "json" => {
            serde_json::to_string_pretty(session)
                .map_err(|e| opencode_core::error::Error::Serialization(e))
        }
        "markdown" => {
            let mut output = String::new();
            output.push_str(&format!("# {}\n\n", session_title_for_export(session)));
            output.push_str(&format!("Session ID: {}\n", session.id));
            output.push_str(&format!("Created: {}\n\n", session.created_at));
            output.push_str("## Messages\n\n");

            for msg in &session.messages {
                let role = match msg.role {
                    Role::User => "User",
                    Role::Assistant => "Assistant",
                    Role::System => "System",
                    Role::Tool => "Tool",
                };
                output.push_str(&format!("### {}\n\n", role));
                output.push_str(&format!("{}\n\n", msg.content));
            }

            Ok(output)
        }
        _ => Err(opencode_core::error::Error::Validation(
            format!("Unsupported format: {}", format),
        )),
    }
}
