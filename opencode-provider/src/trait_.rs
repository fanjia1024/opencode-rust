use async_trait::async_trait;
use futures::Stream;
use opencode_core::error::Result;
use serde_json::Value;

pub struct GenerateRequest {
    pub messages: Vec<Message>,
    pub model: Option<String>,
    pub temperature: Option<f64>,
    pub max_tokens: Option<u32>,
}

pub struct GenerateResponse {
    pub content: String,
    pub usage: Option<Usage>,
}

pub struct Chunk {
    pub content: String,
    pub done: bool,
}

#[derive(Debug)]
pub struct Usage {
    pub prompt_tokens: u32,
    pub completion_tokens: u32,
    pub total_tokens: u32,
}

#[derive(Debug, Clone)]
pub enum MessageRole {
    System,
    User,
    Assistant,
}

#[derive(Debug, Clone)]
pub struct Message {
    pub role: MessageRole,
    pub content: String,
}

#[async_trait]
pub trait Provider: Send + Sync {
    async fn generate(&self, request: GenerateRequest) -> Result<GenerateResponse>;

    async fn stream(
        &self,
        request: GenerateRequest,
    ) -> Result<Box<dyn Stream<Item = Result<Chunk>> + Send + Unpin>>;

    fn models(&self) -> &[ModelInfo];

    /// Returns the underlying LLM when this provider is backed by one (e.g. LangChainAdapter).
    /// Used to run deep agent turns with tools. Returns None for providers that do not expose an LLM.
    fn as_llm(&self) -> Option<std::sync::Arc<dyn langchain_ai_rust::language_models::llm::LLM>> {
        None
    }
}

#[derive(Debug, Clone)]
pub struct ModelInfo {
    pub id: String,
    pub name: String,
    pub provider: String,
}
