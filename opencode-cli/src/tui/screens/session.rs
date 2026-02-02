use ratatui::prelude::*;
use ratatui::widgets::*;
use crate::tui::components::{header, footer, message_view::MessageView, sidebar};
use opencode_core::session::{Session, MessageRole};

pub struct SessionScreen {
    pub session_id: String,
    pub message_view: MessageView,
    pub input: String,
}

impl SessionScreen {
    pub fn new(session_id: String) -> Self {
        Self {
            session_id,
            message_view: MessageView::new(),
            input: String::new(),
        }
    }

    pub fn load_messages(&mut self, session: &Session) {
        for msg in &session.messages {
            let prefix = match msg.role {
                MessageRole::User => "You: ",
                MessageRole::Assistant => "Assistant: ",
                MessageRole::System => "System: ",
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

    pub fn render(&mut self, f: &mut Frame, area: Rect) {
        let chunks = Layout::default()
            .direction(Direction::Horizontal)
            .constraints([Constraint::Percentage(20), Constraint::Percentage(80)])
            .split(area);

        let right_chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Length(3),
                Constraint::Length(2),
                Constraint::Min(1),
                Constraint::Length(3),
                Constraint::Length(3),
            ])
            .split(chunks[1]);

        header::render(f, right_chunks[0], &format!("Session: {}", self.session_id));
        let guidance = Paragraph::new("Welcome. Press C to configure provider. Type below and press Enter to send.")
            .style(Style::default().fg(Color::DarkGray));
        f.render_widget(guidance, right_chunks[1]);
        self.message_view.render(f, right_chunks[2]);
        
        // Input area
        let input_block = Block::default()
            .title("Input (Press Enter to send)")
            .borders(Borders::ALL);
        let input_paragraph = Paragraph::new(self.input.as_str())
            .block(input_block)
            .style(Style::default().fg(Color::Yellow));
        f.render_widget(input_paragraph, right_chunks[3]);

        footer::render(f, right_chunks[4], "Press 'q' to quit, 'Esc' to go back, 'Enter' to send");
        
        sidebar::render(f, chunks[0]);
    }
}
