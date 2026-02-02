use ratatui::prelude::*;
use ratatui::widgets::*;

pub struct Toast {
    pub message: String,
    pub level: ToastLevel,
}

#[derive(Clone, Copy)]
pub enum ToastLevel {
    Info,
    Success,
    Warning,
    Error,
}

impl Toast {
    pub fn new(message: String, level: ToastLevel) -> Self {
        Self { message, level }
    }

    pub fn render(&self, f: &mut Frame, area: Rect) {
        let color = match self.level {
            ToastLevel::Info => Color::Cyan,
            ToastLevel::Success => Color::Green,
            ToastLevel::Warning => Color::Yellow,
            ToastLevel::Error => Color::Red,
        };

        let block = Block::default()
            .borders(Borders::ALL)
            .border_style(Style::default().fg(color));

        let paragraph = Paragraph::new(self.message.as_str())
            .block(block)
            .style(Style::default().fg(color))
            .wrap(Wrap { trim: true });

        f.render_widget(paragraph, area);
    }
}
