//! Deep Agent turn for TUI/CLI: run one conversational turn with tools via langchain DeepAgent.
//! Used when agent is build or plan and tools are available.

use async_trait::async_trait;
use langchain_ai_rust::agent::middleware::summarization::SummarizationMiddleware;
use langchain_ai_rust::agent::{create_deep_agent_from_llm, DeepAgentConfig};
use langchain_ai_rust::language_models::llm::LLM;
use langchain_ai_rust::language_models::{GenerateResult, LLMError};
use langchain_ai_rust::schemas::messages::Message;
use opencode_core::error::{Error, Result};
use opencode_core::tool::{Tool, ToolContext};
use std::path::PathBuf;
use std::sync::Arc;

/// Wraps Arc<dyn LLM> so it can be passed to create_deep_agent_from_llm (which expects Into<Box<dyn LLM>>).
struct LlmArcWrapper(Arc<dyn LLM>);

#[async_trait]
impl LLM for LlmArcWrapper {
    async fn generate(
        &self,
        messages: &[Message],
    ) -> std::result::Result<GenerateResult, LLMError> {
        self.0.generate(messages).await
    }

    async fn stream(
        &self,
        messages: &[Message],
    ) -> std::result::Result<
        std::pin::Pin<
            Box<
                dyn futures::Stream<Item = std::result::Result<langchain_ai_rust::schemas::StreamData, LLMError>>
                    + Send,
            >,
        >,
        LLMError,
    > {
        self.0.stream(messages).await
    }
}

impl langchain_ai_rust::language_models::llm::LLMClone for LlmArcWrapper {
    fn clone_box(&self) -> Box<dyn LLM> {
        Box::new(LlmArcWrapper(Arc::clone(&self.0)))
    }
}

/// Builds system prompt for a deep agent turn (coding assistant + optional workspace).
fn build_turn_system_prompt(workspace_path: Option<&str>, read_only: bool) -> String {
    let mut s = if read_only {
        "You are a read-only coding assistant in OpenCode. Analyze and explore the codebase; do not edit files or run destructive commands. When the user asks to analyze the project or code, use the available tools to read files and search the codebase."
            .to_string()
    } else {
        "You are a coding assistant in OpenCode. Help the user with code, project analysis, and development tasks. When they ask to analyze the project or code, use the available tools to read files, search the codebase, and make edits as needed."
            .to_string()
    };
    if let Some(path) = workspace_path {
        s.push_str("\n\nThe user is working in the following project directory: ");
        s.push_str(path);
        s.push_str(". Resolve relative paths relative to this directory when using tools.");
    }
    s
}

/// Converts session messages (opencode_core format) to langchain Message list.
/// Skips Tool role or encodes as human message so history is preserved.
fn session_messages_to_langchain(messages: &[opencode_core::Message]) -> Vec<Message> {
    messages
        .iter()
        .filter_map(|m| {
            use opencode_core::Role;
            match m.role {
                Role::System => Some(Message::new_system_message(m.content.clone())),
                Role::User => Some(Message::new_human_message(m.content.clone())),
                Role::Assistant => Some(Message::new_ai_message(m.content.clone())),
                Role::Tool => {
                    // Preserve tool result in history as a human-side note so model sees context
                    let content = format!("[Tool result: {}]", m.content);
                    Some(Message::new_human_message(content))
                }
            }
        })
        .collect()
}

/// Event emitted when a tool is invoked during a deep agent turn (for TUI log panel).
pub struct ToolCallEvent {
    pub tool_id: String,
    /// Serialized input (truncated for display).
    pub input_preview: String,
    pub output_len: Option<usize>,
    pub error: Option<String>,
}

/// Callback invoked after each tool run (e.g. to send to session log panel).
pub type OnToolCall = Arc<dyn Fn(ToolCallEvent) + Send + Sync>;

/// Configuration for a single deep agent turn.
pub struct DeepAgentTurnConfig {
    /// Workspace root for crate FS tools and for resolving paths.
    pub workspace_path: Option<PathBuf>,
    /// If true, only read-only tools are used and crate filesystem (write) tools are disabled.
    pub read_only: bool,
    /// If true, enable the crate's built-in filesystem tools (ls, read_file, write_file, etc.).
    /// When false, only the provided opencode tools are used.
    pub use_crate_filesystem: bool,
    /// When set, called after each tool run with event details (for TUI log).
    pub on_tool_call: Option<OnToolCall>,
    /// If set, only the last N messages are sent to the agent (keeps first System if outside window).
    pub max_history_messages: Option<usize>,
    /// If set, truncate each message content to this many characters (UTF-8 safe), appending " ... (truncated)".
    pub max_message_content_len: Option<usize>,
}

impl Default for DeepAgentTurnConfig {
    fn default() -> Self {
        Self {
            workspace_path: None,
            read_only: false,
            use_crate_filesystem: true,
            on_tool_call: None,
            max_history_messages: None,
            max_message_content_len: None,
        }
    }
}

/// Compress session history for context window: optional sliding window and per-message truncation.
fn compress_session_messages(
    messages: &[opencode_core::Message],
    max_history: Option<usize>,
    max_content_len: Option<usize>,
) -> Vec<opencode_core::Message> {
    use opencode_core::Role;

    let mut out: Vec<opencode_core::Message> = if let Some(n) = max_history {
        let len = messages.len();
        let start = len.saturating_sub(n);
        let first_system_idx = messages.iter().position(|m| m.role == Role::System);
        if let Some(sys_idx) = first_system_idx {
            if sys_idx < start {
                let mut v = vec![messages[sys_idx].clone()];
                v.extend_from_slice(&messages[start..]);
                v
            } else {
                messages[start..].to_vec()
            }
        } else {
            messages[start..].to_vec()
        }
    } else {
        messages.to_vec()
    };

    if let Some(max_len) = max_content_len {
        const TRUNCATED_SUFFIX: &str = " ... (truncated)";
        out = out
            .into_iter()
            .map(|m| {
                let content = if m.content.chars().count() > max_len {
                    let truncated: String = m.content.chars().take(max_len).collect();
                    format!("{}{}", truncated, TRUNCATED_SUFFIX)
                } else {
                    m.content
                };
                opencode_core::Message {
                    content,
                    ..m
                }
            })
            .collect();
    }

    out
}

/// Runs one deep agent turn: builds agent with the given LLM and tools, converts message history
/// + new user input to langchain messages, invokes the agent, and returns the final reply string.
/// No streaming; the caller should send the reply to the UI in one shot.
pub async fn run_deep_agent_turn(
    llm: &Arc<dyn langchain_ai_rust::language_models::llm::LLM>,
    session_messages: &[opencode_core::Message],
    user_input: &str,
    tools: &[Arc<dyn Tool>],
    tool_ctx: &ToolContext,
    config: DeepAgentTurnConfig,
) -> Result<String> {
    let on_tool_call = config.on_tool_call.clone();
    let langchain_tools: Vec<Arc<dyn langchain_ai_rust::tools::Tool>> = tools
        .iter()
        .cloned()
        .map(|t| {
            let ctx = tool_ctx.clone();
            let cb = on_tool_call.clone();
            Arc::new(crate::langchain_tool_adapter::LangChainToolAdapter::new_with_context_and_callback(
                t, ctx, cb,
            )) as Arc<dyn langchain_ai_rust::tools::Tool>
        })
        .collect();
    let tool_names: Vec<String> = tools.iter().map(|t| t.id().to_string()).collect();
    tracing::info!(
        tool_count = langchain_tools.len(),
        tool_ids = ?tool_names,
        use_crate_filesystem = config.use_crate_filesystem,
        "deep_agent tools registered"
    );

    let workspace_root = config.workspace_path.clone();
    let mut agent_config = DeepAgentConfig::new()
        .with_planning(true)
        .with_filesystem(config.use_crate_filesystem);
    if let Some(ref root) = workspace_root {
        agent_config = agent_config.with_workspace_root(root.clone());
    }
    if config.read_only {
        agent_config = agent_config.with_filesystem(false);
    }
    let summarization = SummarizationMiddleware::new()
        .with_message_threshold(50)
        .with_token_threshold(4000)
        .with_preserve_recent(10)
        .with_summarizer(llm.clone());
    agent_config = agent_config.with_middleware(vec![Arc::new(summarization)]);

    let system_prompt = build_turn_system_prompt(
        tool_ctx.workspace_path.as_deref(),
        config.read_only,
    );

    let agent = create_deep_agent_from_llm(
        LlmArcWrapper(llm.clone()),
        &langchain_tools,
        Some(&system_prompt),
        agent_config,
    )
    .map_err(|e| Error::Provider(format!("DeepAgent create failed: {}", e)))?;

    let compressed = compress_session_messages(
        session_messages,
        config.max_history_messages,
        config.max_message_content_len,
    );
    let mut messages = session_messages_to_langchain(&compressed);
    messages.push(Message::new_human_message(user_input));

    let reply = agent
        .invoke_messages(messages)
        .await
        .map_err(|e| Error::Provider(format!("DeepAgent invoke failed: {}", e)))?;

    Ok(reply)
}
