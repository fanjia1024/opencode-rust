use crate::common::{build_base_url, validate_api_key};
use crate::trait_::{GenerateRequest, GenerateResponse, ModelInfo, Provider};
use async_trait::async_trait;
use futures::Stream;
use opencode_core::error::{Error, Result};
use reqwest::Client;
use serde::{Deserialize, Serialize};
use std::sync::Arc;

pub struct OpenAIProvider {
    client: Arc<Client>,
    api_key: String,
    base_url: String,
}

impl OpenAIProvider {
    pub fn new(api_key: String, base_url: Option<String>) -> Result<Self> {
        validate_api_key(&api_key)?;
        let client = Client::builder()
            .timeout(std::time::Duration::from_secs(60))
            .build()
            .map_err(|e| Error::Provider(format!("Failed to create HTTP client: {}", e)))?;
        
        Ok(Self {
            client: Arc::new(client),
            api_key,
            base_url: build_base_url(base_url.as_deref(), "https://api.openai.com/v1"),
        })
    }
}

#[derive(Serialize)]
struct OpenAIRequest {
    model: String,
    messages: Vec<OpenAIMessage>,
    temperature: Option<f64>,
    max_tokens: Option<u32>,
    stream: bool,
}

#[derive(Serialize, Deserialize)]
struct OpenAIMessage {
    role: String,
    content: String,
}

#[derive(Deserialize)]
struct OpenAIResponse {
    choices: Vec<Choice>,
    usage: Option<Usage>,
}

#[derive(Deserialize)]
struct Choice {
    message: OpenAIMessage,
}

#[derive(Deserialize)]
struct Usage {
    prompt_tokens: u32,
    completion_tokens: u32,
    total_tokens: u32,
}

#[async_trait]
impl Provider for OpenAIProvider {
    async fn generate(&self, request: GenerateRequest) -> Result<GenerateResponse> {
        let model = request.model.unwrap_or_else(|| "gpt-4o-mini".to_string());
        
        let messages: Vec<OpenAIMessage> = request
            .messages
            .into_iter()
            .map(|m| OpenAIMessage {
                role: match m.role {
                    crate::trait_::MessageRole::System => "system".to_string(),
                    crate::trait_::MessageRole::User => "user".to_string(),
                    crate::trait_::MessageRole::Assistant => "assistant".to_string(),
                },
                content: m.content,
            })
            .collect();

        let req = OpenAIRequest {
            model,
            messages,
            temperature: request.temperature,
            max_tokens: request.max_tokens,
            stream: false,
        };

        let response = self
            .client
            .post(format!("{}/chat/completions", self.base_url))
            .header("Authorization", format!("Bearer {}", self.api_key))
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

        let openai_resp: OpenAIResponse = response
            .json()
            .await
            .map_err(|e| Error::Provider(format!("Failed to parse response: {}", e)))?;

        let content = openai_resp
            .choices
            .first()
            .ok_or_else(|| Error::Provider("No choices in response".to_string()))?
            .message
            .content
            .clone();

        let usage = openai_resp.usage.map(|u| crate::trait_::Usage {
            prompt_tokens: u.prompt_tokens,
            completion_tokens: u.completion_tokens,
            total_tokens: u.total_tokens,
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
