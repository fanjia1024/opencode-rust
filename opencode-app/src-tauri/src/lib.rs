#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use opencode_cli::message_processor::{process_message_async, LogLevel, SessionUpdate};
use opencode_cli::{config::AppConfig, load_session, save_session};
use opencode_core::ids::SessionId;
use opencode_core::session::Session;
use opencode_core::CommandDef;
use std::path::PathBuf;
use std::str::FromStr;
use std::sync::Mutex;
use tauri::{AppHandle, Emitter, State};
use tokio::sync::mpsc;
use tracing_subscriber::{fmt, layer::SubscriberExt, util::SubscriberInitExt, EnvFilter};

fn init_tracing() {
    let _ = std::fs::create_dir_all("logs");
    let filter = EnvFilter::try_from_default_env()
        .unwrap_or_else(|_| EnvFilter::new("info"));
    if let Ok(file) = std::fs::OpenOptions::new()
        .create(true)
        .append(true)
        .open("logs/opencode.log")
    {
        let file_layer = fmt::Layer::new().with_writer(file).with_ansi(false);
        let registry = tracing_subscriber::registry().with(filter).with(file_layer);
        #[cfg(debug_assertions)]
        let registry = registry.with(fmt::Layer::new().with_writer(std::io::stderr));
        let _ = registry.try_init();
    }
}

struct AppState {
    workspace_path: Mutex<Option<PathBuf>>,
    current_agent: Mutex<String>,
}

fn effective_workspace(state: &AppState) -> Option<PathBuf> {
    state
        .workspace_path
        .lock()
        .ok()
        .and_then(|g| g.clone())
        .or_else(|| std::env::current_dir().ok())
}

fn sync_current_agent_from_config(state: &AppState, config: &AppConfig) {
    let name = config
        .get_default_agent()
        .unwrap_or_else(|| "build".to_string());
    if let Ok(mut g) = state.current_agent.lock() {
        *g = name;
    }
}

fn read_last_workspace() -> Option<PathBuf> {
    let dir = opencode_core::config::Config::config_dir().ok()?;
    let p = dir.join("last_workspace");
    let s = std::fs::read_to_string(&p).ok()?;
    let s = s.trim();
    if s.is_empty() {
        None
    } else {
        Some(PathBuf::from(s))
    }
}

fn write_last_workspace(path: Option<&PathBuf>) {
    if let Ok(dir) = opencode_core::config::Config::config_dir() {
        let p = dir.join("last_workspace");
        if let Some(path) = path {
            let _ = std::fs::write(&p, path.to_string_lossy().as_bytes());
        } else {
            let _ = std::fs::remove_file(&p);
        }
    }
}

#[derive(serde::Serialize)]
struct SessionListItem {
    id: String,
    updated_at: String,
}

#[derive(serde::Serialize)]
struct ProviderInfoDto {
    provider_type: String,
    api_key: Option<String>,
    base_url: Option<String>,
    model: Option<String>,
}

#[derive(serde::Serialize)]
struct ProviderListItemDto {
    id: String,
    provider_type: String,
    model: Option<String>,
    base_url: Option<String>,
}

#[tauri::command]
fn list_sessions(state: State<AppState>) -> Result<Vec<SessionListItem>, String> {
    let workspace = effective_workspace(&state).ok_or_else(|| "No workspace path".to_string())?;
    let config = AppConfig::load_from_workspace(&workspace).map_err(|e| e.to_string())?;
    sync_current_agent_from_config(&state, &config);
    let session_dir = config.session_dir();
    if !session_dir.exists() {
        return Ok(Vec::new());
    }
    let mut items = Vec::new();
    for entry in std::fs::read_dir(&session_dir).map_err(|e| e.to_string())? {
        let entry = entry.map_err(|e| e.to_string())?;
        let path = entry.path();
        if path.is_dir() {
            let session_file = path.join("session.json");
            if session_file.exists() {
                let id = path
                    .file_name()
                    .and_then(|n| n.to_str())
                    .unwrap_or("")
                    .to_string();
                if id.is_empty() {
                    continue;
                }
                let updated_at = std::fs::metadata(&session_file)
                    .and_then(|m| m.modified())
                    .map(|t| {
                        chrono::DateTime::from_timestamp(
                            t.duration_since(std::time::UNIX_EPOCH)
                                .unwrap()
                                .as_secs() as i64,
                            0,
                        )
                        .map(|dt| dt.format("%Y-%m-%d %H:%M").to_string())
                        .unwrap_or_else(|| "".to_string())
                    })
                    .unwrap_or_else(|_| "".to_string());
                items.push(SessionListItem { id, updated_at });
            }
        }
    }
    items.sort_by(|a, b| b.updated_at.cmp(&a.updated_at));
    Ok(items)
}

#[tauri::command]
fn get_session(session_id: String, state: State<AppState>) -> Result<Session, String> {
    let workspace = effective_workspace(&state).ok_or_else(|| "No workspace path".to_string())?;
    let config = AppConfig::load_from_workspace(&workspace).map_err(|e| e.to_string())?;
    sync_current_agent_from_config(&state, &config);
    let session_file = config.session_dir().join(&session_id).join("session.json");
    if !session_file.exists() {
        return Err("Session not found".to_string());
    }
    load_session(&session_file).map_err(|e| e.to_string())
}

#[tauri::command]
fn create_session(state: State<AppState>) -> Result<String, String> {
    let workspace = effective_workspace(&state).ok_or_else(|| "No workspace path".to_string())?;
    let config = AppConfig::load_from_workspace(&workspace).map_err(|e| e.to_string())?;
    sync_current_agent_from_config(&state, &config);
    let id = uuid::Uuid::new_v4().to_string();
    let path = config.session_dir().join(&id).join("session.json");
    let session = Session::with_id(
        SessionId::from_str(&id).unwrap_or_else(|_| SessionId::new()),
    );
    save_session(&path, &session).map_err(|e| e.to_string())?;
    Ok(id)
}

#[tauri::command]
fn delete_session(session_id: String, state: State<AppState>) -> Result<(), String> {
    let workspace = effective_workspace(&state).ok_or_else(|| "No workspace path".to_string())?;
    let config = AppConfig::load_from_workspace(&workspace).map_err(|e| e.to_string())?;
    sync_current_agent_from_config(&state, &config);
    let dir = config.session_dir().join(&session_id);
    if dir.exists() {
        std::fs::remove_dir_all(&dir).map_err(|e| e.to_string())?;
    }
    Ok(())
}

#[tauri::command]
fn get_workspace_path(state: State<AppState>) -> Option<PathBuf> {
    state.workspace_path.lock().ok().and_then(|g| g.clone())
}

#[tauri::command]
fn set_workspace_path(path: Option<PathBuf>, state: State<AppState>) {
    if let Ok(mut g) = state.workspace_path.lock() {
        *g = path.clone();
    }
    write_last_workspace(path.as_ref());
}

#[tauri::command]
fn list_commands(state: State<AppState>) -> Vec<CommandDef> {
    let workspace = effective_workspace(&state);
    let config = workspace
        .as_ref()
        .map(|w| AppConfig::load_from_workspace(w).unwrap_or_else(|_| AppConfig::default()))
        .unwrap_or_else(|| AppConfig::default());
    opencode_core::list_commands(workspace.as_deref(), config.core_config())
}

#[tauri::command]
async fn send_message(
    session_id: String,
    input: String,
    command: Option<String>,
    app: AppHandle,
    state: State<'_, AppState>,
) -> Result<(), String> {
    let workspace_path = effective_workspace(&state);
    let config = workspace_path
        .as_ref()
        .map(|w| AppConfig::load_from_workspace(w))
        .transpose()
        .map_err(|e| e.to_string())?
        .unwrap_or_else(|| AppConfig::default());
    let workspace_path = workspace_path.or_else(|| std::env::current_dir().ok());
    let agent_name = state
        .current_agent
        .lock()
        .map(|g| g.clone())
        .unwrap_or_else(|_| "build".to_string());

    let (tx, mut rx) = mpsc::unbounded_channel::<(String, SessionUpdate)>();

    let app_handle = app.clone();
    tokio::spawn(async move {
        while let Some((sid, update)) = rx.recv().await {
            match &update {
                SessionUpdate::Reply(Some(chunk)) => {
                    let _ = app_handle.emit("session-reply-chunk", serde_json::json!({ "session_id": sid, "content": chunk }));
                }
                SessionUpdate::Reply(None) => {
                    let _ = app_handle.emit("session-reply-done", serde_json::json!({ "session_id": sid }));
                }
                SessionUpdate::Log(entry) => {
                    let level = match entry.level {
                        LogLevel::Info => "info",
                        LogLevel::Warn => "warn",
                        LogLevel::Error => "error",
                    };
                    let _ = app_handle.emit(
                        "session-log",
                        serde_json::json!({ "session_id": sid, "level": level, "message": entry.message }),
                    );
                }
            }
        }
    });

    process_message_async(
        &session_id,
        &input,
        &agent_name,
        config,
        workspace_path,
        tx,
        command,
    )
    .await
    .map_err(|e| e.to_string())
}

#[tauri::command]
fn get_config(state: State<AppState>) -> Result<serde_json::Value, String> {
    let workspace = effective_workspace(&state).ok_or_else(|| "No workspace path".to_string())?;
    let config = AppConfig::load_from_workspace(&workspace).map_err(|e| e.to_string())?;
    sync_current_agent_from_config(&state, &config);
    let session_dir = config.session_dir();
    Ok(serde_json::json!({
        "workspace_path": workspace.to_string_lossy(),
        "session_dir": session_dir.to_string_lossy(),
        "default_agent": config.get_default_agent()
    }))
}

#[tauri::command]
fn get_providers(state: State<AppState>) -> Result<Vec<ProviderListItemDto>, String> {
    let workspace = effective_workspace(&state).ok_or_else(|| "No workspace path".to_string())?;
    let config = AppConfig::load_from_workspace(&workspace).map_err(|e| e.to_string())?;
    sync_current_agent_from_config(&state, &config);
    Ok(config
        .list_providers()
        .into_iter()
        .map(|p| ProviderListItemDto {
            id: p.id,
            provider_type: p.provider_type,
            model: p.model,
            base_url: p.base_url,
        })
        .collect())
}

#[tauri::command]
fn set_provider_config(
    provider_id: String,
    provider_type: String,
    api_key: String,
    base_url: Option<String>,
    model: Option<String>,
    state: State<AppState>,
) -> Result<(), String> {
    let workspace = effective_workspace(&state).ok_or_else(|| "No workspace path".to_string())?;
    let mut config = AppConfig::load_from_workspace(&workspace).map_err(|e| e.to_string())?;
    config.set_provider_config_unsaved(&provider_id, provider_type, api_key, base_url, model);
    config.save_to_workspace(&workspace).map_err(|e| e.to_string())
}

#[tauri::command]
fn set_default_provider(id: String, state: State<AppState>) -> Result<(), String> {
    let workspace = effective_workspace(&state).ok_or_else(|| "No workspace path".to_string())?;
    let mut config = AppConfig::load_from_workspace(&workspace).map_err(|e| e.to_string())?;
    config.set_default_provider_id_unsaved(&id);
    config.save_to_workspace(&workspace).map_err(|e| e.to_string())
}

#[tauri::command]
fn list_agents() -> Vec<String> {
    opencode_core::AgentManager::new().list()
}

#[tauri::command]
fn get_current_agent(state: State<AppState>) -> String {
    state
        .current_agent
        .lock()
        .map(|g| g.clone())
        .unwrap_or_else(|_| "build".to_string())
}

#[tauri::command]
fn set_agent(name: String, state: State<AppState>) -> Result<(), String> {
    let list = opencode_core::AgentManager::new().list();
    if !list.contains(&name) {
        return Err(format!("Unknown agent: {}", name));
    }
    if let Ok(mut g) = state.current_agent.lock() {
        *g = name.clone();
    }
    if let Some(workspace) = effective_workspace(&state) {
        let mut config = AppConfig::load_from_workspace(&workspace).map_err(|e| e.to_string())?;
        config.set_default_agent(Some(name));
        config.save_to_workspace(&workspace).map_err(|e| e.to_string())?;
    }
    Ok(())
}

#[tauri::command]
async fn init_agents_md(project_root: Option<PathBuf>) -> Result<String, String> {
    let root = project_root
        .or_else(|| std::env::current_dir().ok())
        .ok_or_else(|| "Could not determine project root".to_string())?;
    let result = opencode_cli::commands::init::init_agents_md(&root, false).await;
    match result {
        Ok(true) => Ok("AGENTS.md created or updated.".to_string()),
        Ok(false) => Ok("AGENTS.md already exists.".to_string()),
        Err(e) => Err(e.to_string()),
    }
}

pub fn run() {
    init_tracing();
    tracing::info!("opencode-app starting");
    let initial_workspace = read_last_workspace();
    tauri::Builder::default()
        .plugin(tauri_plugin_shell::init())
        .manage(AppState {
            workspace_path: Mutex::new(initial_workspace),
            current_agent: Mutex::new("build".to_string()),
        })
        .invoke_handler(tauri::generate_handler![
            list_sessions,
            get_session,
            create_session,
            delete_session,
            get_workspace_path,
            set_workspace_path,
            list_commands,
            send_message,
            get_config,
            get_providers,
            set_provider_config,
            set_default_provider,
            list_agents,
            get_current_agent,
            set_agent,
            init_agents_md,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
