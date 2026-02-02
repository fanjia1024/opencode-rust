use crate::error::{Error, Result};
use serde::{Deserialize, Serialize};
use std::path::PathBuf;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Config {
    pub providers: Vec<ProviderConfig>,
    pub agents: Vec<AgentConfig>,
    pub storage: StorageConfig,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProviderConfig {
    pub id: String,
    pub provider_type: String,
    pub api_key: Option<String>,
    pub base_url: Option<String>,
    #[serde(default)]
    pub model: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgentConfig {
    pub name: String,
    pub mode: String,
    pub model: Option<String>,
    pub provider: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StorageConfig {
    pub session_dir: PathBuf,
    pub config_dir: PathBuf,
}

impl Config {
    pub fn load() -> Result<Self> {
        let config_dir = Self::config_dir()?;
        let config_file = config_dir.join("config.json");
        
        if config_file.exists() {
            let content = std::fs::read_to_string(&config_file)?;
            Ok(serde_json::from_str(&content)?)
        } else {
            Ok(Self::default())
        }
    }

    pub fn config_dir() -> Result<PathBuf> {
        let dir = dirs::config_dir()
            .ok_or_else(|| Error::Config("Cannot find config directory".to_string()))?
            .join("opencode");
        std::fs::create_dir_all(&dir)?;
        Ok(dir)
    }
}

impl Default for Config {
    fn default() -> Self {
        let config_dir = Self::config_dir().unwrap_or_else(|_| PathBuf::from("."));
        Self {
            providers: Vec::new(),
            agents: Vec::new(),
            storage: StorageConfig {
                session_dir: std::env::current_dir()
                    .unwrap_or_else(|_| PathBuf::from("."))
                    .join(".opencode")
                    .join("sessions"),
                config_dir: config_dir.clone(),
            },
        }
    }
}
