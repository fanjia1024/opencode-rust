use anyhow::Result;
use opencode_core::config::{Config as CoreConfig, ProviderConfig as CoreProviderConfig};
use std::path::PathBuf;

#[derive(Clone)]
pub struct AppConfig {
    config: CoreConfig,
}

impl AppConfig {
    pub fn load() -> Result<Self> {
        let config = CoreConfig::load()?;
        Ok(Self { config })
    }

    pub fn save(&self) -> Result<()> {
        let config_dir = CoreConfig::config_dir()?;
        let config_file = config_dir.join("config.json");
        let content = serde_json::to_string_pretty(&self.config)?;
        std::fs::write(&config_file, content)?;
        Ok(())
    }

    pub fn get_provider_config(&self, provider_id: Option<&str>) -> Option<ProviderInfo> {
        let id = provider_id.unwrap_or("default");
        self.config
            .providers
            .iter()
            .find(|p| p.id == id)
            .map(|p| ProviderInfo {
                provider_type: p.provider_type.clone(),
                api_key: p.api_key.clone(),
                base_url: p.base_url.clone(),
                model: p.model.clone(),
            })
    }

    pub fn get_default_provider(&self) -> Option<ProviderInfo> {
        self.get_provider_config(Some("default")).or_else(|| {
            self.config.providers.first().map(|p| ProviderInfo {
                provider_type: p.provider_type.clone(),
                api_key: p.api_key.clone(),
                base_url: p.base_url.clone(),
                model: p.model.clone(),
            })
        })
    }

    /// List all configured providers (id and info) for TUI display.
    pub fn list_providers(&self) -> Vec<ProviderListItem> {
        self.config
            .providers
            .iter()
            .map(|p| ProviderListItem {
                id: p.id.clone(),
                provider_type: p.provider_type.clone(),
                model: p.model.clone(),
                base_url: p.base_url.clone(),
            })
            .collect()
    }

    /// Set the default provider by id (moves it to the front of the list so it is used as default).
    pub fn set_default_provider_id(&mut self, id: &str) -> Result<()> {
        if let Some(pos) = self.config.providers.iter().position(|p| p.id == id) {
            if pos > 0 {
                let item = self.config.providers.remove(pos);
                self.config.providers.insert(0, item);
                self.save()?;
            }
        }
        Ok(())
    }

    pub fn set_provider_config(
        &mut self,
        provider_id: &str,
        provider_type: String,
        api_key: String,
        base_url: Option<String>,
        model: Option<String>,
    ) -> Result<()> {
        let provider_config = CoreProviderConfig {
            id: provider_id.to_string(),
            provider_type,
            api_key: Some(api_key),
            base_url,
            model,
        };

        if let Some(existing) = self
            .config
            .providers
            .iter_mut()
            .find(|p| p.id == provider_id)
        {
            *existing = provider_config;
        } else {
            self.config.providers.push(provider_config);
        }

        self.save()?;
        Ok(())
    }

    pub fn config_dir() -> Result<PathBuf> {
        CoreConfig::config_dir().map_err(|e| anyhow::anyhow!("{}", e))
    }

    pub fn session_dir(&self) -> PathBuf {
        self.config.storage.session_dir.clone()
    }
}

pub struct ProviderInfo {
    pub provider_type: String,
    pub api_key: Option<String>,
    pub base_url: Option<String>,
    pub model: Option<String>,
}

/// One row for the providers list dialog (id, type, model; no api_key).
pub struct ProviderListItem {
    pub id: String,
    pub provider_type: String,
    pub model: Option<String>,
    pub base_url: Option<String>,
}

impl Default for AppConfig {
    fn default() -> Self {
        Self {
            config: CoreConfig::default(),
        }
    }
}
