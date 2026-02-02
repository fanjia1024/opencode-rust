use anyhow::Result;
use opencode_core::session::{Session, Message, Role};
use opencode_provider::ProviderAdapter;
use std::sync::Arc;
use chrono::Utc;
use crate::config::AppConfig;
use crate::session_store;

pub async fn run_command(command: &str) -> Result<()> {
    println!("Processing command: {}", command);
    
    // Load configuration
    let config = AppConfig::load().unwrap_or_else(|_| AppConfig::default());
    let session_dir = config.session_dir();
    
    // Initialize provider
    #[cfg(not(feature = "langchain"))]
    {
        println!("Error: langchain-rust feature not enabled. Rebuild with --features langchain");
        return Ok(());
    }
    
    #[cfg(feature = "langchain")]
    {
        let provider_info = config.get_default_provider();
        let provider_type = provider_info
            .as_ref()
            .map(|p| p.provider_type.clone())
            .unwrap_or_else(|| "openai".to_string());
        let base_url = provider_info.as_ref().and_then(|p| p.base_url.clone());
        let model = provider_info.as_ref().and_then(|p| p.model.clone());

        let api_key = provider_info
            .as_ref()
            .and_then(|p| p.api_key.clone())
            .or_else(|| std::env::var("OPENAI_API_KEY").ok())
            .or_else(|| std::env::var("OPENCODE_OPENAI_API_KEY").ok())
            .unwrap_or_else(|| "".to_string());

        let provider: Arc<dyn opencode_provider::Provider> = match provider_type.as_str() {
            "openai" => {
                if api_key.trim().is_empty() {
                    eprintln!("Error: No API key configured. Please set OPENAI_API_KEY or OPENCODE_OPENAI_API_KEY environment variable.");
                    std::process::exit(1);
                }
                match opencode_provider::LangChainAdapter::from_openai(api_key, base_url, model) {
                    Ok(adapter) => Arc::new(adapter),
                    Err(e) => {
                        eprintln!("Error initializing OpenAI provider: {}", e);
                        std::process::exit(1);
                    }
                }
            }
            "ollama" => {
                match opencode_provider::LangChainAdapter::from_ollama(base_url, model) {
                    Ok(adapter) => Arc::new(adapter),
                    Err(e) => {
                        eprintln!("Error initializing Ollama provider: {}", e);
                        std::process::exit(1);
                    }
                }
            }
            "qwen" => {
                if api_key.trim().is_empty() {
                    eprintln!("Error: No API key configured for Qwen. Please set the API key.");
                    std::process::exit(1);
                }
                match opencode_provider::LangChainAdapter::from_qwen(api_key, base_url, model) {
                    Ok(adapter) => Arc::new(adapter),
                    Err(e) => {
                        eprintln!("Error initializing Qwen provider: {}", e);
                        std::process::exit(1);
                    }
                }
            }
            "anthropic" => {
                if api_key.trim().is_empty() {
                    eprintln!("Error: No API key configured. Please set the API key.");
                    std::process::exit(1);
                }
                match opencode_provider::LangChainAdapter::from_anthropic(api_key) {
                    Ok(adapter) => Arc::new(adapter),
                    Err(e) => {
                        eprintln!("Error initializing Anthropic provider: {}", e);
                        std::process::exit(1);
                    }
                }
            }
            _ => {
                eprintln!("Unsupported provider type: {}", provider_type);
                std::process::exit(1);
            }
        };

        let provider_adapter = ProviderAdapter::new(provider);
        
        // Create or load a temporary session for this command
        let session_id = uuid::Uuid::new_v4().to_string();
        let session_file = session_dir.join(&session_id).join("session.json");
        
        // Create a new session for the command
        let mut session = Session::new();
        
        // Add the user command as a message
        let user_message = Message {
            role: Role::User,
            content: command.to_string(),
            created_at: Utc::now(),
            meta: None,
        };
        session.push_message(user_message);
        
        // Process with the provider
        let messages: Vec<opencode_core::agent::Message> = session
            .messages
            .iter()
            .map(|m| opencode_core::agent::Message {
                role: match m.role {
                    Role::User => opencode_core::agent::MessageRole::User,
                    Role::Assistant => opencode_core::agent::MessageRole::Assistant,
                    Role::System => opencode_core::agent::MessageRole::System,
                    Role::Tool => opencode_core::agent::MessageRole::System,
                },
                content: m.content.clone(),
            })
            .collect();

        let request = opencode_core::agent::ProviderRequest {
            messages,
            model: None,
            temperature: Some(0.7),
            max_tokens: Some(4096),
        };

        use opencode_core::agent::Provider;  // Import the Provider trait
        
        match provider_adapter.generate(request).await {
            Ok(response) => {
                // Create assistant message
                let assistant_message = Message {
                    role: Role::Assistant,
                    content: response.content.clone(),
                    created_at: Utc::now(),
                    meta: None,
                };
                session.push_message(assistant_message);
                
                // Save session if path is valid
                if let Some(parent) = session_file.parent() {
                    let _ = std::fs::create_dir_all(parent); // Ignore errors for CLI
                }
                
                // Attempt to save session
                if let Err(e) = session_store::save_session(&session_file, &session) {
                    eprintln!("Warning: Could not save session: {}", e);
                }
                
                println!("{}", response.content);
            }
            Err(e) => {
                eprintln!("Error processing command: {}", e);
                std::process::exit(1);
            }
        }
    }
    
    Ok(())
}
