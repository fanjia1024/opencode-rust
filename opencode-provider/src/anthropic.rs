use crate::adapter::OpenAIProvider;
use crate::common::{build_base_url, validate_api_key};
use crate::trait_::{GenerateRequest, GenerateResponse, ModelInfo, Provider};
use async_trait::async_trait;
use futures::Stream;
use opencode_core::error::{Error, Result};
use reqwest::Client;
use serde::{Deserialize, Serialize};
use std::sync::Arc;

pub struct AnthropicProvider {
    client: Arc<Client>,
    api_key: String,
    base_url: String,
}

impl AnthropicProvider {
    pub fn new(api_key: String, base_url: Option<String>) -> Result<Self> {
        validate_api_key(&api_key)?;
        let client = Client::builder()
            .timeout(std::time::Duration::from_secs(60))
            .build()
            .map_err(|e| Error::Provider(format!("Failed to create HTTP client: {}", e)))?;

        Ok(Self {
            client: Arc::new(client),
            api_key,
            base_url: build_base_url(base_url.as_deref(), "https://api.anthropic.com/v1"),
        })
    }
}

#[derive(Serialize)]
struct AnthropicRequest {
    model: String,
    messages: Vec<AnthropicMessage>,
    max_tokens: u32,
    temperature: Option<f64>,
}

#[derive(Serialize)]
struct AnthropicMessage {
    role: String,
    content: String,
}

#[derive(Deserialize)]
struct AnthropicResponse {
    content: Vec<ContentBlock>,
    usage: Usage,
}

#[derive(Deserialize)]
struct ContentBlock {
    text: String,
}

#[derive(Deserialize)]
struct Usage {
    input_tokens: u32,
    output_tokens: u32,
}

#[async_trait]
impl Provider for AnthropicProvider {
    async fn generate(&self, request: GenerateRequest) -> Result<GenerateResponse> {
        let model = request.model.unwrap_or_else(|| "claude-3-5-sonnet-20241022".to_string());

        let messages: Vec<AnthropicMessage> = request
            .messages
            .into_iter()
            .map(|m| AnthropicMessage {
                role: match m.role {
                    crate::trait_::MessageRole::System => "user".to_string(),
                    crate::trait_::MessageRole::User => "user".to_string(),
                    crate::trait_::MessageRole::Assistant => "assistant".to_string(),
                },
                content: m.content,
            })
            .collect();

        let req = AnthropicRequest {
            model,
            messages,
            max_tokens: request.max_tokens.unwrap_or(4096),
            temperature: request.temperature,
        };

        let response = self
            .client
            .post(format!("{}/messages", self.base_url))
            .header("x-api-key", &self.api_key)
            .header("anthropic-version", "2023-06-01")
            .header("Content-Type", "application/json")
            .json(&req)
            .send()
            .await
            .map_err(|e| Error::Provider(format!("Request failed: {}", e)))?;

        if !response.status().is_success() {
            let status = response.status();
            let text = response.text().await.unwrap_or_else(|_| "Unknown error".to_string());
            return Err(Error::Provider(format!("API error {}: {}", status, text)));
        }

        let anthropic_resp: AnthropicResponse = response
            .json()
            .await
            .map_err(|e| Error::Provider(format!("Failed to parse response: {}", e)))?;

        let content = anthropic_resp
            .content
            .iter()
            .map(|b| b.text.as_str())
            .collect::<Vec<_>>()
            .join("");

        let usage = Some(crate::trait_::Usage {
            prompt_tokens: anthropic_resp.usage.input_tokens,
            completion_tokens: anthropic_resp.usage.output_tokens,
            total_tokens: anthropic_resp.usage.input_tokens + anthropic_resp.usage.output_tokens,
        });

        Ok(GenerateResponse { content, usage })
    }

    async fn stream(
        &self,
        _request: GenerateRequest,
    ) -> Result<Box<dyn Stream<Item = Result<crate::trait_::Chunk>> + Send + Unpin>> {
        Err(Error::Provider("Streaming not yet implemented".to_string()))
    }

    fn models(&self) -> &[ModelInfo] {
        &[]
    }
}
