use ratatui::prelude::*;
use ratatui::widgets::*;

pub struct AlertDialog {
    pub message: String,
}

impl AlertDialog {
    pub fn new(message: String) -> Self {
        Self { message }
    }

    pub fn render(&self, f: &mut Frame, area: Rect) {
        let block = Block::default()
            .title("Alert")
            .borders(Borders::ALL);
        
        let paragraph = Paragraph::new(self.message.as_str())
            .block(block)
            .wrap(Wrap { trim: true });
        
        f.render_widget(paragraph, area);
    }
}
