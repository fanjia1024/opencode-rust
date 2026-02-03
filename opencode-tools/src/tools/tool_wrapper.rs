use opencode_core::error::Result;
use opencode_provider::langchain_tool_adapter::LangChainToolAdapter;
use std::sync::Arc;

pub fn wrap_for_langchain(tool: Arc<dyn opencode_core::tool::Tool>) -> LangChainToolAdapter {
    LangChainToolAdapter::new(tool)
}

pub fn create_tool_registry_for_agent() -> Result<Vec<LangChainToolAdapter>> {
    use crate::tools::*;

    let mut registry = crate::registry::ToolRegistry::new();
    register_all_tools(&mut registry);

    let tools: Vec<LangChainToolAdapter> = registry
        .list()
        .iter()
        .filter_map(|id| registry.get(id).map(|tool| wrap_for_langchain(tool.clone())))
        .collect();

    Ok(tools)
}
