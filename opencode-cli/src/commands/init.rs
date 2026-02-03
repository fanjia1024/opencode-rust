//! Create or update AGENTS.md in the project root.
//! AGENTS.md guides AI coding agents (Cursor, Copilot, etc.) with project overview, build, test, and conventions.
//! When langchain feature is enabled, uses the configured LLM to generate content from project scan; otherwise uses rule-based placeholder.

use anyhow::Result;
use std::path::Path;

const DEFAULT_IGNORE: &[&str] = &[
    ".git",
    "target",
    "node_modules",
    ".opencode",
    "logs",
    "dist",
    "build",
    "__pycache__",
];
const TREE_MAX_DEPTH: usize = 3;
const README_PREVIEW_LINES: usize = 30;

/// Result of scanning the project directory (tree, stack hints, README preview).
#[derive(Debug, Default)]
pub struct ScanResult {
    pub tree: String,
    pub workspace_members: Vec<String>,
    pub has_package_json: bool,
    pub readme_preview: String,
    pub stack_hint: String,
}

/// Builds a directory tree string (max depth, with ignores).
fn build_tree(project_root: &Path) -> String {
    use walkdir::WalkDir;

    let mut lines = Vec::new();
    for entry in WalkDir::new(project_root)
        .max_depth(TREE_MAX_DEPTH)
        .into_iter()
        .filter_entry(|e| {
            let name = e.file_name().to_string_lossy();
            !DEFAULT_IGNORE.iter().any(|ig| name.as_ref() == *ig)
        })
    {
        let entry = match entry {
            Ok(e) => e,
            Err(_) => continue,
        };
        let path = entry.path();
        let rel = match path.strip_prefix(project_root) {
            Ok(r) => r,
            Err(_) => continue,
        };
        let depth = rel.components().count();
        if depth == 0 {
            continue; // root
        }
        let name = rel
            .file_name()
            .map(|n| n.to_string_lossy())
            .unwrap_or_default();
        let prefix = "  ".repeat(depth - 1);
        let suffix = if entry.file_type().is_dir() { "/" } else { "" };
        lines.push(format!("{}{}{}\n", prefix, name, suffix));
    }
    lines.join("")
}

/// Parse Cargo.toml for [workspace] members (best-effort line-based).
fn parse_cargo_workspace_members(cargo_path: &Path) -> Vec<String> {
    let content = match std::fs::read_to_string(cargo_path) {
        Ok(c) => c,
        Err(_) => return Vec::new(),
    };
    let mut in_members = false;
    let mut members = Vec::new();
    for line in content.lines() {
        let trimmed = line.trim();
        if trimmed.starts_with("members") && trimmed.contains('[') {
            in_members = true;
            // maybe members = [ "a", "b" ] on same line
            if let Some(rest) = trimmed.split('[').nth(1) {
                for part in rest.split(',') {
                    let s = part
                        .trim()
                        .trim_matches(|c| c == '"' || c == ' ' || c == ']');
                    if !s.is_empty() {
                        members.push(s.to_string());
                    }
                }
                if trimmed.contains(']') {
                    break;
                }
            }
            continue;
        }
        if in_members {
            if trimmed == "]" {
                break;
            }
            let s = trimmed.trim_matches(|c: char| c == ',' || c == '"' || c == ' ' || c == ']');
            if !s.is_empty() && !s.starts_with('#') {
                members.push(s.to_string());
            }
        }
    }
    members
}

/// Scan project root: tree, Cargo workspace members, package.json, README preview.
pub fn scan_project(project_root: &Path) -> ScanResult {
    tklog::info!("scan_project start", project_root.display());

    let tree = build_tree(project_root);
    let cargo_path = project_root.join("Cargo.toml");
    let workspace_members = if cargo_path.exists() {
        parse_cargo_workspace_members(&cargo_path)
    } else {
        Vec::new()
    };
    let has_package_json = project_root.join("package.json").exists();
    let readme_preview = std::fs::read_to_string(project_root.join("README.md"))
        .ok()
        .map(|s| {
            s.lines()
                .take(README_PREVIEW_LINES)
                .collect::<Vec<_>>()
                .join("\n")
        })
        .unwrap_or_default();

    let stack_hint = if !workspace_members.is_empty() {
        format!("Rust workspace: {}", workspace_members.join(", "))
    } else if has_package_json {
        "Node/JS project (package.json)".to_string()
    } else {
        "Unknown".to_string()
    };

    tklog::info!(
        "scan_project done",
        "tree_lines",
        tree.lines().count(),
        "members",
        workspace_members.len()
    );
    ScanResult {
        tree,
        workspace_members,
        has_package_json,
        readme_preview,
        stack_hint,
    }
}

/// Generate AGENTS.md content from scan result (rule-based fallback when LLM is not used or fails).
pub fn generate_agents_md_content(_project_root: &Path, scan: &ScanResult) -> String {
    let tree_section = if scan.tree.is_empty() {
        "(no directory tree captured)".to_string()
    } else {
        scan.tree.clone()
    };
    let overview = if scan.readme_preview.is_empty() {
        "<!-- Describe the project: purpose, stack, main entrypoints -->".to_string()
    } else {
        format!(
            "{}\n\n<!-- (README preview above) -->",
            scan.readme_preview.trim()
        )
    };
    let build_cmd = if scan.has_package_json {
        "npm install && npm run build (or your build command)"
    } else {
        "cargo build (or your build command)"
    };
    let test_cmd = if scan.has_package_json {
        "npm test (or your test command)"
    } else {
        "cargo test (or your test command)"
    };

    format!(
        r#"# AGENTS.md

This file guides AI coding agents (Cursor, Copilot, etc.) with project context.

## Project overview

{}

- **Stack**: {}
- **Entry**: (main binary or library entrypoints)

## Directory structure

```
{}
```

## Build

- `{}`

## Test

- `{}`

## Conventions

<!-- Coding style, patterns, and conventions for this codebase -->

---
*Generated by opencode init*
"#,
        overview, scan.stack_hint, tree_section, build_cmd, test_cmd
    )
}

/// Strip optional ```markdown ... ``` wrapper from LLM output.
fn strip_markdown_fence(content: &str) -> &str {
    let content = content.trim();
    if let Some(rest) = content.strip_prefix("```markdown") {
        return rest.trim().strip_suffix("```").unwrap_or(rest).trim();
    }
    if let Some(rest) = content.strip_prefix("```") {
        return rest.trim().strip_suffix("```").unwrap_or(rest).trim();
    }
    content
}

/// Returns true when we should try DeepAgent for this provider. For openai we always try (requests use base_url).
/// For ollama we do not enable the crate's ollama feature, so false.
fn is_deep_agent_supported(provider_type: &str, _model_name: &str) -> bool {
    match provider_type {
        "openai" => true,
        "ollama" => false, // langchain-ai-rust ollama feature not enabled
        _ => false,
    }
}

/// Model names that langchain-ai-rust recognizes as OpenAI (gpt-*, o1-*, othello-*).
fn is_openai_model_recognized(model_name: &str) -> bool {
    model_name.starts_with("gpt-")
        || model_name.starts_with("o1-")
        || model_name.starts_with("othello-")
}

/// For create_deep_agent we must pass a model name the crate recognizes as OpenAI. When provider is openai
/// and the configured model is not recognized, use a fallback so the library uses the OpenAI client; base_url is already set.
fn effective_deep_agent_model(provider_type: &str, model_name: &str) -> String {
    if provider_type == "openai" && !is_openai_model_recognized(model_name) {
        "gpt-4o-mini".to_string()
    } else {
        model_name.to_string()
    }
}

/// Tries to generate content via LLM; returns Some(content) to write, or None if fallback was already written.
async fn try_llm_then_fallback(
    project_root: &Path,
    scan: &ScanResult,
    path: &std::path::Path,
) -> Result<Option<String>> {
    use opencode_core::agent::Provider;

    let config =
        crate::config::AppConfig::load().unwrap_or_else(|_| crate::config::AppConfig::default());
    let provider_info = config.get_default_provider();
    let api_key = provider_info
        .as_ref()
        .and_then(|p| p.api_key.clone())
        .or_else(|| std::env::var("OPENAI_API_KEY").ok())
        .or_else(|| std::env::var("OPENCODE_OPENAI_API_KEY").ok())
        .unwrap_or_else(|| "".to_string());
    let can_use_llm = provider_info.is_some() && !api_key.trim().is_empty();

    if !can_use_llm {
        tklog::info!("no provider or API key, using rule-based content");
        let fallback = generate_agents_md_content(project_root, scan);
        std::fs::write(path, &fallback)?;
        tklog::info!("AGENTS.md written (fallback)", path.display());
        return Ok(None);
    }

    let provider_type = provider_info
        .as_ref()
        .map(|p| p.provider_type.clone())
        .unwrap_or_else(|| "openai".to_string());
    let base_url = provider_info.as_ref().and_then(|p| p.base_url.clone());
    let model = provider_info.as_ref().and_then(|p| p.model.clone());
    let model_name = model.as_deref().unwrap_or(if provider_type == "ollama" {
        "llama3.2"
    } else {
        "gpt-4o-mini"
    });

    // Try DeepAgent when supported; with custom base_url pass real model name; otherwise library-recognized name
    if (provider_type == "openai" || provider_type == "ollama")
        && is_deep_agent_supported(&provider_type, model_name)
    {
        let model_for_deep_agent = if base_url.is_some() {
            model_name.to_string()
        } else {
            effective_deep_agent_model(&provider_type, model_name)
        };
        tklog::info!("trying DeepAgent for init", &provider_type);
        match opencode_provider::try_deep_agent_agents_md(
            project_root,
            &model_for_deep_agent,
            &api_key,
            base_url.as_deref(),
        )
        .await
        {
            Ok(None) => {
                tklog::info!("DeepAgent wrote AGENTS.md");
                return Ok(None);
            }
            Ok(Some(content)) => {
                tklog::info!("DeepAgent returned content, writing AGENTS.md");
                std::fs::write(path, content)?;
                tklog::info!("AGENTS.md written (from DeepAgent content)", path.display());
                return Ok(None);
            }
            Err(e) => {
                tklog::info!("DeepAgent failed, falling back to single generate", &e);
            }
        }
    } else if provider_type == "ollama" {
        tklog::info!(
            "Skipping DeepAgent for ollama (crate feature not enabled)",
            &provider_type
        );
    }

    let provider = match provider_type.as_str() {
        "openai" => opencode_provider::LangChainAdapter::from_openai(
            api_key.clone(),
            base_url.clone(),
            model.clone(),
        )
        .ok()
        .map(std::sync::Arc::new),
        "ollama" => opencode_provider::LangChainAdapter::from_ollama(base_url, model)
            .ok()
            .map(std::sync::Arc::new),
        "qwen" => opencode_provider::LangChainAdapter::from_qwen(api_key, base_url, model)
            .ok()
            .map(std::sync::Arc::new),
        "anthropic" => opencode_provider::LangChainAdapter::from_anthropic(api_key)
            .ok()
            .map(std::sync::Arc::new),
        _ => None,
    };

    let Some(provider) = provider else {
        tklog::info!("provider build failed or unsupported type, using rule-based content");
        let fallback = generate_agents_md_content(project_root, scan);
        std::fs::write(path, &fallback)?;
        tklog::info!("AGENTS.md written (fallback)", path.display());
        return Ok(None);
    };

    let adapter = opencode_provider::ProviderAdapter::new(provider);
    let context = format!(
        "Directory structure:\n```\n{}\n```\n\nStack / workspace: {}\n\nREADME preview:\n{}",
        scan.tree,
        scan.stack_hint,
        if scan.readme_preview.is_empty() {
            "(none)"
        } else {
            scan.readme_preview.as_str()
        }
    );
    let prompt = format!(
        r#"Generate an AGENTS.md file for this project. Use the following context.

{}

Output a single Markdown document with these sections: ## Project overview, ## Directory structure (you may summarize or reference the tree above), ## Build, ## Test, ## Conventions. Write only the Markdown body, no extra commentary. Use clear, concise bullet points where appropriate."#,
        context
    );
    let messages = vec![opencode_core::agent::Message {
        role: opencode_core::agent::MessageRole::User,
        content: prompt,
    }];
    let request = opencode_core::agent::ProviderRequest {
        messages,
        model: None,
        temperature: Some(0.3),
        max_tokens: Some(4096),
    };
    match adapter.generate(request).await {
        Ok(response) => {
            tklog::info!("LLM generate ok, writing AGENTS.md");
            Ok(Some(String::from(strip_markdown_fence(&response.content))))
        }
        Err(e) => {
            tklog::info!("LLM generate failed", &e);
            Ok(Some(generate_agents_md_content(project_root, scan)))
        }
    }
}

/// Creates or updates AGENTS.md in `project_root`.
/// Always runs scan + generate + write (update); does not skip when file already exists.
/// Uses LLM to generate content when langchain feature is enabled and provider is configured; otherwise uses rule-based content.
/// Returns `true` when the file was written.
pub async fn init_agents_md(project_root: &Path, _force: bool) -> Result<bool> {
    tklog::info!("init_agents_md called", project_root.display());
    let path = project_root.join("AGENTS.md");
    tklog::info!("AGENTS.md path", path.display());

    let scan = scan_project(project_root);
    tklog::info!("scan completed, generating content");

    let content = match try_llm_then_fallback(project_root, &scan, &path).await? {
        None => return Ok(true), // already written (e.g. by DeepAgent)
        Some(s) => s,
    };

    tklog::info!("writing AGENTS.md", path.display());
    std::fs::write(&path, content)?;
    tklog::info!("AGENTS.md written", path.display());
    Ok(true)
}

/// CLI entry: create or update AGENTS.md in the current directory.
pub async fn run_init(refresh: bool) -> Result<()> {
    let cwd = std::env::current_dir()?;
    init_agents_md(&cwd, refresh).await?;
    println!("Created or updated AGENTS.md");
    Ok(())
}
