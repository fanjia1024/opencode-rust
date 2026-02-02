use ratatui::prelude::*;
use ratatui::widgets::*;
use crossterm::event::{KeyEvent, KeyCode};

pub struct SessionListDialog {
    sessions: Vec<SessionInfo>,
    selected: usize,
}

pub struct SessionInfo {
    pub id: String,
    pub title: String,
    pub updated: String,
}

impl SessionListDialog {
    pub fn new(sessions: Vec<SessionInfo>) -> Self {
        Self {
            sessions,
            selected: 0,
        }
    }

    pub fn render(&self, f: &mut Frame, area: Rect) {
        let popup_area = centered_rect(70, 60, area);
        
        let block = Block::default()
            .title("Sessions")
            .borders(Borders::ALL);

        let items: Vec<ListItem> = self
            .sessions
            .iter()
            .enumerate()
            .map(|(i, session)| {
                let style = if i == self.selected {
                    Style::default().fg(Color::Yellow)
                } else {
                    Style::default()
                };
                let text = format!("{} - {}", session.title, session.updated);
                ListItem::new(text).style(style)
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
                if self.selected < self.sessions.len().saturating_sub(1) {
                    self.selected += 1;
                }
                None
            }
            KeyCode::Enter => {
                self.sessions.get(self.selected).map(|s| s.id.clone())
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
