use crate::tui::state::{AppState, Screen, DialogState};
use crate::tui::screens::{home::HomeScreen, session::SessionScreen};
use crate::tui::screens::home::SessionInfo;
use crate::tui::screens::dialogs::provider::{ProviderDialog, ProviderConfig, DialogAction};
use crate::tui::sync::{StateSync, SessionListItem};
use crate::config::AppConfig;
use anyhow::Result;
use crossterm::event::{self, Event, KeyCode, KeyEventKind};
use crossterm::execute;
use crossterm::terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen};
use ratatui::prelude::*;
use ratatui::widgets;
use std::io;
use std::cell::RefCell;
use std::path::Path;
use std::sync::Arc;
use std::collections::HashMap;
use tokio::sync::mpsc;
use opencode_core::AgentManager;
use opencode_core::agent::Context;
use opencode_core::session::Session;

pub struct App {
    state: AppState,
    should_quit: bool,
    home_screen: HomeScreen,
    session_screen: RefCell<Option<SessionScreen>>,
    agent_manager: AgentManager,
    sessions: RefCell<HashMap<String, Session>>,
    response_tx: mpsc::UnboundedSender<(String, String)>, // (session_id, response)
    response_rx: RefCell<mpsc::UnboundedReceiver<(String, String)>>,
    session_list_rx: RefCell<mpsc::UnboundedReceiver<Vec<SessionListItem>>>,
    state_sync: StateSync,
    config: RefCell<AppConfig>,
    provider_dialog: RefCell<Option<ProviderDialog>>,
}

fn initial_session_id(session_dir: &Path) -> String {
    if !session_dir.exists() {
        return uuid::Uuid::new_v4().to_string();
    }
    let entries = match std::fs::read_dir(session_dir) {
        Ok(e) => e,
        Err(_) => return uuid::Uuid::new_v4().to_string(),
    };
    let mut latest: Option<(String, std::time::SystemTime)> = None;
    for entry in entries.flatten() {
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
                let mtime = std::fs::metadata(&session_file)
                    .and_then(|m| m.modified())
                    .unwrap_or(std::time::UNIX_EPOCH);
                if latest.is_none() || mtime > latest.as_ref().unwrap().1 {
                    latest = Some((id, mtime));
                }
            }
        }
    }
    latest
        .map(|(id, _)| id)
        .unwrap_or_else(|| uuid::Uuid::new_v4().to_string())
}

impl App {
    pub fn new() -> Self {
        let (tx, rx) = mpsc::unbounded_channel();
        let (session_list_tx, session_list_rx) = mpsc::unbounded_channel();
        let config = AppConfig::load().unwrap_or_else(|_| AppConfig::default());
        let session_dir = config.session_dir();
        let state_sync = StateSync::new(session_dir.clone(), session_list_tx);
        let initial_id = initial_session_id(&session_dir);
        let state = AppState {
            current_screen: Screen::Session(initial_id.clone()),
            ..AppState::default()
        };
        let session_screen = RefCell::new(Some(SessionScreen::new(initial_id)));
        Self {
            state,
            should_quit: false,
            home_screen: HomeScreen::new(),
            session_screen,
            agent_manager: AgentManager::new(),
            sessions: RefCell::new(HashMap::new()),
            response_tx: tx,
            response_rx: RefCell::new(rx),
            session_list_rx: RefCell::new(session_list_rx),
            state_sync,
            config: RefCell::new(config),
            provider_dialog: RefCell::new(None),
        }
    }

    pub async fn run(&mut self) -> Result<()> {
        enable_raw_mode()?;
        let mut stdout = io::stdout();
        execute!(stdout, EnterAlternateScreen)?;
        let backend = CrosstermBackend::new(stdout);
        let mut terminal = Terminal::new(backend)?;

        let session_dir = self.config.borrow().session_dir();
        if let Screen::Session(ref id) = self.state.current_screen {
            let session_file = session_dir.join(id).join("session.json");
            if session_file.exists() {
                if let Ok(session) = Session::load(id, &session_dir).await {
                    if let Some(screen) = self.session_screen.borrow_mut().as_mut() {
                        screen.load_messages(&session);
                    }
                } else {
                    tracing::debug!("Failed to load session {}", id);
                }
            }
        }

        loop {
            terminal.draw(|f| self.ui(f))?;

            // Check for responses from async tasks
            {
                let mut rx = self.response_rx.borrow_mut();
                while let Ok((session_id, response)) = rx.try_recv() {
                    drop(rx); // Release borrow before borrowing session_screen
                    if let Some(screen) = self.session_screen.borrow_mut().as_mut() {
                        if screen.session_id == session_id {
                            screen.add_message(format!("Assistant: {}", response));
                        }
                    }
                    rx = self.response_rx.borrow_mut(); // Re-borrow for next iteration
                }
            }

            // Apply session list updates from StateSync
            {
                let mut list_rx = self.session_list_rx.borrow_mut();
                while let Ok(list) = list_rx.try_recv() {
                    self.home_screen.sessions = list
                        .into_iter()
                        .map(|s| SessionInfo {
                            id: s.id,
                            title: s.title,
                            updated: s.updated,
                        })
                        .collect();
                }
            }

            if let Err(e) = self.state_sync.sync_if_needed().await {
                tracing::debug!("StateSync: {}", e);
            }

            if event::poll(std::time::Duration::from_millis(16))? {
                if let Event::Key(key) = event::read()? {
                    if key.kind == KeyEventKind::Press {
                        self.handle_key(key.code)?;
                    }
                }
            }

            if self.should_quit {
                break;
            }
        }

        disable_raw_mode()?;
        execute!(terminal.backend_mut(), LeaveAlternateScreen)?;
        Ok(())
    }

    fn ui(&self, f: &mut Frame) {
        match &self.state.current_screen {
            Screen::Home => {
                self.home_screen.render(f, f.size());
            }
            Screen::Session(session_id) => {
                let mut session_screen = self.session_screen.borrow_mut();
                if let Some(screen) = session_screen.as_mut() {
                    screen.render(f, f.size());
                } else {
                    let mut new_screen = SessionScreen::new(session_id.clone());
                    new_screen.render(f, f.size());
                }
            }
            Screen::Dialog(dialog_state) => {
                match dialog_state.as_ref() {
                    DialogState::Provider => {
                        if let Some(dialog) = self.provider_dialog.borrow().as_ref() {
                            dialog.render(f, f.size());
                        }
                    }
                    _ => {
                        let block = widgets::Block::default()
                            .title("Dialog")
                            .borders(widgets::Borders::ALL);
                        f.render_widget(block, f.size());
                    }
                }
            }
        }
    }

    fn handle_key(&mut self, key: KeyCode) -> Result<()> {
        // Check if we're in a dialog first (borrow only to get action, then release before mutating)
        if let Screen::Dialog(dialog_state) = &self.state.current_screen {
            if let DialogState::Provider = dialog_state.as_ref() {
                use crossterm::event::KeyEvent;
                let key_event = KeyEvent {
                    code: key,
                    modifiers: crossterm::event::KeyModifiers::empty(),
                    kind: KeyEventKind::Press,
                    state: crossterm::event::KeyEventState::empty(),
                };
                let action = {
                    if let Some(dialog) = self.provider_dialog.borrow_mut().as_mut() {
                        Some(dialog.handle_key(key_event))
                    } else {
                        None
                    }
                };
                if let Some(action) = action {
                    match action {
                        DialogAction::Continue => return Ok(()),
                        DialogAction::Save(config) => {
                            if let Err(e) = self.save_provider_config(&config) {
                                tracing::error!("Failed to save provider config: {}", e);
                            } else {
                                self.state.current_screen = match &self.state.current_screen {
                                    Screen::Home => Screen::Home,
                                    Screen::Session(id) => Screen::Session(id.clone()),
                                    _ => Screen::Home,
                                };
                                *self.provider_dialog.borrow_mut() = None;
                            }
                            return Ok(());
                        }
                        DialogAction::Cancel => {
                            self.state.current_screen = match &self.state.current_screen {
                                Screen::Home => Screen::Home,
                                Screen::Session(id) => Screen::Session(id.clone()),
                                _ => Screen::Home,
                            };
                            *self.provider_dialog.borrow_mut() = None;
                            return Ok(());
                        }
                    }
                }
            }
        }

        // Global shortcuts (work in all screens)
        match key {
            KeyCode::Char('q') => {
                self.should_quit = true;
                return Ok(());
            }
            KeyCode::Char('c') => {
                // Open provider config dialog
                self.open_provider_dialog();
                return Ok(());
            }
            _ => {}
        }

        // Screen-specific handling
        match &self.state.current_screen {
            Screen::Home => {
                match key {
                    KeyCode::Char('n') => {
                        let new_id = uuid::Uuid::new_v4().to_string();
                        self.state.current_screen = Screen::Session(new_id.clone());
                        *self.session_screen.borrow_mut() = Some(SessionScreen::new(new_id));
                    }
                    _ => {}
                }
            }
            Screen::Session(_) => {
                match key {
                    KeyCode::Esc => {
                        self.state.current_screen = Screen::Home;
                        *self.session_screen.borrow_mut() = None;
                    }
                    KeyCode::Up => {
                        if let Some(screen) = self.session_screen.borrow_mut().as_mut() {
                            screen.scroll_up();
                        }
                    }
                    KeyCode::Down => {
                        if let Some(screen) = self.session_screen.borrow_mut().as_mut() {
                            screen.scroll_down();
                        }
                    }
                    KeyCode::Enter => {
                        if let Some(screen) = self.session_screen.borrow_mut().as_mut() {
                            let input = screen.get_input().to_string();
                            if !input.trim().is_empty() {
                                // Add user message
                                screen.add_message(format!("You: {}", input));
                                
                                // Process with agent
                                let session_id = screen.session_id.clone();
                                let input_clone = input.clone();
                                let tx = self.response_tx.clone();
                                
                                // Spawn async task to process
                                let rt = tokio::runtime::Handle::try_current()
                                    .unwrap_or_else(|_| {
                                        tokio::runtime::Runtime::new().unwrap().handle().clone()
                                    });
                                
                                // Clone what we need for the async task
                                let agent_name = self.agent_manager.current_name().to_string();
                                let config = self.config.borrow().clone();
                                
                                // We need to pass sessions in a way that works across async boundaries
                                // For now, we'll create a new session in the async function
                                rt.spawn(async move {
                                    if let Err(e) = Self::process_message_async(
                                        &session_id,
                                        &input_clone,
                                        &agent_name,
                                        config,
                                        tx.clone(),
                                    ).await {
                                        tracing::error!("Failed to process message: {}", e);
                                        let _ = tx.send((session_id, format!("Error: {}", e)));
                                    }
                                });
                                
                                screen.clear_input();
                            }
                        }
                    }
                    KeyCode::Backspace => {
                        if let Some(screen) = self.session_screen.borrow_mut().as_mut() {
                            screen.delete_char();
                        }
                    }
                    KeyCode::Char(c) => {
                        if let Some(screen) = self.session_screen.borrow_mut().as_mut() {
                            screen.add_char(c);
                        }
                    }
                    _ => {}
                }
            }
            Screen::Dialog(_) => {
                // Dialog handling is done at the top of handle_key
            }
        }
        Ok(())
    }

    fn open_provider_dialog(&mut self) {
        let config = self.config.borrow();
        let provider_info = config.get_default_provider();
        let dialog = if let Some(info) = provider_info {
            ProviderDialog::with_initial_values(
                Some(info.provider_type),
                info.model,
                info.api_key,
                info.base_url,
            )
        } else {
            ProviderDialog::new()
        };
        *self.provider_dialog.borrow_mut() = Some(dialog);
        self.state.current_screen = Screen::Dialog(Box::new(DialogState::Provider));
    }

    fn save_provider_config(&self, config: &ProviderConfig) -> Result<()> {
        let mut app_config = self.config.borrow_mut();
        app_config.set_provider_config(
            "default",
            config.provider.clone(),
            config.api_key.clone(),
            config.base_url.clone(),
            config.model.clone(),
        )?;
        Ok(())
    }

    async fn process_message_async(
        session_id: &str,
        input: &str,
        agent_name: &str,
        config: AppConfig,
        tx: mpsc::UnboundedSender<(String, String)>,
    ) -> Result<()> {
        let session_dir = config.session_dir();
        let session_file = session_dir.join(session_id).join("session.json");
        let mut session = if session_file.exists() {
            Session::load(session_id, &session_dir)
                .await
                .unwrap_or_else(|_| {
                    Session::new(
                        session_id.to_string(),
                        "default".to_string(),
                        std::env::temp_dir().to_string_lossy().to_string(),
                    )
                })
        } else {
            Session::new(
                session_id.to_string(),
                "default".to_string(),
                std::env::temp_dir().to_string_lossy().to_string(),
            )
        };

        // Initialize provider and create adapter
        #[cfg(not(feature = "langchain"))]
        {
            let _ = tx.send((session_id.to_string(), "Error: langchain-rust feature not enabled. Rebuild with --features langchain".to_string()));
            return Err(anyhow::anyhow!("langchain-rust feature not enabled"));
        }
        
        #[cfg(feature = "langchain")]
        let provider_adapter = {
            let provider_info = config.get_default_provider();
            let provider_type = provider_info
                .as_ref()
                .map(|p| p.provider_type.clone())
                .unwrap_or_else(|| "openai".to_string());
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
                        let _ = tx.send((session_id.to_string(), "Error: No API key configured. Press 'C' to configure provider and API key.".to_string()));
                        return Err(anyhow::anyhow!("No API key configured"));
                    }
                    match opencode_provider::LangChainAdapter::from_openai(api_key, base_url, model) {
                        Ok(adapter) => Arc::new(adapter),
                        Err(e) => {
                            let _ = tx.send((session_id.to_string(), format!("Error initializing OpenAI provider: {}", e)));
                            return Err(anyhow::anyhow!("Failed to initialize provider: {}", e));
                        }
                    }
                }
                "ollama" => {
                    match opencode_provider::LangChainAdapter::from_ollama(base_url, model) {
                        Ok(adapter) => Arc::new(adapter),
                        Err(e) => {
                            let _ = tx.send((session_id.to_string(), format!("Error initializing Ollama provider: {}", e)));
                            return Err(anyhow::anyhow!("Failed to initialize provider: {}", e));
                        }
                    }
                }
                "qwen" => {
                    if api_key.trim().is_empty() {
                        let _ = tx.send((session_id.to_string(), "Error: No API key configured for Qwen. Press 'C' to configure.".to_string()));
                        return Err(anyhow::anyhow!("No API key configured"));
                    }
                    match opencode_provider::LangChainAdapter::from_qwen(api_key, base_url, model) {
                        Ok(adapter) => Arc::new(adapter),
                        Err(e) => {
                            let _ = tx.send((session_id.to_string(), format!("Error initializing Qwen provider: {}", e)));
                            return Err(anyhow::anyhow!("Failed to initialize provider: {}", e));
                        }
                    }
                }
                "anthropic" => {
                    if api_key.trim().is_empty() {
                        let _ = tx.send((session_id.to_string(), "Error: No API key configured. Press 'C' to configure.".to_string()));
                        return Err(anyhow::anyhow!("No API key configured"));
                    }
                    match opencode_provider::LangChainAdapter::from_anthropic(api_key) {
                        Ok(adapter) => Arc::new(adapter),
                        Err(e) => {
                            let _ = tx.send((session_id.to_string(), format!("Error initializing Anthropic provider: {}", e)));
                            return Err(anyhow::anyhow!("Failed to initialize provider: {}", e));
                        }
                    }
                }
                _ => {
                    let _ = tx.send((session_id.to_string(), format!("Unsupported provider type: {}", provider_type)));
                    return Err(anyhow::anyhow!("Unsupported provider type: {}", provider_type));
                }
            };

            opencode_provider::ProviderAdapter::new(provider)
        };
        
        #[cfg(feature = "langchain")]
        {
            // Get tools (empty for now, can be populated later)
            let tools: Vec<Arc<dyn opencode_core::tool::Tool>> = vec![];
            
            // Create agent manager for processing
            let mut agent_manager = AgentManager::new();
            if let Err(e) = agent_manager.switch(agent_name) {
                let _ = tx.send((session_id.to_string(), format!("Error switching agent: {}", e)));
                return Err(anyhow::anyhow!("Failed to switch agent: {}", e));
            }
            
            // Process with agent
            let ctx = Context {
                session_id: session_id.to_string(),
                message_id: uuid::Uuid::new_v4().to_string(),
                agent: agent_name.to_string(),
            };
            
            match agent_manager.process(&ctx, input, &mut session, &provider_adapter, &tools).await {
                Ok(_) => {
                    if let Err(e) = session.save(&session_dir).await {
                        tracing::warn!("Failed to save session: {}", e);
                    }
                    if let Some(last_msg) = session.messages.back() {
                        if matches!(last_msg.role, opencode_core::session::MessageRole::Assistant) {
                            let _ = tx.send((session_id.to_string(), last_msg.content.clone()));
                        }
                    }
                }
                Err(e) => {
                    let _ = tx.send((session_id.to_string(), format!("Error: {}", e)));
                    return Err(anyhow::anyhow!("Agent processing failed: {}", e));
                }
            }
        }

        Ok(())
    }
}
