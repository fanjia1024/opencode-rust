//! Async message processing: loads session, runs agent (deep or streaming), sends reply chunks and log entries via channel.
//! Used by the Tauri app backend; channel type is UI-agnostic.

use crate::config::AppConfig;
use crate::session_store;
use anyhow::Result;
use chrono::Utc;
use opencode_core::agent::Context;
use opencode_core::ids::SessionId;
use opencode_core::session::{Message as SessionMessage, Role, Session};
use opencode_core::AgentManager;
use opencode_core::tool::ToolContext;
use std::path::PathBuf;
use std::str::FromStr;
use std::sync::Arc;
use tokio::sync::mpsc;

/// Session UI update: either a reply chunk/done or a log entry for the log panel.
#[derive(Clone)]
pub enum SessionUpdate {
    /// Some(chunk) = stream chunk or full response; None = stream done.
    Reply(Option<String>),
    /// Append to the session's log panel.
    Log(LogEntry),
}

/// One line in the session log panel (agent lifecycle, tool calls, etc.).
#[derive(Clone)]
pub struct LogEntry {
    pub level: LogLevel,
    pub message: String,
}

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum LogLevel {
    Info,
    Warn,
    Error,
}

/// Process one user message: load or create session, run agent (deep or streaming), send reply chunks and logs via `tx`.
/// Caller (e.g. Tauri backend) should forward `(session_id, SessionUpdate)` to the frontend (e.g. via Tauri events).
/// If `command_id` is set, user input is formatted via opencode_core::format_input_for_command before being sent to the agent.
pub async fn process_message_async(
    session_id: &str,
    input: &str,
    agent_name: &str,
    config: AppConfig,
    workspace_path: Option<PathBuf>,
    tx: mpsc::UnboundedSender<(String, SessionUpdate)>,
    command_id: Option<String>,
) -> Result<()> {
    let effective_input: String = match &command_id {
        Some(id) => opencode_core::format_input_for_command(
            id,
            input,
            workspace_path.as_deref(),
            config.core_config(),
        )
        .unwrap_or_else(|| input.to_string()),
        None => input.to_string(),
    };
    let effective_input = effective_input.as_str();

    let send_log = |level: LogLevel, message: String| {
        let _ = tx.send((
            session_id.to_string(),
            SessionUpdate::Log(LogEntry { level, message }),
        ));
    };

    let preview: String = effective_input.chars().take(50).collect();
    let preview = preview.trim();
    let suffix = if effective_input.chars().count() > 50 { "…" } else { "" };
    send_log(
        LogLevel::Info,
        format!("► Request: {}{}", preview, suffix),
    );
    send_log(
        LogLevel::Info,
        format!("process_message_async started input_len={}", effective_input.len()),
    );
    tracing::info!(session_id = %session_id, input_len = effective_input.len(), "process_message_async started");

    let session_dir = config.session_dir();
    let session_file = session_dir.join(session_id).join("session.json");
    let mut session = if session_file.exists() {
        session_store::load_session(&session_file).unwrap_or_else(|_| Session::new())
    } else {
        let s = Session::with_id(
            SessionId::from_str(session_id).unwrap_or_else(|_| SessionId::new()),
        );
        let path = session_dir.join(session_id).join("session.json");
        let _ = session_store::save_session(&path, &s);
        s
    };
    let session_id_owned = session_id.to_string();

    let provider_adapter = {
        let provider_info = config.get_default_provider();
        let provider_type = provider_info
            .as_ref()
            .map(|p| p.provider_type.clone())
            .unwrap_or_else(|| "openai".to_string());
        tracing::info!(provider_type = %provider_type, "provider selected");
        let base_url = provider_info.as_ref().and_then(|p| p.base_url.clone());
        let model = provider_info.as_ref().and_then(|p| p.model.clone());

        let api_key = provider_info
            .as_ref()
            .and_then(|p| p.api_key.clone())
            .or_else(|| std::env::var("OPENAI_API_KEY").ok())
            .or_else(|| std::env::var("OPENCODE_OPENAI_API_KEY").ok())
            .unwrap_or_else(|| "".to_string());

        let provider: Arc<dyn opencode_provider::Provider> = match provider_type.as_str() {
            "openai" => {
                if api_key.trim().is_empty() {
                    tracing::error!("No API key configured for OpenAI");
                    let _ = tx.send((
                        session_id_owned.clone(),
                        SessionUpdate::Reply(Some(
                            "Error: No API key configured. Configure provider and API key.".to_string(),
                        )),
                    ));
                    let _ = tx.send((session_id_owned.clone(), SessionUpdate::Reply(None)));
                    return Err(anyhow::anyhow!("No API key configured"));
                }
                match opencode_provider::LangChainAdapter::from_openai(api_key, base_url, model) {
                    Ok(adapter) => Arc::new(adapter),
                    Err(e) => {
                        tracing::error!(error = %e, "Failed to initialize OpenAI provider");
                        let _ = tx.send((
                            session_id_owned.clone(),
                            SessionUpdate::Reply(Some(format!(
                                "Error initializing OpenAI provider: {}",
                                e
                            ))),
                        ));
                        let _ = tx.send((session_id_owned.clone(), SessionUpdate::Reply(None)));
                        return Err(anyhow::anyhow!("Failed to initialize provider: {}", e));
                    }
                }
            }
            "ollama" => {
                match opencode_provider::LangChainAdapter::from_ollama(base_url, model) {
                    Ok(adapter) => Arc::new(adapter),
                    Err(e) => {
                        tracing::error!(error = %e, "Failed to initialize Ollama provider");
                        let _ = tx.send((
                            session_id_owned.clone(),
                            SessionUpdate::Reply(Some(format!(
                                "Error initializing Ollama provider: {}",
                                e
                            ))),
                        ));
                        let _ = tx.send((session_id_owned.clone(), SessionUpdate::Reply(None)));
                        return Err(anyhow::anyhow!("Failed to initialize provider: {}", e));
                    }
                }
            }
            "qwen" => {
                if api_key.trim().is_empty() {
                    tracing::error!("No API key configured for Qwen");
                    let _ = tx.send((
                        session_id_owned.clone(),
                        SessionUpdate::Reply(Some(
                            "Error: No API key configured for Qwen.".to_string(),
                        )),
                    ));
                    let _ = tx.send((session_id_owned.clone(), SessionUpdate::Reply(None)));
                    return Err(anyhow::anyhow!("No API key configured"));
                }
                match opencode_provider::LangChainAdapter::from_qwen(api_key, base_url, model) {
                    Ok(adapter) => Arc::new(adapter),
                    Err(e) => {
                        tracing::error!(error = %e, "Failed to initialize Qwen provider");
                        let _ = tx.send((
                            session_id_owned.clone(),
                            SessionUpdate::Reply(Some(format!(
                                "Error initializing Qwen provider: {}",
                                e
                            ))),
                        ));
                        let _ = tx.send((session_id_owned.clone(), SessionUpdate::Reply(None)));
                        return Err(anyhow::anyhow!("Failed to initialize provider: {}", e));
                    }
                }
            }
            "anthropic" => {
                if api_key.trim().is_empty() {
                    tracing::error!("No API key configured for Anthropic");
                    let _ = tx.send((
                        session_id_owned.clone(),
                        SessionUpdate::Reply(Some(
                            "Error: No API key configured.".to_string(),
                        )),
                    ));
                    let _ = tx.send((session_id_owned.clone(), SessionUpdate::Reply(None)));
                    return Err(anyhow::anyhow!("No API key configured"));
                }
                match opencode_provider::LangChainAdapter::from_anthropic(api_key) {
                    Ok(adapter) => Arc::new(adapter),
                    Err(e) => {
                        tracing::error!(error = %e, "Failed to initialize Anthropic provider");
                        let _ = tx.send((
                            session_id_owned.clone(),
                            SessionUpdate::Reply(Some(format!(
                                "Error initializing Anthropic provider: {}",
                                e
                            ))),
                        ));
                        let _ = tx.send((session_id_owned.clone(), SessionUpdate::Reply(None)));
                        return Err(anyhow::anyhow!("Failed to initialize provider: {}", e));
                    }
                }
            }
            _ => {
                tracing::error!(provider_type = %provider_type, "Unsupported provider type");
                let _ = tx.send((
                    session_id_owned.clone(),
                    SessionUpdate::Reply(Some(format!(
                        "Unsupported provider type: {}",
                        provider_type
                    ))),
                ));
                let _ = tx.send((session_id_owned.clone(), SessionUpdate::Reply(None)));
                return Err(anyhow::anyhow!(
                    "Unsupported provider type: {}",
                    provider_type
                ));
            }
        };

        opencode_provider::ProviderAdapter::new(provider)
    };

    {
        use opencode_tools::registry::ToolRegistry;
        use opencode_tools::tools;
        let mut tool_registry = ToolRegistry::new();
        tools::register_all_tools(&mut tool_registry);

        let tools: Vec<Arc<dyn opencode_core::tool::Tool>> = tool_registry
            .list()
            .iter()
            .filter_map(|id| tool_registry.get(id))
            .cloned()
            .collect();

        let mut agent_manager = AgentManager::new();
        if let Err(e) = agent_manager.switch(agent_name) {
            tracing::error!(error = %e, "Failed to switch agent");
            let _ = tx.send((
                session_id_owned.clone(),
                SessionUpdate::Reply(Some(format!("Error switching agent: {}", e))),
            ));
            let _ = tx.send((session_id_owned.clone(), SessionUpdate::Reply(None)));
            return Err(anyhow::anyhow!("Failed to switch agent: {}", e));
        }

        let ctx = Context {
            session_id: session_id_owned.clone(),
            message_id: uuid::Uuid::new_v4().to_string(),
            agent: agent_name.to_string(),
            workspace_path: workspace_path.as_ref().map(|p| p.to_string_lossy().into_owned()),
        };

        let use_deep_agent = (agent_name == "build" || agent_name == "plan")
            && !tools.is_empty()
            && provider_adapter.inner().as_llm().is_some();

        if use_deep_agent {
            send_log(
                LogLevel::Info,
                format!("using deep agent tools_count={}", tools.len()),
            );
        }

        if use_deep_agent {
            let llm = provider_adapter.inner().as_llm().unwrap();
            let tools_for_agent: Vec<Arc<dyn opencode_core::tool::Tool>> = if agent_name == "plan" {
                const READ_ONLY_IDS: &[&str] =
                    &["read", "ls", "list_files", "grep", "codesearch", "glob"];
                tools
                    .iter()
                    .filter(|t| READ_ONLY_IDS.contains(&t.id()))
                    .cloned()
                    .collect()
            } else {
                tools.clone()
            };
            let tool_ctx = ToolContext {
                session_id: ctx.session_id.clone(),
                message_id: ctx.message_id.clone(),
                agent: ctx.agent.clone(),
                call_id: None,
                workspace_path: ctx.workspace_path.clone(),
            };
            let tx_log = tx.clone();
            let session_id_log = session_id_owned.clone();
            let on_tool_call: opencode_provider::OnToolCall = Arc::new(
                move |event: opencode_provider::ToolCallEvent| {
                    let (level, message) = if let Some(ref e) = event.error {
                        (
                            LogLevel::Error,
                            format!(
                                "tool {} err input={} error={}",
                                event.tool_id, event.input_preview, e
                            ),
                        )
                    } else {
                        let out_len = event.output_len.unwrap_or(0);
                        (
                            LogLevel::Info,
                            format!(
                                "tool {} ok input={} output_len={}",
                                event.tool_id, event.input_preview, out_len
                            ),
                        )
                    };
                    let _ = tx_log.send((
                        session_id_log.clone(),
                        SessionUpdate::Log(LogEntry { level, message }),
                    ));
                },
            );
            let turn_config = opencode_provider::DeepAgentTurnConfig {
                workspace_path: workspace_path.clone(),
                read_only: agent_name == "plan",
                use_crate_filesystem: agent_name != "plan",
                on_tool_call: Some(on_tool_call),
                max_history_messages: Some(24),
                max_message_content_len: Some(4000),
                max_iterations: config.core_config().max_agent_iterations,
            };
            send_log(LogLevel::Info, "deep_agent invoke started".to_string());
            match opencode_provider::run_deep_agent_turn(
                &llm,
                &session.messages,
                effective_input,
                &tools_for_agent,
                &tool_ctx,
                turn_config,
            )
            .await
            {
                Ok(reply) => {
                    send_log(LogLevel::Info, "deep_agent invoke done".to_string());
                    session.push_message(SessionMessage {
                        role: Role::User,
                        content: effective_input.to_string(),
                        created_at: Utc::now(),
                        meta: None,
                    });
                    session.push_message(SessionMessage {
                        role: Role::Assistant,
                        content: reply.clone(),
                        created_at: Utc::now(),
                        meta: None,
                    });
                    const MAX_CHUNK_LEN: usize = 200;
                    let mut chunks: Vec<String> = Vec::new();
                    for part in reply.split_inclusive('\n') {
                        if part.len() <= MAX_CHUNK_LEN {
                            chunks.push(part.to_string());
                        } else {
                            let chars: Vec<char> = part.chars().collect();
                            for c in chars.chunks(MAX_CHUNK_LEN) {
                                chunks.push(c.iter().collect());
                            }
                        }
                    }
                    for chunk in chunks {
                        let _ =
                            tx.send((session_id_owned.clone(), SessionUpdate::Reply(Some(chunk))));
                        tokio::time::sleep(std::time::Duration::from_millis(20)).await;
                    }
                    let _ = tx.send((session_id_owned.clone(), SessionUpdate::Reply(None)));
                    let save_path = session_dir.join(&session_id_owned).join("session.json");
                    if let Err(e) = session_store::save_session(&save_path, &session) {
                        tracing::warn!("Failed to save session: {}", e);
                    }
                }
                Err(e) => {
                    send_log(
                        LogLevel::Error,
                        format!("deep_agent invoke failed: {}", e),
                    );
                    tracing::error!(error = %e, "Deep agent turn failed");
                    session.push_message(SessionMessage {
                        role: Role::User,
                        content: effective_input.to_string(),
                        created_at: Utc::now(),
                        meta: None,
                    });
                    session.push_message(SessionMessage {
                        role: Role::Assistant,
                        content: format!("Error: {}", e),
                        created_at: Utc::now(),
                        meta: None,
                    });
                    let _ = tx.send((
                        session_id_owned.clone(),
                        SessionUpdate::Reply(Some(format!("Error: {}", e))),
                    ));
                    let _ = tx.send((session_id_owned.clone(), SessionUpdate::Reply(None)));
                    let save_path = session_dir.join(&session_id_owned).join("session.json");
                    if let Err(save_err) = session_store::save_session(&save_path, &session) {
                        tracing::warn!("Failed to save session: {}", save_err);
                    }
                    return Err(anyhow::anyhow!("Deep agent failed: {}", e));
                }
            }
        } else {
            let (inner_tx, mut inner_rx) =
                mpsc::unbounded_channel::<(String, Option<String>)>();
            let tx_forward = tx.clone();
            tokio::spawn(async move {
                while let Some((id, opt)) = inner_rx.recv().await {
                    let _ = tx_forward.send((id, SessionUpdate::Reply(opt)));
                }
            });
            let stream_ok = agent_manager
                .process_stream(
                    &ctx,
                    effective_input,
                    &mut session,
                    &provider_adapter,
                    &tools,
                    inner_tx,
                )
                .await;

            if let Ok(()) = stream_ok {
                let save_path = session_dir.join(&session_id_owned).join("session.json");
                if let Err(e) = session_store::save_session(&save_path, &session) {
                    tracing::warn!("Failed to save session: {}", e);
                }
            } else {
                tracing::debug!("stream not supported, using process()");
                match agent_manager
                    .process(&ctx, effective_input, &mut session, &provider_adapter, &tools)
                    .await
                {
                    Ok(_) => {
                        let save_path =
                            session_dir.join(&session_id_owned).join("session.json");
                        if let Err(e) = session_store::save_session(&save_path, &session) {
                            tracing::warn!("Failed to save session: {}", e);
                        }
                        if let Some(last_msg) = session.messages.last() {
                            if matches!(last_msg.role, Role::Assistant) {
                                let _ = tx.send((
                                    session_id_owned.clone(),
                                    SessionUpdate::Reply(Some(last_msg.content.clone())),
                                ));
                                let _ = tx.send((session_id_owned.clone(), SessionUpdate::Reply(None)));
                            }
                        }
                    }
                    Err(e) => {
                        tracing::error!(error = %e, "Agent processing failed");
                        let _ = tx.send((
                            session_id_owned.clone(),
                            SessionUpdate::Reply(Some(format!("Error: {}", e))),
                        ));
                        let _ = tx.send((session_id_owned.clone(), SessionUpdate::Reply(None)));
                        return Err(anyhow::anyhow!("Agent processing failed: {}", e));
                    }
                }
            }
        }
    }

    Ok(())
}
