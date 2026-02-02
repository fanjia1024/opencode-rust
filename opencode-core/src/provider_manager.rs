use crate::error::Result;
use crate::agent::Provider;
use std::collections::HashMap;
use std::sync::Arc;

pub struct ProviderManager {
    providers: HashMap<String, Arc<dyn Provider>>,
    default_provider: Option<String>,
}

impl ProviderManager {
    pub fn new() -> Self {
        Self {
            providers: HashMap::new(),
            default_provider: None,
        }
    }

    pub fn register(&mut self, id: String, provider: Arc<dyn Provider>) {
        if self.default_provider.is_none() {
            self.default_provider = Some(id.clone());
        }
        self.providers.insert(id, provider);
    }

    pub fn get(&self, id: &str) -> Option<&Arc<dyn Provider>> {
        self.providers.get(id)
    }

    pub fn default(&self) -> Option<&Arc<dyn Provider>> {
        self.default_provider
            .as_ref()
            .and_then(|id| self.providers.get(id))
    }

    pub fn set_default(&mut self, id: &str) -> Result<()> {
        if self.providers.contains_key(id) {
            self.default_provider = Some(id.to_string());
            Ok(())
        } else {
            Err(crate::error::Error::Provider(format!("Provider not found: {}", id)))
        }
    }

    pub fn list(&self) -> Vec<String> {
        self.providers.keys().cloned().collect()
    }

    pub fn remove(&mut self, id: &str) -> Option<Arc<dyn Provider>> {
        if self.default_provider.as_deref() == Some(id) {
            self.default_provider = None;
        }
        self.providers.remove(id)
    }
}

impl Default for ProviderManager {
    fn default() -> Self {
        Self::new()
    }
}
