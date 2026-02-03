use crate::trait_::{GenerateRequest, GenerateResponse, Provider};
use async_trait::async_trait;
use futures::Stream;
use opencode_core::error::{Error, Result};
use std::sync::Arc;

pub struct LangChainAdapter {
    llm: Arc<dyn langchain_ai_rust::language_models::llm::LLM>,
}

impl LangChainAdapter {
    pub fn new(llm: Arc<dyn langchain_ai_rust::language_models::llm::LLM>) -> Self {
        Self { llm }
    }

    pub fn from_openai(
        api_key: String,
        base_url: Option<String>,
        model: Option<String>,
    ) -> Result<Self> {
        use langchain_ai_rust::llm::openai::{OpenAI, OpenAIConfig};

        let config = OpenAIConfig::default().with_api_key(api_key);
        let config = if let Some(url) = base_url {
            config.with_api_base(url)
        } else {
            config
        };
        let model_name = model.unwrap_or_else(|| "gpt-4o-mini".to_string());
        let openai = OpenAI::new(config).with_model(model_name);

        Ok(Self {
            llm: Arc::new(openai),
        })
    }

    pub fn from_ollama(base_url: Option<String>, model: Option<String>) -> Result<Self> {
        use langchain_ai_rust::llm::ollama::openai::OllamaConfig;
        use langchain_ai_rust::llm::openai::OpenAI;

        let api_base = base_url.unwrap_or_else(|| "http://localhost:11434/v1".to_string());
        let config = OllamaConfig::default().with_api_base(api_base);
        let model_name = model.unwrap_or_else(|| "llama3.2".to_string());
        let openai = OpenAI::new(config).with_model(model_name);

        Ok(Self {
            llm: Arc::new(openai),
        })
    }

    pub fn from_qwen(
        api_key: String,
        base_url: Option<String>,
        model: Option<String>,
    ) -> Result<Self> {
        use langchain_ai_rust::llm::qwen::Qwen;

        let qwen = Qwen::new().with_api_key(api_key);
        let qwen = if let Some(url) = base_url {
            qwen.with_base_url(url)
        } else {
            qwen
        };
        let model_name = model.unwrap_or_else(|| "qwen-turbo".to_string());
        let qwen = qwen.with_model(model_name);

        Ok(Self {
            llm: Arc::new(qwen),
        })
    }

    pub fn from_anthropic(_api_key: String) -> Result<Self> {
        Err(Error::Provider(
            "Anthropic support not available in langchain-ai-rust".to_string(),
        ))
    }
}

/// Strips `think>...</think>` blocks and unclosed `think>...` from model output
/// so only the visible reply is shown (e.g. for One API / deep-thinking models).
fn strip_think_blocks(s: &str) -> String {
    const THINK_OPEN: &str = "think>";
    const THINK_CLOSE: &str = "</think>";
    let mut out = String::new();
    let mut rest = s;
    loop {
        if let Some(open_pos) = rest.find(THINK_OPEN) {
            out.push_str(&rest[..open_pos]);
            let after_open = open_pos + THINK_OPEN.len();
            if let Some(close_pos) = rest[after_open..].find(THINK_CLOSE) {
                rest = &rest[after_open + close_pos + THINK_CLOSE.len()..];
            } else {
                let remainder = &rest[after_open..];
                if let Some(dbl) = remainder.find("\n\n") {
                    out.push_str(&remainder[dbl + 2..]);
                }
                break;
            }
        } else {
            out.push_str(rest);
            break;
        }
    }
    out.trim().to_string()
}

#[async_trait]
impl Provider for LangChainAdapter {
    async fn generate(&self, request: GenerateRequest) -> Result<GenerateResponse> {
        let prompt = request
            .messages
            .iter()
            .map(|m| match m.role {
                crate::trait_::MessageRole::System => format!("System: {}\n", m.content),
                crate::trait_::MessageRole::User => format!("User: {}\n", m.content),
                crate::trait_::MessageRole::Assistant => format!("Assistant: {}\n", m.content),
            })
            .collect::<Vec<_>>()
            .join("");
        tracing::debug!(prompt_len = prompt.len(), "LangChainAdapter::generate");
        tracing::debug!("calling llm.invoke");
        let response = match self.llm.invoke(&prompt).await {
            Ok(r) => {
                tracing::debug!(response_len = r.len(), "llm.invoke succeeded");
                r
            }
            Err(e) => {
                tracing::error!(error = %e, "llm.invoke failed");
                return Err(Error::Provider(format!("LLM invocation failed: {}", e)));
            }
        };

        let content = strip_think_blocks(&response);
        Ok(GenerateResponse {
            content,
            usage: None,
        })
    }

    async fn stream(
        &self,
        _request: GenerateRequest,
    ) -> Result<Box<dyn Stream<Item = Result<crate::trait_::Chunk>> + Send + Unpin>> {
        Err(Error::Provider(
            "Streaming not yet implemented with langchain-ai-rust".to_string(),
        ))
    }

    fn models(&self) -> &[crate::trait_::ModelInfo] {
        &[]
    }
}
