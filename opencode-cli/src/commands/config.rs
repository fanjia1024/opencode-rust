use anyhow::Result;
use crate::config::AppConfig;

pub async fn show_config() -> Result<()> {
    let config = AppConfig::load().unwrap_or_else(|_| AppConfig::default());
    println!("Configuration:");
    println!("  Session Directory: {:?}", config.session_dir());
    
    if let Some(provider_info) = config.get_default_provider() {
        println!("  Default Provider: {}", provider_info.provider_type);
        println!("  Model: {}", provider_info.model.unwrap_or_else(|| "default".to_string()));
        println!("  Base URL: {}", provider_info.base_url.unwrap_or_else(|| "default".to_string()));
    } else {
        println!("  Default Provider: Not configured");
    }
    
    Ok(())
}

pub async fn reset_config() -> Result<()> {
    let config = AppConfig::default();
    config.save()?;
    println!("Configuration reset to defaults.");
    println!("  Session Directory: {:?}", config.session_dir());
    Ok(())
}