use ratatui::prelude::*;
use ratatui::widgets::*;

pub struct PromptDialog {
    pub message: String,
    pub input: String,
}

impl PromptDialog {
    pub fn new(message: String) -> Self {
        Self {
            message,
            input: String::new(),
        }
    }

    pub fn render(&self, f: &mut Frame, area: Rect) {
        let block = Block::default()
            .title("Prompt")
            .borders(Borders::ALL);
        
        let paragraph = Paragraph::new(format!("{}\n\nInput: {}", self.message, self.input))
            .block(block)
            .wrap(Wrap { trim: true });
        
        f.render_widget(paragraph, area);
    }
}
