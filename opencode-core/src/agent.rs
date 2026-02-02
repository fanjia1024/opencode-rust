use crate::error::Result;
use crate::session::Session;
use crate::tool::Tool;
use async_trait::async_trait;
use std::sync::Arc;

// Provider trait is defined in opencode-provider, but we use a trait object here
// to avoid circular dependency. The actual Provider implementation will be passed
// from the caller.
#[async_trait::async_trait]
pub trait Provider: Send + Sync {
    async fn generate(&self, request: ProviderRequest) -> Result<ProviderResponse>;
    fn models(&self) -> &[ModelInfo];
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
        _ctx: &Context,
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

        let messages: Vec<crate::agent::Message> = session
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

        let request = ProviderRequest {
            messages,
            model: None,
            temperature: Some(0.7),
            max_tokens: Some(4096),
        };

        let response = provider.generate(request).await?;

        let assistant_message = Message {
            role: Role::Assistant,
            content: response.content,
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
        BuildAgent::new().process(ctx, input, session, provider, tools).await
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
        BuildAgent::new().process(ctx, input, session, provider, tools).await
    }

    fn name(&self) -> &str {
        "general"
    }

    fn mode(&self) -> AgentMode {
        AgentMode::General
    }
}
