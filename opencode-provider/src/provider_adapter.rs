use crate::trait_::{GenerateRequest, GenerateResponse, Provider as ProviderTrait};
use opencode_core::agent::{Provider, ProviderChunk, ProviderRequest, ProviderResponse};
use async_trait::async_trait;
use futures::Stream;
use futures::StreamExt;
use std::pin::Pin;
use std::sync::Arc;

pub struct ProviderAdapter {
    provider: Arc<dyn ProviderTrait>,
}

impl ProviderAdapter {
    pub fn new(provider: Arc<dyn ProviderTrait>) -> Self {
        Self { provider }
    }

    /// Returns the inner provider (e.g. for as_llm() when using deep agent).
    pub fn inner(&self) -> &Arc<dyn ProviderTrait> {
        &self.provider
    }
}

fn truncate(s: &str, max: usize) -> &str {
    if s.len() <= max {
        s
    } else {
        s.get(..max).unwrap_or(s)
    }
}

#[async_trait]
impl Provider for ProviderAdapter {
    async fn generate(&self, request: ProviderRequest) -> opencode_core::error::Result<ProviderResponse> {
        let message_count = request.messages.len();
        let request_len: usize = request.messages.iter().map(|m| m.content.len()).sum();

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

        tracing::info!(
            message_count,
            request_len,
            "ProviderAdapter: LLM generate request"
        );
        if let Some(m) = messages.last() {
            tracing::debug!(
                last_msg_trunc = %truncate(&m.content, 200),
                "ProviderAdapter: request detail"
            );
        }

        let provider_request = GenerateRequest {
            messages,
            model: request.model,
            temperature: request.temperature.map(|t| t as f64),
            max_tokens: request.max_tokens,
        };

        let response = self.provider.generate(provider_request).await?;

        tracing::info!(
            response_len = response.content.len(),
            usage = ?response.usage,
            "ProviderAdapter: LLM generate response"
        );
        tracing::debug!(
            response_trunc = %truncate(&response.content, 500),
            "ProviderAdapter: response detail"
        );

        Ok(ProviderResponse {
            content: response.content,
            usage: response.usage.map(|u| serde_json::json!({
                "prompt_tokens": u.prompt_tokens,
                "completion_tokens": u.completion_tokens,
                "total_tokens": u.total_tokens,
            })),
        })
    }

    async fn stream(
        &self,
        request: ProviderRequest,
    ) -> opencode_core::error::Result<
        Pin<Box<dyn Stream<Item = opencode_core::error::Result<ProviderChunk>> + Send + Unpin>>,
    > {
        let messages: Vec<crate::trait_::Message> = request
            .messages
            .into_iter()
            .map(|m| crate::trait_::Message {
                role: match m.role {
                    opencode_core::agent::MessageRole::System => crate::trait_::MessageRole::System,
                    opencode_core::agent::MessageRole::User => crate::trait_::MessageRole::User,
                    opencode_core::agent::MessageRole::Assistant => {
                        crate::trait_::MessageRole::Assistant
                    }
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

        let inner = self.provider.stream(provider_request).await?;
        let mapped = inner.map(|r| {
            r.map(|c| ProviderChunk {
                content: c.content,
                done: c.done,
            })
        });
        Ok(Box::pin(mapped))
    }

    fn models(&self) -> &[opencode_core::agent::ModelInfo] {
        // Convert provider models to core models
        &[]
    }
}
