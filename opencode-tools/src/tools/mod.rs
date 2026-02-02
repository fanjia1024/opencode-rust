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
    registry.register(read::ReadTool::new());
    registry.register(write::WriteTool::new());
    registry.register(ls::ListTool::new());
    registry.register(grep::GrepTool::new());
    registry.register(glob::GlobTool::new());
    registry.register(edit::EditTool::new());
    registry.register(multiedit::MultiEditTool::new());
    registry.register(patch::PatchTool::new());
    registry.register(bash::BashTool::new());
    // registry.register(batch::BatchTool::new()); // BatchTool requires special setup
    registry.register(lsp::LspTool::new());
    registry.register(question::QuestionTool::new());
    registry.register(webfetch::WebFetchTool::new());
    registry.register(websearch::WebSearchTool::new());
    registry.register(codesearch::CodeSearchTool::new());
    registry.register(task::TaskTool::new());
    registry.register(todo::TodoTool::new());
}
