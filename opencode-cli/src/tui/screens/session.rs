use ratatui::prelude::*;
use ratatui::widgets::*;
use crate::tui::components::{header, message_view::MessageView, sidebar, spinner::Spinner};
use crate::tui::theme::Theme;
use opencode_core::session::{Role, Session};

pub struct SessionScreen {
    pub session_id: String,
    pub message_view: MessageView,
    pub input: String,
    pub is_processing: bool,
    pub spinner: Spinner,
}

impl SessionScreen {
    pub fn new(session_id: String) -> Self {
        Self {
            session_id,
            message_view: MessageView::new(),
            input: String::new(),
            is_processing: false,
            spinner: Spinner::new(),
        }
    }

    pub fn load_messages(&mut self, session: &Session) {
        for msg in &session.messages {
            let prefix = match msg.role {
                Role::User => "You: ",
                Role::Assistant => "Assistant: ",
                Role::System => "System: ",
                Role::Tool => "Tool: ",
            };
            self.message_view.add_message(format!("{}{}", prefix, msg.content));
        }
    }

    pub fn add_message(&mut self, message: String) {
        self.message_view.add_message(message);
    }

    pub fn scroll_up(&mut self) {
        self.message_view.scroll_up();
    }

    pub fn scroll_down(&mut self) {
        self.message_view.scroll_down();
    }

    pub fn add_char(&mut self, c: char) {
        self.input.push(c);
    }

    pub fn delete_char(&mut self) {
        self.input.pop();
    }

    pub fn clear_input(&mut self) {
        self.input.clear();
    }

    pub fn get_input(&self) -> &str {
        &self.input
    }

    pub fn set_processing(&mut self, processing: bool) {
        self.is_processing = processing;
    }

    pub fn render(&mut self, f: &mut Frame, area: Rect, theme: &Theme) {
        let chunks = Layout::default()
            .direction(Direction::Horizontal)
            .constraints([Constraint::Percentage(20), Constraint::Percentage(80)])
            .split(area);

        let right_chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Length(3),
                Constraint::Min(1),
                Constraint::Length(3),
            ])
            .split(chunks[1]);

        header::render(f, right_chunks[0], &format!("Session: {}", self.session_id), theme);
        self.message_view.render(f, right_chunks[1], theme);
        
        // Input area
        let input_title = if self.is_processing {
            format!("⏳ Processing... {}", self.spinner.get_frame())
        } else {
            "⌨️ Input (Enter to send, Esc to Home)".to_string()
        };
        
        let input_block = Block::default()
            .title(vec![
                Span::styled(input_title, if self.is_processing { 
                    theme.warning_style() 
                } else { 
                    theme.secondary_style() 
                }),
            ])
            .borders(Borders::TOP | Borders::LEFT | Borders::RIGHT)
            .border_style(theme.border_style())
            .style(theme.panel_style());
        
        let input_paragraph = Paragraph::new(self.input.as_str())
            .block(input_block)
            .style(Style::default().fg(if self.is_processing { 
                theme.warning 
            } else { 
                theme.primary 
            }));
        
        f.render_widget(input_paragraph, right_chunks[2]);
        sidebar::render(f, chunks[0], theme);
    }
}