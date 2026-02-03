//! Deep Agent integration for init: analyze project with filesystem tools and write AGENTS.md.
//! When this succeeds, the agent may have written AGENTS.md itself; otherwise we return content to write.

use langchain_ai_rust::agent::{create_deep_agent, create_deep_agent_from_llm, DeepAgentConfig};
use langchain_ai_rust::llm::openai::{OpenAI, OpenAIConfig};
use langchain_ai_rust::schemas::messages::Message;
use opencode_core::error::{Error, Result};
use std::path::Path;

const SYSTEM_PROMPT: &str = "You are a deep agent with planning and file system tools. \
Your task is to analyze a codebase and write an AGENTS.md file. \
Use write_todos to break the work into steps if needed. \
Use ls, read_file, write_file, edit_file to explore the workspace and then write AGENTS.md in the project root. \
AGENTS.md must contain these sections: ## Project overview, ## Directory structure, ## Build, ## Test, ## Conventions. \
Write only the file; do not output the full content in your reply.";

const USER_MESSAGE: &str = "Analyze this project (directory structure, framework, entry points, build and test commands, conventions) and write AGENTS.md in the project root with sections: Project overview, Directory structure, Build, Test, Conventions. Use the file system tools to explore, then use write_file to create AGENTS.md.";

/// Tries to run a Deep Agent to analyze the project and write AGENTS.md.
/// When base_url is set, builds an OpenAI LLM with api_key + base_url + model_name and uses
/// create_deep_agent_from_llm so requests go to the user's endpoint. When base_url is None,
/// sets OPENAI_API_KEY and uses create_deep_agent (default OpenAI endpoint).
/// Returns Ok(None) if the agent wrote AGENTS.md (caller should not write again).
/// Returns Ok(Some(content)) if the agent returned content but did not write the file (caller should write content).
/// Returns Err on creation or invocation failure (caller should fall back to single-call generate).
pub async fn try_deep_agent_agents_md(
    project_root: &Path,
    model_name: &str,
    api_key: &str,
    base_url: Option<&str>,
) -> Result<Option<String>> {
    let workspace_root = project_root.to_path_buf();
    let agent_config = DeepAgentConfig::new()
        .with_planning(true)
        .with_filesystem(true)
        .with_workspace_root(workspace_root);

    let agent = if let Some(url) = base_url {
        let openai_config = OpenAIConfig::default()
            .with_api_key(api_key)
            .with_api_base(url);
        let llm = OpenAI::new(openai_config).with_model(model_name);
        create_deep_agent_from_llm(llm, &[], Some(SYSTEM_PROMPT), agent_config)
            .map_err(|e| Error::Provider(format!("DeepAgent create failed: {}", e)))?
    } else {
        std::env::set_var("OPENAI_API_KEY", api_key);
        create_deep_agent(model_name, &[], Some(SYSTEM_PROMPT), agent_config)
            .map_err(|e| Error::Provider(format!("DeepAgent create failed: {}", e)))?
    };
    tracing::info!(model = %model_name, "DeepAgent created, invoking");

    let messages = vec![Message::new_human_message(USER_MESSAGE)];
    let result = agent
        .invoke_messages(messages)
        .await
        .map_err(|e| Error::Provider(format!("DeepAgent invoke failed: {}", e)))?;
    tracing::info!("DeepAgent invoke completed");

    let agents_md_path = project_root.join("AGENTS.md");
    if agents_md_path.exists() {
        return Ok(None);
    }
    // Agent did not write file; use response as content if non-empty
    let s = result.trim();
    if s.is_empty() {
        return Err(Error::Provider(
            "DeepAgent did not write AGENTS.md and returned empty response".to_string(),
        ));
    }
    Ok(Some(s.to_string()))
}
