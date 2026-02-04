use crate::trait_::{Chunk, GenerateRequest, GenerateResponse, Provider};
use async_trait::async_trait;
use futures::Stream;
use futures::StreamExt;
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

    /// Returns the underlying LLM for use with deep agent (e.g. run_deep_agent_turn).
    pub fn llm(&self) -> Arc<dyn langchain_ai_rust::language_models::llm::LLM> {
        Arc::clone(&self.llm)
    }
}

/// Strips `think>...</think>` and `<think>...</think>` blocks from model output so only the
/// visible reply is shown (e.g. for One API / deep-thinking / MiniMax-style models).
/// Unclosed blocks are dropped up to the next `\n\n` or end of string.
pub(crate) fn strip_thinking_blocks(s: &str) -> String {
    const THINK_OPEN: &str = "think>";
    const THINK_XML_OPEN: &str = "<think>";
    const CLOSE: &str = "</think>";
    let mut out = String::new();
    let mut rest = s;
    loop {
        let (open_pos, open_len) = match (rest.find(THINK_OPEN), rest.find(THINK_XML_OPEN)) {
            (Some(p), None) => (p, THINK_OPEN.len()),
            (None, Some(p)) => (p, THINK_XML_OPEN.len()),
            (Some(t), Some(x)) => {
                if t <= x {
                    (t, THINK_OPEN.len())
                } else {
                    (x, THINK_XML_OPEN.len())
                }
            }
            (None, None) => {
                out.push_str(rest);
                break;
            }
        };
        out.push_str(&rest[..open_pos]);
        let after_open = open_pos + open_len;
        if let Some(close_pos) = rest[after_open..].find(CLOSE) {
            rest = &rest[after_open + close_pos + CLOSE.len()..];
        } else {
            let remainder = &rest[after_open..];
            if let Some(dbl) = remainder.find("\n\n") {
                out.push_str(&remainder[dbl + 2..]);
            }
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
        tracing::info!(prompt_len = prompt.len(), "LangChainAdapter: llm.invoke start");
        let prompt_trunc = if prompt.len() > 200 { &prompt[..200] } else { prompt.as_str() };
        tracing::debug!(prompt_trunc = %prompt_trunc, "LangChainAdapter: request detail");
        let response = match self.llm.invoke(&prompt).await {
            Ok(r) => {
                tracing::info!(response_len = r.len(), "LangChainAdapter: llm.invoke succeeded");
                r
            }
            Err(e) => {
                tracing::error!(error = %e, "llm.invoke failed");
                return Err(Error::Provider(format!("LLM invocation failed: {}", e)));
            }
        };

        let response_trunc = if response.len() > 500 { &response[..500] } else { response.as_str() };
        tracing::debug!(response_trunc = %response_trunc, "LangChainAdapter: response detail");
        let content = strip_thinking_blocks(&response);
        Ok(GenerateResponse {
            content,
            usage: None,
        })
    }

    async fn stream(
        &self,
        request: GenerateRequest,
    ) -> Result<Box<dyn Stream<Item = Result<Chunk>> + Send + Unpin>> {
        use langchain_ai_rust::schemas::Message;

        let messages: Vec<Message> = request
            .messages
            .into_iter()
            .map(|m| match m.role {
                crate::trait_::MessageRole::System => Message::new_system_message(m.content),
                crate::trait_::MessageRole::User => Message::new_human_message(m.content),
                crate::trait_::MessageRole::Assistant => Message::new_ai_message(m.content),
            })
            .collect();

        let stream = self
            .llm
            .stream(&messages)
            .await
            .map_err(|e| Error::Provider(format!("LLM stream failed: {}", e)))?;

        let mapped = stream.map(|r| {
            r.map(|stream_data| Chunk {
                content: stream_data.content,
                done: false,
            })
            .map_err(|e| Error::Provider(format!("Stream error: {}", e)))
        });
        Ok(Box::new(mapped) as Box<dyn Stream<Item = Result<Chunk>> + Send + Unpin>)
    }

    fn models(&self) -> &[crate::trait_::ModelInfo] {
        &[]
    }

    fn as_llm(&self) -> Option<Arc<dyn langchain_ai_rust::language_models::llm::LLM>> {
        Some(self.llm())
    }
}
