use ratatui::prelude::*;
use ratatui::widgets::*;

pub struct Tips {
    pub tips: Vec<String>,
    pub current: usize,
}

impl Tips {
    pub fn new() -> Self {
        Self {
            tips: vec![
                "Press 'n' to create a new session".to_string(),
                "Press 'Tab' to switch between agents".to_string(),
                "Press 'q' to quit".to_string(),
                "Use 'Esc' to go back".to_string(),
            ],
            current: 0,
        }
    }

    pub fn render(&self, f: &mut Frame, area: Rect) {
        if let Some(tip) = self.tips.get(self.current) {
            let block = Block::default()
                .title("Tip")
                .borders(Borders::ALL);
            
            let paragraph = Paragraph::new(tip.as_str())
                .block(block)
                .wrap(Wrap { trim: true });
            
            f.render_widget(paragraph, area);
        }
    }
}

impl Default for Tips {
    fn default() -> Self {
        Self::new()
    }
}
