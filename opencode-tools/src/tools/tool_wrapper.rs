#[cfg(feature = "langchain")]
use opencode_core::error::Result;
#[cfg(feature = "langchain")]
use opencode_provider::langchain_tool_adapter::LangChainToolAdapter;
#[cfg(feature = "langchain")]
use std::sync::Arc;

#[cfg(feature = "langchain")]
pub fn wrap_for_langchain(tool: Arc<dyn opencode_core::tool::Tool>) -> LangChainToolAdapter {
    LangChainToolAdapter::new(tool)
}

#[cfg(feature = "langchain")]
pub fn create_tool_registry_for_agent() -> Result<Vec<LangChainToolAdapter>> {
    use crate::tools::*;
    
    let mut registry = crate::registry::ToolRegistry::new();
    register_all_tools(&mut registry);
    
    let tools: Vec<LangChainToolAdapter> = registry
        .list()
        .iter()
        .filter_map(|id| {
            registry.get(id).map(|tool| wrap_for_langchain(tool.clone()))
        })
        .collect();
    
    Ok(tools)
}

#[cfg(not(feature = "langchain"))]
pub fn wrap_for_langchain(_tool: std::sync::Arc<dyn opencode_core::tool::Tool>) -> () {
    // No-op when langchain feature is disabled
}

#[cfg(not(feature = "langchain"))]
pub fn create_tool_registry_for_agent() -> opencode_core::error::Result<Vec<()>> {
    Ok(vec![])
}
