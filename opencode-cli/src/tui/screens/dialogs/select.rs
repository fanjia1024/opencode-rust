use ratatui::prelude::*;
use ratatui::widgets::*;

pub struct SelectDialog {
    pub title: String,
    pub items: Vec<String>,
    pub selected: usize,
}

impl SelectDialog {
    pub fn new(title: String, items: Vec<String>) -> Self {
        Self {
            title,
            items,
            selected: 0,
        }
    }

    pub fn render(&self, f: &mut Frame, area: Rect) {
        let block = Block::default()
            .title(self.title.as_str())
            .borders(Borders::ALL);
        
        let items: Vec<ListItem> = self
            .items
            .iter()
            .enumerate()
            .map(|(i, item)| {
                let style = if i == self.selected {
                    Style::default().fg(Color::Yellow)
                } else {
                    Style::default()
                };
                ListItem::new(item.as_str()).style(style)
            })
            .collect();
        
        let list = List::new(items).block(block);
        f.render_widget(list, area);
    }
}
