use ratatui::prelude::*;
use ratatui::widgets::*;

pub struct PromptComponent {
    pub input: String,
    pub cursor_position: usize,
}

impl PromptComponent {
    pub fn new() -> Self {
        Self {
            input: String::new(),
            cursor_position: 0,
        }
    }

    pub fn render(&self, f: &mut Frame, area: Rect) {
        let block = Block::default()
            .title("Input")
            .borders(Borders::ALL);
        let paragraph = Paragraph::new(self.input.as_str())
            .block(block);
        f.render_widget(paragraph, area);
    }
}

impl Default for PromptComponent {
    fn default() -> Self {
        Self::new()
    }
}
