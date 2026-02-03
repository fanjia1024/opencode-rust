use ratatui::prelude::*;
use ratatui::widgets::*;
use crate::tui::theme::Theme;

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

    pub fn render(&self, f: &mut Frame, area: Rect, theme: &Theme) {
        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([Constraint::Length(3), Constraint::Min(1)])
            .split(area);

        let header = Block::default()
            .title(vec![
                Span::styled(" 🤖 ", Style::default().fg(theme.accent)),
                Span::styled("OpenCode - Home", theme.primary_style()),
                Span::raw(" (n: new session)"),
            ])
            .borders(Borders::ALL)
            .border_style(theme.border_style())
            .style(theme.panel_style());
        f.render_widget(header, chunks[0]);

        if self.sessions.is_empty() {
            let empty = Paragraph::new(vec![
                Line::from(vec![Span::styled("📭", Style::default().fg(theme.secondary)), Span::raw(" No sessions yet.")]),
                Line::from(vec![Span::styled("💡", Style::default().fg(theme.accent)), Span::raw(" Press 'n' to create a new session.")]),
            ])
                .block(Block::default().borders(Borders::NONE).style(theme.default_style()))
                .alignment(Alignment::Center);
            f.render_widget(empty, chunks[1]);
        } else {
            let items: Vec<ListItem> = self
                .sessions
                .iter()
                .enumerate()
                .map(|(i, session)| {
                    let style = if i == self.selected {
                        Style::default().fg(theme.highlight).bg(Color::Rgb(40, 40, 60)).add_modifier(Modifier::BOLD)
                    } else {
                        Style::default().fg(theme.foreground)
                    };
                    
                    let title_span = Span::styled(&session.title, Style::default().add_modifier(Modifier::BOLD));
                    let time_span = Span::styled(&session.updated, Style::default().fg(theme.secondary));
                    
                    ListItem::new(vec![
                        Line::from(vec![Span::styled("📝 ", Style::default().fg(theme.primary)), title_span]),
                        Line::from(vec![Span::styled("🕒 ", Style::default().fg(theme.secondary)), time_span]),
                    ]).style(style)
                })
                .collect();

            let list = List::new(items)
                .block(
                    Block::default()
                        .title(vec![Span::styled("📋", Style::default().fg(theme.accent)), Span::raw(" Sessions")])
                        .borders(Borders::ALL)
                        .border_style(theme.border_style())
                        .style(theme.panel_style()),
                )
                .highlight_style(theme.highlight_style().add_modifier(Modifier::BOLD))
                .style(Style::default().fg(theme.foreground));
            f.render_widget(list, chunks[1]);
        }
    }
}

impl Default for HomeScreen {
    fn default() -> Self {
        Self::new()
    }
}