use ratatui::prelude::*;
use ratatui::widgets::*;

pub struct TodoItem {
    pub id: String,
    pub content: String,
    pub completed: bool,
}

impl TodoItem {
    pub fn new(id: String, content: String) -> Self {
        Self {
            id,
            content,
            completed: false,
        }
    }

    pub fn render(&self, f: &mut Frame, area: Rect) {
        let checkbox = if self.completed {
            "[x]"
        } else {
            "[ ]"
        };
        
        let style = if self.completed {
            Style::default().fg(Color::Gray).add_modifier(Modifier::CROSSED_OUT)
        } else {
            Style::default()
        };

        let text = format!("{} {}", checkbox, self.content);
        let paragraph = Paragraph::new(text.as_str())
            .style(style);
        
        f.render_widget(paragraph, area);
    }

    pub fn toggle(&mut self) {
        self.completed = !self.completed;
    }
}
