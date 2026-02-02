use opencode_core::error::{Error, Result};

pub fn validate_api_key(api_key: &str) -> Result<()> {
    if api_key.is_empty() {
        return Err(Error::Provider("API key cannot be empty".to_string()));
    }
    Ok(())
}

pub fn build_base_url(base_url: Option<&str>, default: &str) -> String {
    base_url.map(|s| s.to_string()).unwrap_or_else(|| default.to_string())
}
