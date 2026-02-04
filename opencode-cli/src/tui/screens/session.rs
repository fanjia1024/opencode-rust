use crate::tui::app::LogEntry;
use crate::tui::components::{
    header,
    log_panel::LogPanel,
    message_view::MessageView,
    sidebar,
    spinner::Spinner,
    virtual_scroll::VirtualScroll,
};
use crate::tui::theme::Theme;
use opencode_core::session::{Role, Session};
use ratatui::prelude::*;
use ratatui::widgets::*;
use std::env;

pub struct SessionScreen {
    pub session_id: String,
    pub message_view: MessageView,
    pub log_entries: Vec<LogEntry>,
    pub log_scroll: VirtualScroll,
    pub input: String,
    pub is_processing: bool,
    pub spinner: Spinner,
}

impl SessionScreen {
    pub fn new(session_id: String) -> Self {
        Self {
            session_id,
            message_view: MessageView::new(),
            log_entries: Vec::new(),
            log_scroll: VirtualScroll::new(10),
            input: String::new(),
            is_processing: false,
            spinner: Spinner::new(),
        }
    }

    pub fn append_log_entry(&mut self, entry: LogEntry) {
        self.log_entries.push(entry);
        self.log_scroll.scroll_to_bottom();
    }

    pub fn scroll_log_up(&mut self) {
        self.log_scroll.scroll_up(1);
    }

    pub fn scroll_log_down(&mut self) {
        self.log_scroll.scroll_down(1);
    }

    pub fn load_messages(&mut self, session: &Session) {
        for msg in &session.messages {
            let prefix = match msg.role {
                Role::User => "You: ",
                Role::Assistant => "Assistant: ",
                Role::System => "System: ",
                Role::Tool => "Tool: ",
            };
            self.message_view
                .add_message(format!("{}{}", prefix, msg.content));
        }
    }

    pub fn add_message(&mut self, message: String) {
        self.message_view.add_message(message);
    }

    pub fn start_streaming_assistant(&mut self) {
        self.message_view.start_streaming_assistant();
    }

    pub fn append_streaming_chunk(&mut self, chunk: String) {
        self.message_view.append_streaming_chunk(chunk);
    }

    pub fn finish_streaming_assistant(&mut self) {
        self.message_view.finish_streaming_assistant();
    }

    pub fn is_streaming(&self) -> bool {
        self.message_view.is_streaming()
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

    pub fn render(
        &mut self,
        f: &mut Frame,
        area: Rect,
        theme: &Theme,
        current_agent: Option<&str>,
        current_model: Option<&str>,
    ) {
        let chunks = Layout::default()
            .direction(Direction::Horizontal)
            .constraints([Constraint::Percentage(20), Constraint::Percentage(80)])
            .split(area);

        let right_chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Length(3),
                Constraint::Min(10),
                Constraint::Length(10),
                Constraint::Length(3),
                Constraint::Length(1),
                Constraint::Length(1),
            ])
            .split(chunks[1]);

        header::render(
            f,
            right_chunks[0],
            &format!("Session: {}", self.session_id),
            theme,
        );
        self.message_view.render(f, right_chunks[1], theme);
        LogPanel::render(
            f,
            right_chunks[2],
            &self.log_entries,
            &mut self.log_scroll,
            theme,
        );

        // Input area
        let input_title = if self.is_processing {
            format!("⏳ Processing... {}", self.spinner.get_frame())
        } else {
            "⌨️ Input (Enter to send, Esc to Home)".to_string()
        };

        let input_block = Block::default()
            .title(vec![Span::styled(
                input_title,
                if self.is_processing {
                    theme.warning_style()
                } else {
                    theme.secondary_style()
                },
            )])
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

        f.render_widget(input_paragraph, right_chunks[3]);

        // Keybind hint row (with current agent and model)
        let agent_str = current_agent.unwrap_or("—");
        let model_str = current_model.unwrap_or("—");
        let keybind_text = format!(
            "Enter send  Esc Home  Up/Down msg  Alt+Up/Down log  Ctrl+Q/C/A/P/H  |  Top: chat, Bottom: log  |  Agent: {}  Model: {}",
            agent_str, model_str
        );
        let keybind_line = Paragraph::new(keybind_text.as_str())
            .style(Style::default().fg(theme.secondary).bg(theme.panel_bg));
        f.render_widget(keybind_line, right_chunks[4]);

        // Footer: cwd left, version right
        let cwd = env::current_dir()
            .ok()
            .and_then(|p| p.into_os_string().into_string().ok())
            .unwrap_or_else(|| ".".to_string());
        let cwd_display = if let Ok(home) = env::var("HOME") {
            cwd.replace(&home, "~")
        } else {
            cwd
        };
        let version = env!("CARGO_PKG_VERSION");
        let footer_area = right_chunks[5];
        let footer_chunks = Layout::default()
            .direction(Direction::Horizontal)
            .constraints([Constraint::Percentage(50), Constraint::Percentage(50)])
            .split(footer_area);
        let footer_left = Paragraph::new(cwd_display.as_str())
            .style(Style::default().fg(theme.secondary).bg(theme.panel_bg))
            .alignment(Alignment::Left);
        let footer_right = Paragraph::new(version)
            .style(Style::default().fg(theme.secondary).bg(theme.panel_bg))
            .alignment(Alignment::Right);
        f.render_widget(footer_left, footer_chunks[0]);
        f.render_widget(footer_right, footer_chunks[1]);

        sidebar::render(f, chunks[0], theme);
    }
}
