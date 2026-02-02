use crate::trait_::{GenerateRequest, GenerateResponse, Provider};
use async_trait::async_trait;
use futures::Stream;
use opencode_core::cache::ProviderCache;
use opencode_core::error::Result;
use std::sync::Arc;

pub struct CachedProvider {
    provider: Arc<dyn Provider>,
    cache: Arc<ProviderCache>,
}

impl CachedProvider {
    pub fn new(provider: Arc<dyn Provider>) -> Self {
        Self {
            provider,
            cache: Arc::new(ProviderCache::new()),
        }
    }

    pub fn with_cache(provider: Arc<dyn Provider>, cache: Arc<ProviderCache>) -> Self {
        Self { provider, cache }
    }
}

#[async_trait]
impl Provider for CachedProvider {
    async fn generate(&self, request: GenerateRequest) -> Result<GenerateResponse> {
        use opencode_core::agent::{ProviderRequest, Message, MessageRole};
        use std::collections::hash_map::DefaultHasher;
        use std::hash::{Hash, Hasher};
        
        let core_request = ProviderRequest {
            messages: request.messages.iter().map(|m| Message {
                role: match m.role {
                    crate::trait_::MessageRole::System => MessageRole::System,
                    crate::trait_::MessageRole::User => MessageRole::User,
                    crate::trait_::MessageRole::Assistant => MessageRole::Assistant,
                },
                content: m.content.clone(),
            }).collect(),
            model: request.model.clone(),
            temperature: request.temperature.map(|t| t as f32),
            max_tokens: request.max_tokens,
        };
        
        let mut hasher = DefaultHasher::new();
        let key_str = format!("{:?}", core_request);
        key_str.hash(&mut hasher);
        let cache_key = format!("provider:{}", hasher.finish());
        
        if let Some(cached) = self.cache.get_response(&cache_key) {
            return Ok(GenerateResponse {
                content: cached,
                usage: None,
            });
        }

        let response = self.provider.generate(GenerateRequest {
            messages: request.messages.clone(),
            model: request.model.clone(),
            temperature: request.temperature,
            max_tokens: request.max_tokens,
        }).await?;
        self.cache.cache_response(cache_key, response.content.clone());
        
        Ok(response)
    }

    async fn stream(
        &self,
        request: GenerateRequest,
    ) -> Result<Box<dyn Stream<Item = Result<crate::trait_::Chunk>> + Send + Unpin>> {
        self.provider.stream(request).await
    }

    fn models(&self) -> &[crate::trait_::ModelInfo] {
        self.provider.models()
    }
}
