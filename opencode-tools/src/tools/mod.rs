pub mod bash;
pub mod batch;
pub mod codesearch;
pub mod edit;
pub mod glob;
pub mod grep;
pub mod ls;
pub mod lsp;
pub mod multiedit;
pub mod patch;
pub mod question;
pub mod read;
pub mod task;
pub mod todo;
pub mod tool_wrapper;
pub mod webfetch;
pub mod websearch;
pub mod write;

#[cfg(test)]
mod tests;

use crate::registry::ToolRegistry;

pub fn register_all_tools(registry: &mut ToolRegistry) {
    // Essential tools for terminal coding experience
    registry.register(read::ReadTool::new());
    registry.register(write::WriteTool::new());
    registry.register(ls::ListTool::new());
    registry.register(grep::GrepTool::new());
    registry.register(edit::EditTool::new());
    registry.register(patch::PatchTool::new());
    registry.register(bash::BashTool::new());
    
    // Optional but useful tools
    registry.register(glob::GlobTool::new());
    registry.register(multiedit::MultiEditTool::new());
    registry.register(codesearch::CodeSearchTool::new());
    
    // Removed tools that are not terminal-oriented or directly useful in coding chat:
    // - lsp::LspTool (external dependency, may not be available)
    // - question::QuestionTool (might be redundant)
    // - webfetch::WebFetchTool (external network calls)
    // - websearch::WebSearchTool (external network calls)
    // - task::TaskTool (workflow functionality)
    // - todo::TodoTool (workflow functionality)
    // - batch::BatchTool (workflow functionality)
}
