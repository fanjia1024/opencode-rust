use ratatui::prelude::*;
use ratatui::widgets::*;
use crossterm::event::{KeyEvent, KeyCode};

pub struct CommandDialog {
    commands: Vec<String>,
    selected: usize,
    filter: String,
}

impl CommandDialog {
    pub fn new() -> Self {
        Self {
            commands: vec![
                "read".to_string(),
                "write".to_string(),
                "edit".to_string(),
                "grep".to_string(),
                "bash".to_string(),
                "question".to_string(),
            ],
            selected: 0,
            filter: String::new(),
        }
    }

    pub fn render(&self, f: &mut Frame, area: Rect) {
        let popup_area = centered_rect(60, 50, area);
        
        let block = Block::default()
            .title("Select Command")
            .borders(Borders::ALL);

        let filtered: Vec<&String> = if self.filter.is_empty() {
            self.commands.iter().collect()
        } else {
            self.commands
                .iter()
                .filter(|cmd| cmd.contains(&self.filter))
                .collect()
        };

        let items: Vec<ListItem> = filtered
            .iter()
            .enumerate()
            .map(|(i, cmd)| {
                let style = if i == self.selected {
                    Style::default().fg(Color::Yellow)
                } else {
                    Style::default()
                };
                ListItem::new(cmd.as_str()).style(style)
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
                let filtered_len = if self.filter.is_empty() {
                    self.commands.len()
                } else {
                    self.commands.iter().filter(|c| c.contains(&self.filter)).count()
                };
                if self.selected < filtered_len - 1 {
                    self.selected += 1;
                }
                None
            }
            KeyCode::Enter => {
                let filtered: Vec<&String> = if self.filter.is_empty() {
                    self.commands.iter().collect()
                } else {
                    self.commands
                        .iter()
                        .filter(|cmd| cmd.contains(&self.filter))
                        .collect()
                };
                filtered.get(self.selected).map(|s| s.to_string())
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

impl Default for CommandDialog {
    fn default() -> Self {
        Self::new()
    }
}
