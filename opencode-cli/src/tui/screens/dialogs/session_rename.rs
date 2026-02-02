use ratatui::prelude::*;
use ratatui::widgets::*;
use crossterm::event::{KeyEvent, KeyCode};

pub struct SessionRenameDialog {
    current_name: String,
    new_name: String,
    cursor_position: usize,
}

impl SessionRenameDialog {
    pub fn new(current_name: String) -> Self {
        Self {
            current_name: current_name.clone(),
            new_name: current_name,
            cursor_position: 0,
        }
    }

    pub fn render(&self, f: &mut Frame, area: Rect) {
        let popup_area = centered_rect(50, 20, area);
        
        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Length(3),
                Constraint::Length(3),
            ])
            .split(popup_area);

        let input_block = Block::default()
            .title("New Name")
            .borders(Borders::ALL);
        
        let input = Paragraph::new(self.new_name.as_str())
            .block(input_block);
        
        f.render_widget(input, chunks[1]);
    }

    pub fn handle_key(&mut self, key: KeyEvent) -> Option<String> {
        match key.code {
            KeyCode::Char(c) => {
                self.new_name.insert(self.cursor_position, c);
                self.cursor_position += 1;
                None
            }
            KeyCode::Backspace => {
                if self.cursor_position > 0 {
                    self.cursor_position -= 1;
                    self.new_name.remove(self.cursor_position);
                }
                None
            }
            KeyCode::Left => {
                if self.cursor_position > 0 {
                    self.cursor_position -= 1;
                }
                None
            }
            KeyCode::Right => {
                if self.cursor_position < self.new_name.len() {
                    self.cursor_position += 1;
                }
                None
            }
            KeyCode::Enter => {
                Some(self.new_name.clone())
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
