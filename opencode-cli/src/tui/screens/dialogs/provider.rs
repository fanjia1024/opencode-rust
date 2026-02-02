use ratatui::prelude::*;
use ratatui::widgets::*;
use crossterm::event::{KeyEvent, KeyCode};

#[derive(Clone, Copy, PartialEq)]
enum InputMode {
    SelectProvider,
    InputApiKey,
    InputBaseUrl,
}

pub struct ProviderDialog {
    providers: Vec<String>,
    selected: usize,
    api_key: String,
    base_url: String,
    input_mode: InputMode,
    error_message: Option<String>,
}

impl ProviderDialog {
    pub fn new() -> Self {
        Self {
            providers: vec![
                "openai".to_string(),
                "anthropic".to_string(),
            ],
            selected: 0,
            api_key: String::new(),
            base_url: String::new(),
            input_mode: InputMode::SelectProvider,
            error_message: None,
        }
    }

    pub fn with_initial_values(provider: Option<String>, api_key: Option<String>, base_url: Option<String>) -> Self {
        let mut dialog = Self::new();
        if let Some(p) = provider {
            if let Some(idx) = dialog.providers.iter().position(|x| x == &p) {
                dialog.selected = idx;
            }
        }
        if let Some(key) = api_key {
            dialog.api_key = key;
        }
        if let Some(url) = base_url {
            dialog.base_url = url;
        }
        dialog
    }

    fn masked_api_key(&self) -> String {
        if self.api_key.is_empty() {
            String::new()
        } else {
            "•".repeat(self.api_key.len().min(50))
        }
    }

    pub fn render(&self, f: &mut Frame, area: Rect) {
        let popup_area = centered_rect(70, 70, area);
        
        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Length(3),
                Constraint::Min(5),
                Constraint::Length(3),
                Constraint::Length(3),
                Constraint::Length(3),
            ])
            .split(popup_area);

        let title = if self.error_message.is_some() {
            "Provider Configuration (Error)"
        } else {
            "Provider Configuration"
        };
        let block = Block::default()
            .title(title)
            .borders(Borders::ALL);

        let provider_style = if self.input_mode == InputMode::SelectProvider {
            Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD)
        } else {
            Style::default()
        };

        let items: Vec<ListItem> = self
            .providers
            .iter()
            .enumerate()
            .map(|(i, provider)| {
                let style = if i == self.selected {
                    provider_style
                } else {
                    Style::default()
                };
                let prefix = if i == self.selected && self.input_mode == InputMode::SelectProvider {
                    "> "
                } else {
                    "  "
                };
                ListItem::new(format!("{}{}", prefix, provider.as_str())).style(style)
            })
            .collect();

        let list = List::new(items)
            .block(block);

        f.render_widget(list, chunks[1]);

        let api_key_style = if self.input_mode == InputMode::InputApiKey {
            Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD)
        } else {
            Style::default()
        };
        let api_key_title = if self.input_mode == InputMode::InputApiKey {
            "> API Key (Press Tab to switch fields)"
        } else {
            "API Key"
        };
        let api_key_block = Block::default()
            .title(api_key_title)
            .borders(Borders::ALL)
            .border_style(api_key_style);
        let api_key_display = if self.input_mode == InputMode::InputApiKey {
            self.api_key.as_str()
        } else {
            &self.masked_api_key()
        };
        let api_key_text = Paragraph::new(api_key_display)
            .block(api_key_block)
            .style(api_key_style);
        f.render_widget(api_key_text, chunks[2]);

        let base_url_style = if self.input_mode == InputMode::InputBaseUrl {
            Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD)
        } else {
            Style::default()
        };
        let base_url_title = if self.input_mode == InputMode::InputBaseUrl {
            "> Base URL (optional, Press Tab to switch fields)"
        } else {
            "Base URL (optional)"
        };
        let base_url_block = Block::default()
            .title(base_url_title)
            .borders(Borders::ALL)
            .border_style(base_url_style);
        let base_url_text = Paragraph::new(self.base_url.as_str())
            .block(base_url_block)
            .style(base_url_style);
        f.render_widget(base_url_text, chunks[3]);

        if let Some(error) = &self.error_message {
            let error_text = Paragraph::new(error.as_str())
                .style(Style::default().fg(Color::Red))
                .block(Block::default().borders(Borders::ALL).title("Error"));
            f.render_widget(error_text, chunks[4]);
        } else {
            let help_text = Paragraph::new("Tab: Switch field | Enter: Save | Esc: Cancel")
                .style(Style::default().fg(Color::DarkGray))
                .block(Block::default().borders(Borders::ALL).title("Help"));
            f.render_widget(help_text, chunks[4]);
        }
    }

    pub fn handle_key(&mut self, key: KeyEvent) -> DialogAction {
        self.error_message = None;

        match key.code {
            KeyCode::Tab => {
                self.input_mode = match self.input_mode {
                    InputMode::SelectProvider => InputMode::InputApiKey,
                    InputMode::InputApiKey => InputMode::InputBaseUrl,
                    InputMode::InputBaseUrl => InputMode::SelectProvider,
                };
                DialogAction::Continue
            }
            KeyCode::Up => {
                if self.input_mode == InputMode::SelectProvider {
                    if self.selected > 0 {
                        self.selected -= 1;
                    }
                }
                DialogAction::Continue
            }
            KeyCode::Down => {
                if self.input_mode == InputMode::SelectProvider {
                    if self.selected < self.providers.len() - 1 {
                        self.selected += 1;
                    }
                }
                DialogAction::Continue
            }
            KeyCode::Enter => {
                if self.api_key.trim().is_empty() {
                    self.error_message = Some("API Key cannot be empty".to_string());
                    DialogAction::Continue
                } else {
                    DialogAction::Save(ProviderConfig {
                        provider: self.providers[self.selected].clone(),
                        api_key: self.api_key.clone(),
                        base_url: if self.base_url.trim().is_empty() { None } else { Some(self.base_url.trim().to_string()) },
                    })
                }
            }
            KeyCode::Esc => DialogAction::Cancel,
            KeyCode::Backspace => {
                match self.input_mode {
                    InputMode::InputApiKey => {
                        self.api_key.pop();
                    }
                    InputMode::InputBaseUrl => {
                        self.base_url.pop();
                    }
                    _ => {}
                }
                DialogAction::Continue
            }
            KeyCode::Char(c) => {
                match self.input_mode {
                    InputMode::InputApiKey => {
                        self.api_key.push(c);
                    }
                    InputMode::InputBaseUrl => {
                        self.base_url.push(c);
                    }
                    _ => {}
                }
                DialogAction::Continue
            }
            _ => DialogAction::Continue,
        }
    }
}

pub enum DialogAction {
    Continue,
    Save(ProviderConfig),
    Cancel,
}

pub struct ProviderConfig {
    pub provider: String,
    pub api_key: String,
    pub base_url: Option<String>,
}

fn centered_rect(percent_x: u16, percent_y: u16, r: Rect) -> Rect {
    let popup_layout = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Percentage((100 - percent_y) / 2),
            Constraint::Percentage(percent_y),
            Constraint::Percentage((100 - percent_y) / 2),
        ])
        .split(r);

    Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Percentage((100 - percent_x) / 2),
            Constraint::Percentage(percent_x),
            Constraint::Percentage((100 - percent_x) / 2),
        ])
        .split(popup_layout[1])[1]
}

impl Default for ProviderDialog {
    fn default() -> Self {
        Self::new()
    }
}
