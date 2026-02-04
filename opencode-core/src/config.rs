use crate::error::{Error, Result};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::{Path, PathBuf};

/// Per-command config in JSON (command: { "name": { template, description?, agent?, model?, subtask? } }).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CommandOption {
    pub template: String,
    #[serde(default)]
    pub description: Option<String>,
    #[serde(default)]
    pub agent: Option<String>,
    #[serde(default)]
    pub model: Option<String>,
    #[serde(default)]
    pub subtask: Option<bool>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Config {
    pub providers: Vec<ProviderConfig>,
    pub agents: Vec<AgentConfig>,
    pub storage: StorageConfig,
    /// Selected agent name for the desktop app (persisted in workspace config).
    #[serde(default)]
    pub default_agent: Option<String>,
    /// Custom commands: name -> { template, description?, agent?, model?, subtask? }.
    #[serde(default)]
    pub command: Option<HashMap<String, CommandOption>>,
    /// Max agent steps per turn (deep agent). When unset, langchain default (10) is used. Set to e.g. 25 to allow longer runs.
    #[serde(default)]
    pub max_agent_iterations: Option<i32>,
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

    /// Load config from a workspace directory. Config file: `workspace_root/.opencode/config.json`.
    /// Session and config dirs are forced to workspace-relative paths.
    pub fn load_from_dir(workspace_root: &Path) -> Result<Self> {
        let config_file = workspace_root.join(".opencode").join("config.json");
        let session_dir = workspace_root.join(".opencode").join("sessions");
        let config_dir = workspace_root.join(".opencode");

        if config_file.exists() {
            let content = std::fs::read_to_string(&config_file)?;
            let mut config: Config = serde_json::from_str(&content)?;
            config.storage.session_dir = session_dir;
            config.storage.config_dir = config_dir;
            Ok(config)
        } else {
            Ok(Self::default_for_workspace(workspace_root))
        }
    }

    /// Save config to a workspace directory. Writes to `workspace_root/.opencode/config.json`.
    pub fn save_to_dir(&self, workspace_root: &Path) -> Result<()> {
        let opencode_dir = workspace_root.join(".opencode");
        std::fs::create_dir_all(&opencode_dir)?;
        let config_file = opencode_dir.join("config.json");

        let mut config = self.clone();
        config.storage.session_dir = workspace_root.join(".opencode").join("sessions");
        config.storage.config_dir = workspace_root.join(".opencode");

        let content = serde_json::to_string_pretty(&config)?;
        std::fs::write(&config_file, content)?;
        Ok(())
    }

    fn default_for_workspace(workspace_root: &Path) -> Self {
        Self {
            providers: Vec::new(),
            agents: Vec::new(),
            storage: StorageConfig {
                session_dir: workspace_root.join(".opencode").join("sessions"),
                config_dir: workspace_root.join(".opencode"),
            },
            default_agent: None,
            command: None,
            max_agent_iterations: None,
        }
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
            default_agent: None,
            command: None,
            max_agent_iterations: None,
        }
    }
}
