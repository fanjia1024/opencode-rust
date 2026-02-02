use anyhow::Result;
use crate::config::AppConfig;
use crate::session_store;
use opencode_core::session::Session;
use std::fs;

pub async fn list_sessions() -> Result<()> {
    let config = AppConfig::load().unwrap_or_else(|_| AppConfig::default());
    let session_dir = config.session_dir();
    
    if !session_dir.exists() {
        println!("No sessions found. Session directory does not exist: {:?}", session_dir);
        return Ok(());
    }
    
    let entries = fs::read_dir(&session_dir)?;
    let mut sessions = Vec::new();
    
    for entry in entries {
        let entry = entry?;
        if entry.file_type()?.is_dir() {
            let session_file = entry.path().join("session.json");
            if session_file.exists() {
                match session_store::load_session(&session_file) {
                    Ok(session) => {
                        let title = if session.messages.is_empty() {
                            "New Session".to_string()
                        } else {
                            session.messages.first().map(|m| {
                                let content = &m.content;
                                if content.len() > 50 {
                                    format!("{}...", &content[..50])
                                } else {
                                    content.clone()
                                }
                            }).unwrap_or("New Session".to_string())
                        };
                        
                        sessions.push((entry.file_name().to_string_lossy().to_string(), title, session.updated_at));
                    }
                    Err(_) => {
                        sessions.push((entry.file_name().to_string_lossy().to_string(), "Corrupted Session".to_string(), chrono::Utc::now()));
                    }
                }
            }
        }
    }
    
    sessions.sort_by(|a, b| b.2.cmp(&a.2)); // Sort by updated time, newest first
    
    if sessions.is_empty() {
        println!("No sessions found.");
    } else {
        println!("Found {} session(s):", sessions.len());
        for (id, title, updated_at) in sessions {
            println!("{:<36} {:<50} {}", id, title, updated_at.format("%Y-%m-%d %H:%M:%S"));
        }
    }
    
    Ok(())
}

pub async fn delete_session(session_id: &str) -> Result<()> {
    let config = AppConfig::load().unwrap_or_else(|_| AppConfig::default());
    let session_dir = config.session_dir();
    let session_path = session_dir.join(session_id);
    
    if !session_path.exists() {
        eprintln!("Session '{}' does not exist.", session_id);
        return Ok(());
    }
    
    fs::remove_dir_all(&session_path)?;
    println!("Deleted session: {}", session_id);
    
    Ok(())
}

pub async fn show_session(session_id: &str) -> Result<()> {
    let config = AppConfig::load().unwrap_or_else(|_| AppConfig::default());
    let session_dir = config.session_dir();
    let session_file = session_dir.join(session_id).join("session.json");
    
    if !session_file.exists() {
        eprintln!("Session '{}' does not exist.", session_id);
        return Ok(());
    }
    
    let session: Session = session_store::load_session(&session_file)?;
    
    println!("Session ID: {}", session.id);
    println!("Created: {}", session.created_at);
    println!("Updated: {}", session.updated_at);
    println!("Messages: {}", session.messages.len());
    println!("\nMessages:");
    println!("{}", "=" .repeat(60));
    
    for (i, message) in session.messages.iter().enumerate() {
        let role_prefix = match message.role {
            opencode_core::session::Role::User => "👤 USER",
            opencode_core::session::Role::Assistant => "🤖 ASSISTANT", 
            opencode_core::session::Role::System => "⚙️ SYSTEM",
            opencode_core::session::Role::Tool => "🔧 TOOL",
        };
        
        println!("[{}] {}", i + 1, role_prefix);
        println!("{}", message.content);
        println!("{}", "-" .repeat(60));
    }
    
    Ok(())
}