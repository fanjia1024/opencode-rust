use crate::error::Result;
use crate::session::Session;
use crate::tool::Tool;
use async_trait::async_trait;
use futures::Stream;
use futures::StreamExt;
use std::pin::Pin;
use std::sync::Arc;
use tokio::sync::mpsc::UnboundedSender;

// Provider trait is defined in opencode-provider, but we use a trait object here
// to avoid circular dependency. The actual Provider implementation will be passed
// from the caller.
#[async_trait::async_trait]
pub trait Provider: Send + Sync {
    async fn generate(&self, request: ProviderRequest) -> Result<ProviderResponse>;
    /// Stream LLM response chunks. Returns Err if streaming is not supported.
    async fn stream(
        &self,
        request: ProviderRequest,
    ) -> Result<Pin<Box<dyn Stream<Item = Result<ProviderChunk>> + Send + Unpin>>>;
    fn models(&self) -> &[ModelInfo];
}

/// A chunk of streamed LLM output.
#[derive(Debug, Clone)]
pub struct ProviderChunk {
    pub content: String,
    pub done: bool,
}

#[derive(Debug, Clone)]
pub struct ProviderRequest {
    pub messages: Vec<Message>,
    pub model: Option<String>,
    pub temperature: Option<f32>,
    pub max_tokens: Option<u32>,
}

#[derive(Debug, Clone)]
pub struct ProviderResponse {
    pub content: String,
    pub usage: Option<serde_json::Value>,
}

#[derive(Debug, Clone)]
pub struct ModelInfo {
    pub id: String,
    pub name: String,
}

#[derive(Debug, Clone)]
pub struct Message {
    pub role: MessageRole,
    pub content: String,
}

#[derive(Debug, Clone)]
pub enum MessageRole {
    System,
    User,
    Assistant,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AgentMode {
    Build,
    Plan,
    General,
}

pub struct Context {
    pub session_id: String,
    pub message_id: String,
    pub agent: String,
    /// Current project/workspace directory for prompts and future tool execution.
    pub workspace_path: Option<String>,
}

/// Builds the system message content for BuildAgent: coding assistant role and project context.
fn build_system_message(ctx: &Context) -> String {
    let mut s = "You are a coding assistant in OpenCode. Help the user with code, project analysis, and development tasks.".to_string();
    if let Some(ref path) = ctx.workspace_path {
        s.push_str("\n\nThe user is working in the following project directory: ");
        s.push_str(path);
        s.push_str(". When they ask to analyze the project or code, reason about the project context. If you have access to tools, use them to read files and search the codebase as needed.");
    }
    s
}

#[async_trait]
pub trait Agent: Send + Sync {
    async fn process(
        &self,
        ctx: &Context,
        input: &str,
        session: &mut Session,
        provider: &dyn Provider,
        tools: &[Arc<dyn Tool>],
    ) -> Result<()>;
    /// Process with streaming: send chunks via stream_tx (Some(content) per chunk, None when done).
    /// Returns Err if the provider does not support streaming; caller may fall back to process().
    async fn process_stream(
        &self,
        ctx: &Context,
        input: &str,
        session: &mut Session,
        provider: &dyn Provider,
        tools: &[Arc<dyn Tool>],
        stream_tx: UnboundedSender<(String, Option<String>)>,
    ) -> Result<()>;
    fn name(&self) -> &str;
    fn mode(&self) -> AgentMode;
}

pub struct BuildAgent;

impl BuildAgent {
    pub fn new() -> Self {
        Self
    }
}

#[async_trait]
impl Agent for BuildAgent {
    async fn process(
        &self,
        ctx: &Context,
        input: &str,
        session: &mut Session,
        provider: &dyn Provider,
        _tools: &[Arc<dyn Tool>],
    ) -> Result<()> {
        use crate::session::{Message, Role};
        use chrono::Utc;

        let user_message = Message {
            role: Role::User,
            content: input.to_string(),
            created_at: Utc::now(),
            meta: None,
        };
        session.push_message(user_message);

        let mut messages: Vec<crate::agent::Message> = session
            .messages
            .iter()
            .map(|m| crate::agent::Message {
                role: match m.role {
                    Role::User => crate::agent::MessageRole::User,
                    Role::Assistant => crate::agent::MessageRole::Assistant,
                    Role::System => crate::agent::MessageRole::System,
                    Role::Tool => crate::agent::MessageRole::System,
                },
                content: m.content.clone(),
            })
            .collect();

        if messages.first().map(|m| !matches!(m.role, MessageRole::System)).unwrap_or(true) {
            let system_content = build_system_message(ctx);
            messages.insert(
                0,
                crate::agent::Message {
                    role: MessageRole::System,
                    content: system_content,
                },
            );
        }

        let message_count = messages.len();
        let total_prompt_chars: usize = messages.iter().map(|m| m.content.len()).sum();
        let last_user_msg_trunc: Option<String> = messages
            .iter()
            .rev()
            .find(|m| matches!(m.role, MessageRole::User))
            .map(|m| if m.content.len() > 200 { m.content[..200].to_string() } else { m.content.clone() });
        let request = ProviderRequest {
            messages,
            model: None,
            temperature: Some(0.7),
            max_tokens: Some(4096),
        };

        tracing::info!(
            session_id = %ctx.session_id,
            agent = %ctx.agent,
            message_count,
            total_prompt_chars,
            model = ?request.model,
            temperature = ?request.temperature,
            max_tokens = ?request.max_tokens,
            "LLM request start"
        );
        if let Some(ref trunc) = last_user_msg_trunc {
            tracing::debug!(last_user_msg_trunc = %trunc, "LLM request detail");
        }

        let response = match provider.generate(request).await {
            Ok(r) => {
                tracing::info!(
                    response_len = r.content.len(),
                    usage = ?r.usage,
                    "LLM response received"
                );
                let trunc = if r.content.len() > 500 {
                    r.content.get(..500).unwrap_or(&r.content)
                } else {
                    r.content.as_str()
                };
                tracing::debug!(response_trunc = %trunc, "LLM response detail");
                r
            }
            Err(e) => {
                tracing::error!(error = %e, "provider.generate failed");
                return Err(e);
            }
        };

        let assistant_message = Message {
            role: Role::Assistant,
            content: response.content,
            created_at: Utc::now(),
            meta: None,
        };
        session.push_message(assistant_message);

        Ok(())
    }

    async fn process_stream(
        &self,
        ctx: &Context,
        input: &str,
        session: &mut Session,
        provider: &dyn Provider,
        _tools: &[Arc<dyn Tool>],
        stream_tx: UnboundedSender<(String, Option<String>)>,
    ) -> Result<()> {
        use crate::session::{Message, Role};
        use chrono::Utc;

        // Build request including the new user message but do not push to session yet
        // so that on stream Err the caller can fall back to process() without double user message.
        let mut messages: Vec<crate::agent::Message> = session
            .messages
            .iter()
            .map(|m| crate::agent::Message {
                role: match m.role {
                    Role::User => crate::agent::MessageRole::User,
                    Role::Assistant => crate::agent::MessageRole::Assistant,
                    Role::System => crate::agent::MessageRole::System,
                    Role::Tool => crate::agent::MessageRole::System,
                },
                content: m.content.clone(),
            })
            .collect();

        if messages.first().map(|m| !matches!(m.role, MessageRole::System)).unwrap_or(true) {
            let system_content = build_system_message(ctx);
            messages.insert(
                0,
                crate::agent::Message {
                    role: MessageRole::System,
                    content: system_content,
                },
            );
        }

        messages.push(crate::agent::Message {
            role: MessageRole::User,
            content: input.to_string(),
        });

        let request = ProviderRequest {
            messages,
            model: None,
            temperature: Some(0.7),
            max_tokens: Some(4096),
        };

        let mut stream = match provider.stream(request).await {
            Ok(s) => s,
            Err(e) => {
                tracing::debug!(error = %e, "provider.stream not supported, use process()");
                return Err(e);
            }
        };

        let user_message = Message {
            role: Role::User,
            content: input.to_string(),
            created_at: Utc::now(),
            meta: None,
        };
        session.push_message(user_message);

        let mut buffer = String::new();
        while let Some(item) = stream.next().await {
            let chunk = item?;
            let _ = stream_tx.send((ctx.session_id.clone(), Some(chunk.content.clone())));
            buffer.push_str(&chunk.content);
            if chunk.done {
                break;
            }
        }
        let _ = stream_tx.send((ctx.session_id.clone(), None));

        let assistant_message = Message {
            role: Role::Assistant,
            content: buffer,
            created_at: Utc::now(),
            meta: None,
        };
        session.push_message(assistant_message);

        Ok(())
    }

    fn name(&self) -> &str {
        "build"
    }

    fn mode(&self) -> AgentMode {
        AgentMode::Build
    }
}

/// Plan agent: currently delegates to BuildAgent (same processing path).
/// If the product goal is read-only analysis, tools can be filtered in the CLI
/// (e.g. in process_message_async when agent_name == "plan") to only pass read-only tools.
pub struct PlanAgent;

impl PlanAgent {
    pub fn new() -> Self {
        Self
    }
}

#[async_trait]
impl Agent for PlanAgent {
    async fn process(
        &self,
        ctx: &Context,
        input: &str,
        session: &mut Session,
        provider: &dyn Provider,
        tools: &[Arc<dyn Tool>],
    ) -> Result<()> {
        BuildAgent::new()
            .process(ctx, input, session, provider, tools)
            .await
    }

    async fn process_stream(
        &self,
        ctx: &Context,
        input: &str,
        session: &mut Session,
        provider: &dyn Provider,
        tools: &[Arc<dyn Tool>],
        stream_tx: UnboundedSender<(String, Option<String>)>,
    ) -> Result<()> {
        BuildAgent::new()
            .process_stream(ctx, input, session, provider, tools, stream_tx)
            .await
    }

    fn name(&self) -> &str {
        "plan"
    }

    fn mode(&self) -> AgentMode {
        AgentMode::Plan
    }
}

pub struct GeneralAgent;

impl GeneralAgent {
    pub fn new() -> Self {
        Self
    }
}

#[async_trait]
impl Agent for GeneralAgent {
    async fn process(
        &self,
        ctx: &Context,
        input: &str,
        session: &mut Session,
        provider: &dyn Provider,
        tools: &[Arc<dyn Tool>],
    ) -> Result<()> {
        BuildAgent::new()
            .process(ctx, input, session, provider, tools)
            .await
    }

    async fn process_stream(
        &self,
        ctx: &Context,
        input: &str,
        session: &mut Session,
        provider: &dyn Provider,
        tools: &[Arc<dyn Tool>],
        stream_tx: UnboundedSender<(String, Option<String>)>,
    ) -> Result<()> {
        BuildAgent::new()
            .process_stream(ctx, input, session, provider, tools, stream_tx)
            .await
    }

    fn name(&self) -> &str {
        "general"
    }

    fn mode(&self) -> AgentMode {
        AgentMode::General
    }
}
