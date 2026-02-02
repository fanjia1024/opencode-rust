use ratatui::prelude::*;
use ratatui::widgets::*;

pub struct HomeScreen {
    pub sessions: Vec<SessionInfo>,
    pub selected: usize,
}

pub struct SessionInfo {
    pub id: String,
    pub title: String,
    pub updated: String,
}

impl HomeScreen {
    pub fn new() -> Self {
        Self {
            sessions: Vec::new(),
            selected: 0,
        }
    }

    pub fn render(&self, f: &mut Frame, area: Rect) {
        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([Constraint::Length(3), Constraint::Min(1)])
            .split(area);

        let header = Block::default()
            .title("OpenCode - Home (n: new session)")
            .borders(Borders::ALL);
        f.render_widget(header, chunks[0]);

        if self.sessions.is_empty() {
            let empty = Paragraph::new(vec![
                Line::from("No sessions yet."),
                Line::from("Press 'n' to create a new session."),
            ])
                .block(Block::default().borders(Borders::NONE))
                .alignment(Alignment::Center);
            f.render_widget(empty, chunks[1]);
        } else {
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
                    ListItem::new(format!("{} - {}", session.title, session.updated))
                        .style(style)
                })
                .collect();

            let list = List::new(items)
                .block(
                    Block::default()
                        .title("Sessions")
                        .borders(Borders::NONE),
                );
            f.render_widget(list, chunks[1]);
        }
    }
}

impl Default for HomeScreen {
    fn default() -> Self {
        Self::new()
    }
}
