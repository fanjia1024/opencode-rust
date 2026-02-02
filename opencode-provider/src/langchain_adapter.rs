#[cfg(feature = "langchain")]
use crate::trait_::{GenerateRequest, GenerateResponse, Provider};
use async_trait::async_trait;
use futures::Stream;
use opencode_core::error::{Error, Result};
use std::sync::Arc;

#[cfg(feature = "langchain")]
pub struct LangChainAdapter {
    llm: Arc<dyn langchain_ai_rust::language_models::llm::LLM>,
}

#[cfg(feature = "langchain")]
impl LangChainAdapter {
    pub fn new(llm: Arc<dyn langchain_ai_rust::language_models::llm::LLM>) -> Self {
        Self { llm }
    }

    pub fn from_openai(api_key: String) -> Result<Self> {
        use langchain_ai_rust::llm::openai::{OpenAI, OpenAIConfig, OpenAIModel};
        
        let config = OpenAIConfig::default()
            .with_api_key(api_key);
        
        let openai = OpenAI::new(config)
            .with_model(OpenAIModel::Gpt4oMini);
        
        Ok(Self {
            llm: Arc::new(openai),
        })
    }

    pub fn from_anthropic(_api_key: String) -> Result<Self> {
        // Anthropic support may not be available in this version of langchain-rust
        // Return an error for now
        Err(Error::Provider("Anthropic support not available in langchain-rust".to_string()))
    }
}

#[cfg(feature = "langchain")]
#[async_trait]
impl Provider for LangChainAdapter {
    async fn generate(&self, request: GenerateRequest) -> Result<GenerateResponse> {
        let prompt = request
            .messages
            .iter()
            .map(|m| {
                match m.role {
                    crate::trait_::MessageRole::System => format!("System: {}\n", m.content),
                    crate::trait_::MessageRole::User => format!("User: {}\n", m.content),
                    crate::trait_::MessageRole::Assistant => format!("Assistant: {}\n", m.content),
                }
            })
            .collect::<Vec<_>>()
            .join("");
        
        let response = self.llm
            .invoke(&prompt)
            .await
            .map_err(|e| Error::Provider(format!("LLM invocation failed: {}", e)))?;

        // langchain-rust's invoke returns a String directly
        Ok(GenerateResponse {
            content: response,
            usage: None,
        })
    }

    async fn stream(
        &self,
        _request: GenerateRequest,
    ) -> Result<Box<dyn Stream<Item = Result<crate::trait_::Chunk>> + Send + Unpin>> {
        Err(Error::Provider("Streaming not yet implemented with langchain-ai-rust".to_string()))
    }

    fn models(&self) -> &[crate::trait_::ModelInfo] {
        &[]
    }
}

#[cfg(not(feature = "langchain"))]
pub struct LangChainAdapter;

#[cfg(not(feature = "langchain"))]
impl LangChainAdapter {
    pub fn new(_llm: ()) -> Self {
        Self
    }

    pub fn from_openai(_api_key: String) -> Result<Self> {
        Err(Error::Provider("langchain-rust feature not enabled".to_string()))
    }

    pub fn from_anthropic(_api_key: String) -> Result<Self> {
        Err(Error::Provider("langchain-rust feature not enabled".to_string()))
    }
}
