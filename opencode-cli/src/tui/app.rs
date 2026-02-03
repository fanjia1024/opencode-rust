use crate::commands::init;
use crate::config::AppConfig;
use crate::session_store;
use crate::tui::screens::dialogs::agent::{AgentDialog, AgentDialogAction};
use crate::tui::screens::dialogs::command::CommandDialog;
use crate::tui::screens::dialogs::help::HelpDialog;
use crate::tui::screens::dialogs::provider::{DialogAction, ProviderConfig, ProviderDialog};
use crate::tui::screens::dialogs::providers_list::{ProvidersListAction, ProvidersListDialog};
use crate::tui::screens::home::SessionInfo;
use crate::tui::screens::{home::HomeScreen, session::SessionScreen};
use crate::tui::state::{AppState, DialogState, Screen};
use crate::tui::sync::{SessionListItem, StateSync};
use crate::tui::theme::Theme;
use anyhow::Result;
use crossterm::event::{self, Event, KeyCode, KeyEventKind};
use crossterm::execute;
use crossterm::terminal::{
    disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen,
};
use opencode_core::agent::Context;
use opencode_core::session::Session;
use opencode_core::AgentManager;
use ratatui::prelude::*;
use ratatui::widgets;
use std::cell::RefCell;
use std::collections::HashMap;
use std::env;
use std::io;
use std::path::{Path, PathBuf};
use std::sync::Arc;
use tokio::sync::mpsc;

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
    agent_dialog: RefCell<Option<AgentDialog>>,
    providers_list_dialog: RefCell<Option<ProvidersListDialog>>,
    help_dialog: RefCell<Option<HelpDialog>>,
    command_dialog: RefCell<Option<CommandDialog>>,
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
            agent_dialog: RefCell::new(None),
            providers_list_dialog: RefCell::new(None),
            help_dialog: RefCell::new(None),
            command_dialog: RefCell::new(None),
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
                if let Ok(session) = session_store::load_session(&session_file) {
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
                    tracing::debug!(session_id = %session_id, response_len = response.len(), "received response, updating UI");
                    drop(rx); // Release borrow before borrowing session_screen
                    if let Some(screen) = self.session_screen.borrow_mut().as_mut() {
                        if screen.session_id == session_id {
                            screen.add_message(format!("Assistant: {}", response));
                            screen.set_processing(false); // Clear processing state
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
        let theme = Theme::default();

        match &self.state.current_screen {
            Screen::Home => {
                self.home_screen.render(f, f.size(), &theme);
            }
            Screen::Session(session_id) => {
                let current_agent = self.agent_manager.current_name();
                let current_model = self
                    .config
                    .borrow()
                    .get_default_provider()
                    .and_then(|p| p.model.clone());
                let mut session_screen = self.session_screen.borrow_mut();
                if let Some(screen) = session_screen.as_mut() {
                    screen.render(
                        f,
                        f.size(),
                        &theme,
                        Some(current_agent),
                        current_model.as_deref(),
                    );
                } else {
                    let mut new_screen = SessionScreen::new(session_id.clone());
                    new_screen.render(
                        f,
                        f.size(),
                        &theme,
                        Some(current_agent),
                        current_model.as_deref(),
                    );
                }
            }
            Screen::Dialog(dialog_state) => {
                let theme = Theme::default();
                match dialog_state.as_ref() {
                    DialogState::Provider => {
                        if let Some(dialog) = self.provider_dialog.borrow().as_ref() {
                            dialog.render(f, f.size());
                        }
                    }
                    DialogState::Agent => {
                        if let Some(dialog) = self.agent_dialog.borrow().as_ref() {
                            dialog.render(f, f.size(), &theme);
                        }
                    }
                    DialogState::ProvidersList => {
                        if let Some(dialog) = self.providers_list_dialog.borrow().as_ref() {
                            dialog.render(f, f.size(), &theme);
                        }
                    }
                    DialogState::Help => {
                        if let Some(dialog) = self.help_dialog.borrow().as_ref() {
                            dialog.render(f, f.size());
                        }
                    }
                    DialogState::Command(_) => {
                        if let Some(dialog) = self.command_dialog.borrow().as_ref() {
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
        use crossterm::event::KeyEvent;
        let key_event = KeyEvent {
            code: key,
            modifiers: crossterm::event::KeyModifiers::empty(),
            kind: KeyEventKind::Press,
            state: crossterm::event::KeyEventState::empty(),
        };

        fn restore_screen(state: &mut Screen) {
            *state = match state {
                Screen::Home => Screen::Home,
                Screen::Session(id) => Screen::Session(id.clone()),
                _ => Screen::Home,
            };
        }

        if let Screen::Dialog(dialog_state) = &self.state.current_screen {
            if let DialogState::Agent = dialog_state.as_ref() {
                let action = self
                    .agent_dialog
                    .borrow_mut()
                    .as_mut()
                    .map(|d| d.handle_key(key_event));
                if let Some(action) = action {
                    match action {
                        AgentDialogAction::Switch(name) => {
                            if let Err(e) = self.agent_manager.switch(&name) {
                                tracing::error!("Failed to switch agent: {}", e);
                            }
                            restore_screen(&mut self.state.current_screen);
                            *self.agent_dialog.borrow_mut() = None;
                        }
                        AgentDialogAction::Cancel => {
                            restore_screen(&mut self.state.current_screen);
                            *self.agent_dialog.borrow_mut() = None;
                        }
                        AgentDialogAction::Continue => {}
                    }
                }
                return Ok(());
            }

            if let DialogState::ProvidersList = dialog_state.as_ref() {
                let (action, edit_provider_id) = {
                    if let Some(dialog) = self.providers_list_dialog.borrow_mut().as_mut() {
                        let action = dialog.handle_key(key_event);
                        let edit_id = match &action {
                            ProvidersListAction::Edit(i) => {
                                dialog.items.get(*i).map(|p| p.id.clone())
                            }
                            _ => None,
                        };
                        (action, edit_id)
                    } else {
                        (ProvidersListAction::Continue, None)
                    }
                };
                match action {
                    ProvidersListAction::SetDefault(i) => {
                        let items = self.config.borrow().list_providers();
                        if let Some(item) = items.get(i) {
                            let mut config = self.config.borrow_mut();
                            let _ = config.set_default_provider_id(&item.id);
                        }
                        if let Some(dialog) = self.providers_list_dialog.borrow_mut().as_mut() {
                            let items = self.config.borrow().list_providers();
                            *dialog = ProvidersListDialog::new(items);
                        }
                    }
                    ProvidersListAction::Edit(_) => {
                        if let Some(id) = edit_provider_id {
                            self.providers_list_dialog.borrow_mut().take();
                            restore_screen(&mut self.state.current_screen);
                            self.open_provider_dialog(Some(&id));
                        }
                    }
                    ProvidersListAction::Cancel => {
                        restore_screen(&mut self.state.current_screen);
                        *self.providers_list_dialog.borrow_mut() = None;
                    }
                    ProvidersListAction::Continue => {}
                }
                return Ok(());
            }

            if let DialogState::Help = dialog_state.as_ref() {
                if key == KeyCode::Esc {
                    restore_screen(&mut self.state.current_screen);
                    *self.help_dialog.borrow_mut() = None;
                }
                return Ok(());
            }

            if let DialogState::Command(session_id) = dialog_state.as_ref() {
                let session_id = session_id.clone();
                tklog::info!("command_dialog key", format!("{:?}", key_event.code));
                let action = self
                    .command_dialog
                    .borrow_mut()
                    .as_mut()
                    .and_then(|d| d.handle_key(key_event));
                if let Some((id, label)) = action {
                    tklog::info!("command_dialog action", "id", &id, "label", &label);
                    if id == "init" {
                        tklog::info!("command id is init, running init_agents_md", &id);
                        let project_root: PathBuf =
                            env::current_dir().unwrap_or_else(|_| PathBuf::from("."));
                        tklog::info!("calling init_agents_md", project_root.display());
                        if let Some(screen) = self.session_screen.borrow_mut().as_mut() {
                            screen.set_processing(true);
                        }
                        let tx = self.response_tx.clone();
                        let session_id_for_task = session_id.clone();
                        let rt = tokio::runtime::Handle::try_current().unwrap_or_else(|_| {
                            tokio::runtime::Runtime::new().unwrap().handle().clone()
                        });
                        rt.spawn(async move {
                            let r = init::init_agents_md(&project_root, false).await;
                            let msg = match r {
                                Ok(true) => "AGENTS.md created or updated.".to_string(),
                                Ok(false) => "AGENTS.md already exists.".to_string(),
                                Err(e) => format!("Error: {}", e),
                            };
                            let _ = tx.send((session_id_for_task, msg));
                        });
                    } else {
                        if let Some(screen) = self.session_screen.borrow_mut().as_mut() {
                            screen.input.push_str(&label);
                            screen.input.push(' ');
                        }
                    }
                    self.state.current_screen = Screen::Session(session_id);
                    *self.command_dialog.borrow_mut() = None;
                    return Ok(());
                }
                if key == KeyCode::Esc {
                    self.state.current_screen = Screen::Session(session_id);
                    *self.command_dialog.borrow_mut() = None;
                    return Ok(());
                }
                return Ok(());
            }

            if let DialogState::Provider = dialog_state.as_ref() {
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
                                restore_screen(&mut self.state.current_screen);
                                *self.provider_dialog.borrow_mut() = None;
                            }
                            return Ok(());
                        }
                        DialogAction::Cancel => {
                            restore_screen(&mut self.state.current_screen);
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
                self.open_provider_dialog(None);
                return Ok(());
            }
            KeyCode::Char('a') => {
                self.open_agent_dialog();
                return Ok(());
            }
            KeyCode::Char('p') => {
                self.open_providers_list_dialog();
                return Ok(());
            }
            KeyCode::Char('h') | KeyCode::Char('H') | KeyCode::Char('?') => {
                self.open_help_dialog();
                return Ok(());
            }
            _ => {}
        }

        // Screen-specific handling
        match &self.state.current_screen {
            Screen::Home => match key {
                KeyCode::Char('n') => {
                    let new_id = uuid::Uuid::new_v4().to_string();
                    self.state.current_screen = Screen::Session(new_id.clone());
                    *self.session_screen.borrow_mut() = Some(SessionScreen::new(new_id));
                }
                _ => {}
            },
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

                                // Set processing state
                                screen.set_processing(true);

                                // Process with agent
                                let session_id = screen.session_id.clone();
                                let input_clone = input.clone();
                                let tx = self.response_tx.clone();

                                // Spawn async task to process
                                let rt =
                                    tokio::runtime::Handle::try_current().unwrap_or_else(|_| {
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
                                    )
                                    .await
                                    {
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
                    KeyCode::Char('/') => {
                        self.open_command_dialog();
                        return Ok(());
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

    fn open_provider_dialog(&mut self, edit_id: Option<&str>) {
        let config = self.config.borrow();
        let dialog = if let Some(id) = edit_id {
            config.get_provider_config(Some(id)).map(|info| {
                ProviderDialog::with_initial_values_for_edit(
                    id.to_string(),
                    Some(info.provider_type),
                    info.model,
                    info.api_key,
                    info.base_url,
                )
            })
        } else {
            None
        };
        let dialog = dialog
            .or_else(|| {
                config.get_default_provider().map(|info| {
                    ProviderDialog::with_initial_values(
                        Some(info.provider_type),
                        info.model,
                        info.api_key,
                        info.base_url,
                    )
                })
            })
            .unwrap_or_else(ProviderDialog::new);
        *self.provider_dialog.borrow_mut() = Some(dialog);
        self.state.current_screen = Screen::Dialog(Box::new(DialogState::Provider));
    }

    fn open_agent_dialog(&mut self) {
        let mut agents = self.agent_manager.list();
        agents.sort();
        let current = self.agent_manager.current_name().to_string();
        *self.agent_dialog.borrow_mut() = Some(AgentDialog::new(agents, current));
        self.state.current_screen = Screen::Dialog(Box::new(DialogState::Agent));
    }

    fn open_providers_list_dialog(&mut self) {
        let items = self.config.borrow().list_providers();
        *self.providers_list_dialog.borrow_mut() = Some(ProvidersListDialog::new(items));
        self.state.current_screen = Screen::Dialog(Box::new(DialogState::ProvidersList));
    }

    fn open_help_dialog(&mut self) {
        *self.help_dialog.borrow_mut() = Some(HelpDialog::new());
        self.state.current_screen = Screen::Dialog(Box::new(DialogState::Help));
    }

    fn open_command_dialog(&mut self) {
        let session_id = match &self.state.current_screen {
            Screen::Session(id) => id.clone(),
            _ => return,
        };
        tklog::info!("command_dialog opened", &session_id);
        *self.command_dialog.borrow_mut() = Some(CommandDialog::new());
        self.state.current_screen = Screen::Dialog(Box::new(DialogState::Command(session_id)));
    }

    fn save_provider_config(&self, config: &ProviderConfig) -> Result<()> {
        let id = config.provider_id.as_deref().unwrap_or("default");
        let mut app_config = self.config.borrow_mut();
        app_config.set_provider_config(
            id,
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
        tracing::info!(session_id = %session_id, input_len = input.len(), "process_message_async started");
        let session_dir = config.session_dir();
        let session_file = session_dir.join(session_id).join("session.json");
        let mut session = if session_file.exists() {
            session_store::load_session(&session_file).unwrap_or_else(|_| Session::new())
        } else {
            let s = Session::new();
            let path = session_dir.join(s.id.to_string()).join("session.json");
            let _ = session_store::save_session(&path, &s);
            s
        };
        let session_id_str = session.id.to_string();

        // Initialize provider and create adapter
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
                        let _ = tx.send((session_id_str.clone(), "Error: No API key configured. Press 'C' to configure provider and API key.".to_string()));
                        return Err(anyhow::anyhow!("No API key configured"));
                    }
                    match opencode_provider::LangChainAdapter::from_openai(api_key, base_url, model)
                    {
                        Ok(adapter) => Arc::new(adapter),
                        Err(e) => {
                            tracing::error!(error = %e, "Failed to initialize OpenAI provider");
                            let _ = tx.send((
                                session_id_str.clone(),
                                format!("Error initializing OpenAI provider: {}", e),
                            ));
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
                                session_id_str.clone(),
                                format!("Error initializing Ollama provider: {}", e),
                            ));
                            return Err(anyhow::anyhow!("Failed to initialize provider: {}", e));
                        }
                    }
                }
                "qwen" => {
                    if api_key.trim().is_empty() {
                        tracing::error!("No API key configured for Qwen");
                        let _ = tx.send((
                            session_id_str.clone(),
                            "Error: No API key configured for Qwen. Press 'C' to configure."
                                .to_string(),
                        ));
                        return Err(anyhow::anyhow!("No API key configured"));
                    }
                    match opencode_provider::LangChainAdapter::from_qwen(api_key, base_url, model) {
                        Ok(adapter) => Arc::new(adapter),
                        Err(e) => {
                            tracing::error!(error = %e, "Failed to initialize Qwen provider");
                            let _ = tx.send((
                                session_id_str.clone(),
                                format!("Error initializing Qwen provider: {}", e),
                            ));
                            return Err(anyhow::anyhow!("Failed to initialize provider: {}", e));
                        }
                    }
                }
                "anthropic" => {
                    if api_key.trim().is_empty() {
                        tracing::error!("No API key configured for Anthropic");
                        let _ = tx.send((
                            session_id_str.clone(),
                            "Error: No API key configured. Press 'C' to configure.".to_string(),
                        ));
                        return Err(anyhow::anyhow!("No API key configured"));
                    }
                    match opencode_provider::LangChainAdapter::from_anthropic(api_key) {
                        Ok(adapter) => Arc::new(adapter),
                        Err(e) => {
                            tracing::error!(error = %e, "Failed to initialize Anthropic provider");
                            let _ = tx.send((
                                session_id_str.clone(),
                                format!("Error initializing Anthropic provider: {}", e),
                            ));
                            return Err(anyhow::anyhow!("Failed to initialize provider: {}", e));
                        }
                    }
                }
                _ => {
                    tracing::error!(provider_type = %provider_type, "Unsupported provider type");
                    let _ = tx.send((
                        session_id_str.clone(),
                        format!("Unsupported provider type: {}", provider_type),
                    ));
                    return Err(anyhow::anyhow!(
                        "Unsupported provider type: {}",
                        provider_type
                    ));
                }
            };

            opencode_provider::ProviderAdapter::new(provider)
        };

        {
            // Initialize tools
            use opencode_tools::registry::ToolRegistry;
            use opencode_tools::tools;
            let mut tool_registry = ToolRegistry::new();
            tools::register_all_tools(&mut tool_registry);

            // Convert registry to vector of tools
            let tools: Vec<Arc<dyn opencode_core::tool::Tool>> = tool_registry
                .list()
                .iter()
                .filter_map(|id| tool_registry.get(id))
                .cloned()
                .collect();

            // Create agent manager for processing
            let mut agent_manager = AgentManager::new();
            if let Err(e) = agent_manager.switch(agent_name) {
                tracing::error!(error = %e, "Failed to switch agent");
                let _ = tx.send((
                    session_id_str.clone(),
                    format!("Error switching agent: {}", e),
                ));
                return Err(anyhow::anyhow!("Failed to switch agent: {}", e));
            }

            tracing::debug!("calling agent_manager.process");
            // Process with agent
            let ctx = Context {
                session_id: session_id_str.clone(),
                message_id: uuid::Uuid::new_v4().to_string(),
                agent: agent_name.to_string(),
            };

            match agent_manager
                .process(&ctx, input, &mut session, &provider_adapter, &tools)
                .await
            {
                Ok(_) => {
                    let save_path = session_dir.join(&session_id_str).join("session.json");
                    if let Err(e) = session_store::save_session(&save_path, &session) {
                        tracing::warn!("Failed to save session: {}", e);
                    }
                    if let Some(last_msg) = session.messages.last() {
                        if matches!(last_msg.role, opencode_core::session::Role::Assistant) {
                            tracing::info!(session_id = %session_id_str, response_len = last_msg.content.len(), "response sent to UI");
                            let _ = tx.send((session_id_str.clone(), last_msg.content.clone()));
                        }
                    }
                }
                Err(e) => {
                    tracing::error!(error = %e, "Agent processing failed");
                    let _ = tx.send((session_id_str.clone(), format!("Error: {}", e)));
                    return Err(anyhow::anyhow!("Agent processing failed: {}", e));
                }
            }
        }

        Ok(())
    }
}
