use opencode_core::error::Result;
use opencode_core::session::Session;
use serde_json;

pub fn export_transcript(session: &Session, format: &str) -> Result<String> {
    match format {
        "json" => {
            serde_json::to_string_pretty(session)
                .map_err(|e| opencode_core::error::Error::Serialization(e))
        }
        "markdown" => {
            let mut output = String::new();
            output.push_str(&format!("# {}\n\n", session.title));
            output.push_str(&format!("Session ID: {}\n", session.id));
            output.push_str(&format!("Created: {}\n\n", session.created_at));
            output.push_str("## Messages\n\n");
            
            for msg in &session.messages {
                let role = match msg.role {
                    opencode_core::session::MessageRole::User => "User",
                    opencode_core::session::MessageRole::Assistant => "Assistant",
                    opencode_core::session::MessageRole::System => "System",
                };
                output.push_str(&format!("### {}\n\n", role));
                output.push_str(&format!("{}\n\n", msg.content));
            }
            
            Ok(output)
        }
        _ => Err(opencode_core::error::Error::Validation(
            format!("Unsupported format: {}", format)
        ))
    }
}
