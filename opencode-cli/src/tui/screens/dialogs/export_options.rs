use ratatui::prelude::*;
use ratatui::widgets::*;
use crossterm::event::{KeyEvent, KeyCode};

pub struct ExportOptionsDialog {
    formats: Vec<String>,
    selected: usize,
}

impl ExportOptionsDialog {
    pub fn new() -> Self {
        Self {
            formats: vec![
                "JSON".to_string(),
                "Markdown".to_string(),
                "Text".to_string(),
            ],
            selected: 0,
        }
    }

    pub fn render(&self, f: &mut Frame, area: Rect) {
        let popup_area = centered_rect(40, 30, area);
        
        let block = Block::default()
            .title("Export Format")
            .borders(Borders::ALL);

        let items: Vec<ListItem> = self
            .formats
            .iter()
            .enumerate()
            .map(|(i, format)| {
                let style = if i == self.selected {
                    Style::default().fg(Color::Yellow)
                } else {
                    Style::default()
                };
                ListItem::new(format.as_str()).style(style)
            })
            .collect();

        let list = List::new(items)
            .block(block);

        f.render_widget(list, popup_area);
    }

    pub fn handle_key(&mut self, key: KeyEvent) -> Option<String> {
        match key.code {
            KeyCode::Up => {
                if self.selected > 0 {
                    self.selected -= 1;
                }
                None
            }
            KeyCode::Down => {
                if self.selected < self.formats.len() - 1 {
                    self.selected += 1;
                }
                None
            }
            KeyCode::Enter => {
                self.formats.get(self.selected).cloned()
            }
            KeyCode::Esc => None,
            _ => None,
        }
    }
}

fn centered_rect(percent_x: u16, percent_y: u16, r: Rect) -> Rect {
    let popup_layout = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Percentage((100 - percent_y) / 2),
            Constraint::Percentage(percent_y),
            Constraint::Percentage((100 - percent_y) / 2),
        ])
        .split(r);

    Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Percentage((100 - percent_x) / 2),
            Constraint::Percentage(percent_x),
            Constraint::Percentage((100 - percent_x) / 2),
        ])
        .split(popup_layout[1])[1]
}

impl Default for ExportOptionsDialog {
    fn default() -> Self {
        Self::new()
    }
}
