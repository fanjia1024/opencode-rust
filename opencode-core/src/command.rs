//! Session commands: load from config + Markdown, expand template ($ARGUMENTS, $1-$n, !`cmd`, @path).
//! Aligned with https://opencode.ai/docs/commands/

use crate::config::Config;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::Path;
use std::process::Command;

/// Definition of a session command, serializable for the frontend.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CommandDef {
    pub id: String,
    pub label: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    /// Full prompt template (supports $ARGUMENTS, $1-$n, !`cmd`, @path). Not sent to frontend if sensitive.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub template: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub agent: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub model: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub subtask: Option<bool>,
}

#[derive(Debug, Deserialize)]
struct Frontmatter {
    #[serde(default)]
    description: Option<String>,
    #[serde(default)]
    agent: Option<String>,
    #[serde(default)]
    model: Option<String>,
    #[serde(default)]
    subtask: Option<bool>,
}

fn builtin_commands() -> Vec<CommandDef> {
    vec![
        CommandDef {
            id: "init".to_string(),
            label: "init".to_string(),
            description: Some("Initialize project".to_string()),
            template: Some("User requested /init. Follow the standard project initialization steps for this codebase.".to_string()),
            agent: None,
            model: None,
            subtask: None,
        },
        CommandDef {
            id: "undo".to_string(),
            label: "undo".to_string(),
            description: Some("Undo last change".to_string()),
            template: Some("User requested /undo. Revert the last edit or change as appropriate.".to_string()),
            agent: None,
            model: None,
            subtask: None,
        },
        CommandDef {
            id: "redo".to_string(),
            label: "redo".to_string(),
            description: Some("Redo last undone change".to_string()),
            template: Some("User requested /redo. Reapply the last reverted change.".to_string()),
            agent: None,
            model: None,
            subtask: None,
        },
        CommandDef {
            id: "share".to_string(),
            label: "share".to_string(),
            description: Some("Share conversation".to_string()),
            template: Some("User requested /share. Help prepare or export this conversation for sharing.".to_string()),
            agent: None,
            model: None,
            subtask: None,
        },
        CommandDef {
            id: "help".to_string(),
            label: "help".to_string(),
            description: Some("Show help".to_string()),
            template: Some("User requested /help. Provide a brief overview of available commands and how to use this assistant.".to_string()),
            agent: None,
            model: None,
            subtask: None,
        },
    ]
}

fn load_commands_from_dir(md_dir: &Path) -> HashMap<String, CommandDef> {
    let mut map = HashMap::new();
    let Ok(entries) = std::fs::read_dir(md_dir) else {
        return map;
    };
    for entry in entries.flatten() {
        let path = entry.path();
        if path.extension().map_or(false, |e| e == "md") {
            let id = path
                .file_stem()
                .and_then(|s| s.to_str())
                .unwrap_or("")
                .to_string();
            if id.is_empty() {
                continue;
            }
            let content = match std::fs::read_to_string(&path) {
                Ok(c) => c,
                Err(_) => continue,
            };
            let (frontmatter, body) = parse_frontmatter(&content);
            let template = body.trim().to_string();
            if template.is_empty() {
                continue;
            }
            let label = frontmatter
                .as_ref()
                .and_then(|f| f.description.clone())
                .unwrap_or_else(|| id.clone());
            map.insert(
                id.clone(),
                CommandDef {
                    id,
                    label,
                    description: frontmatter.as_ref().and_then(|f| f.description.clone()),
                    template: Some(template),
                    agent: frontmatter.as_ref().and_then(|f| f.agent.clone()),
                    model: frontmatter.as_ref().and_then(|f| f.model.clone()),
                    subtask: frontmatter.and_then(|f| f.subtask),
                },
            );
        }
    }
    map
}

fn parse_frontmatter(content: &str) -> (Option<Frontmatter>, String) {
    let rest = content.strip_prefix("---").unwrap_or(content);
    let Some(pos) = rest.find("\n---") else {
        return (None, content.to_string());
    };
    let yaml_str = rest[..pos].trim();
    let body = rest[pos + 4..].trim();
    let front: Frontmatter = match serde_yaml::from_str(yaml_str) {
        Ok(f) => f,
        Err(_) => return (None, body.to_string()),
    };
    (Some(front), body.to_string())
}

fn load_commands_from_config(config: &Config) -> HashMap<String, CommandDef> {
    let mut map = HashMap::new();
    let Some(ref cmd_map) = config.command else {
        return map;
    };
    for (id, opt) in cmd_map {
        let template = opt.template.clone();
        let label = opt
            .description
            .clone()
            .unwrap_or_else(|| id.clone());
        map.insert(
            id.clone(),
            CommandDef {
                id: id.clone(),
                label,
                description: opt.description.clone(),
                template: Some(template),
                agent: opt.agent.clone(),
                model: opt.model.clone(),
                subtask: opt.subtask,
            },
        );
    }
    map
}

/// Returns the merged list of commands: builtin, then global (md + config), then project (md + config).
/// Custom commands override built-in when same id.
pub fn list_commands(workspace: Option<&Path>, project_config: &Config) -> Vec<CommandDef> {
    let mut map: HashMap<String, CommandDef> = HashMap::new();
    for c in builtin_commands() {
        map.insert(c.id.clone(), c);
    }
    if let Ok(global_dir) = Config::config_dir() {
        let global_commands_dir = global_dir.join("commands");
        if global_commands_dir.is_dir() {
            for (k, v) in load_commands_from_dir(&global_commands_dir) {
                map.insert(k, v);
            }
        }
    }
    if let Ok(global_config) = Config::load() {
        for (k, v) in load_commands_from_config(&global_config) {
            map.insert(k, v);
        }
    }
    if let Some(ws) = workspace {
        let project_commands_dir = ws.join(".opencode").join("commands");
        if project_commands_dir.is_dir() {
            for (k, v) in load_commands_from_dir(&project_commands_dir) {
                map.insert(k, v);
            }
        }
        for (k, v) in load_commands_from_config(project_config) {
            map.insert(k, v);
        }
    }
    let mut list: Vec<CommandDef> = map.into_values().collect();
    list.sort_by(|a, b| a.id.cmp(&b.id));
    list
}

/// Expands template: $ARGUMENTS, $1-$n, !`cmd`, @path. Returns None if command not found.
pub fn format_input_for_command(
    command_id: &str,
    user_input: &str,
    workspace: Option<&Path>,
    project_config: &Config,
) -> Option<String> {
    let commands = list_commands(workspace, project_config);
    let cmd = commands.iter().find(|c| c.id == command_id)?;
    let template = cmd.template.as_deref().unwrap_or("").to_string();
    let args_trimmed = user_input.trim();
    let positional: Vec<&str> = args_trimmed.split_whitespace().collect();

    let mut out = template
        .replace("$ARGUMENTS", args_trimmed);
    for (i, arg) in positional.iter().enumerate() {
        let placeholder = format!("${}", i + 1);
        out = out.replace(&placeholder, arg);
    }

    out = expand_shell_placeholders(&out, workspace);
    out = expand_file_placeholders(&out, workspace);

    Some(out)
}

fn expand_shell_placeholders(s: &str, work_dir: Option<&Path>) -> String {
    let mut result = String::new();
    let mut i = 0;
    while i < s.len() {
        let rest = &s[i..];
        if rest.starts_with("!`") {
            if let Some(close) = rest[2..].find('`') {
                let cmd = rest[2..2 + close].trim();
                let output = run_shell_cmd(cmd, work_dir);
                result.push_str(&output);
                i += 2 + close + 1;
                continue;
            }
        }
        if let Some(c) = rest.chars().next() {
            result.push(c);
            i += c.len_utf8();
        } else {
            break;
        }
    }
    result
}

fn run_shell_cmd(cmd: &str, work_dir: Option<&Path>) -> String {
    let mut c = Command::new("sh");
    c.arg("-c").arg(cmd);
    if let Some(dir) = work_dir {
        c.current_dir(dir);
    }
    match c.output() {
        Ok(o) if o.status.success() => String::from_utf8_lossy(&o.stdout).into_owned(),
        _ => String::new(),
    }
}

fn expand_file_placeholders(s: &str, workspace: Option<&Path>) -> String {
    let Some(ws) = workspace else {
        return s.to_string();
    };
    let mut result = String::new();
    let mut i = 0;
    while i < s.len() {
        let rest = &s[i..];
        if rest.starts_with('@') {
            let path_len = rest[1..]
                .find(|c: char| c.is_whitespace())
                .unwrap_or(rest.len().saturating_sub(1));
            let path_str = rest[1..1 + path_len].trim();
            if !path_str.is_empty() {
                let path = ws.join(path_str);
                let content = std::fs::read_to_string(&path).unwrap_or_default();
                result.push_str(&content);
                i += 1 + path_len;
                continue;
            }
        }
        if let Some(c) = rest.chars().next() {
            result.push(c);
            i += c.len_utf8();
        } else {
            break;
        }
    }
    result
}
