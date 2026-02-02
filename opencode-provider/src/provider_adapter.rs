use crate::trait_::{GenerateRequest, GenerateResponse, Provider as ProviderTrait};
use opencode_core::agent::{Provider, ProviderRequest, ProviderResponse};
use async_trait::async_trait;
use std::sync::Arc;

pub struct ProviderAdapter {
    provider: Arc<dyn ProviderTrait>,
}

impl ProviderAdapter {
    pub fn new(provider: Arc<dyn ProviderTrait>) -> Self {
        Self { provider }
    }
}

#[async_trait]
impl Provider for ProviderAdapter {
    async fn generate(&self, request: ProviderRequest) -> opencode_core::error::Result<ProviderResponse> {
        let messages: Vec<crate::trait_::Message> = request
            .messages
            .into_iter()
            .map(|m| crate::trait_::Message {
                role: match m.role {
                    opencode_core::agent::MessageRole::System => crate::trait_::MessageRole::System,
                    opencode_core::agent::MessageRole::User => crate::trait_::MessageRole::User,
                    opencode_core::agent::MessageRole::Assistant => crate::trait_::MessageRole::Assistant,
                },
                content: m.content,
            })
            .collect();

        let provider_request = GenerateRequest {
            messages,
            model: request.model,
            temperature: request.temperature.map(|t| t as f64),
            max_tokens: request.max_tokens,
        };

        let response = self.provider.generate(provider_request).await?;

        Ok(ProviderResponse {
            content: response.content,
            usage: response.usage.map(|u| serde_json::json!({
                "prompt_tokens": u.prompt_tokens,
                "completion_tokens": u.completion_tokens,
                "total_tokens": u.total_tokens,
            })),
        })
    }

    fn models(&self) -> &[opencode_core::agent::ModelInfo] {
        // Convert provider models to core models
        &[]
    }
}
