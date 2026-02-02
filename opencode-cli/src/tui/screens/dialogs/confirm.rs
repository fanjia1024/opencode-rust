use ratatui::prelude::*;
use ratatui::widgets::*;

pub struct ConfirmDialog {
    pub message: String,
    pub confirmed: bool,
}

impl ConfirmDialog {
    pub fn new(message: String) -> Self {
        Self {
            message,
            confirmed: false,
        }
    }

    pub fn render(&self, f: &mut Frame, area: Rect) {
        let block = Block::default()
            .title("Confirm")
            .borders(Borders::ALL);
        
        let paragraph = Paragraph::new(self.message.as_str())
            .block(block)
            .wrap(Wrap { trim: true });
        
        f.render_widget(paragraph, area);
    }
}
