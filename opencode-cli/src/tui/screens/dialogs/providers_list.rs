use ratatui::prelude::*;
use ratatui::widgets::*;
use crossterm::event::{KeyCode, KeyEvent};

use crate::config::ProviderListItem;

pub struct ProvidersListDialog {
    pub items: Vec<ProviderListItem>,
    pub selected: usize,
}

impl ProvidersListDialog {
    pub fn new(items: Vec<ProviderListItem>) -> Self {
        Self {
            items,
            selected: 0,
        }
    }

    pub fn render(&self, f: &mut Frame, area: Rect, theme: &crate::tui::theme::Theme) {
        let popup_area = centered_rect(70, 60, area);
        let block = Block::default()
            .title("Providers (Enter: set default, e: edit, Esc: close)")
            .borders(Borders::ALL)
            .border_style(theme.border_style())
            .style(theme.panel_style());
        let inner = block.inner(popup_area);
        f.render_widget(block, popup_area);

        if self.items.is_empty() {
            let empty = Paragraph::new("No providers configured. Press 'C' to add default.")
                .style(Style::default().fg(theme.secondary));
            f.render_widget(empty, inner);
            return;
        }

        let list_items: Vec<ListItem> = self
            .items
            .iter()
            .enumerate()
            .map(|(i, p)| {
                let model = p.model.as_deref().unwrap_or("—");
                let line = format!("{}  {}  model: {}", p.id, p.provider_type, model);
                let style = if i == self.selected {
                    theme.highlight_style()
                } else {
                    Style::default().fg(theme.foreground)
                };
                ListItem::new(line).style(style)
            })
            .collect();
        let list = List::new(list_items);
        f.render_widget(list, inner);
    }

    pub fn handle_key(&mut self, key: KeyEvent) -> ProvidersListAction {
        match key.code {
            KeyCode::Esc => ProvidersListAction::Cancel,
            KeyCode::Enter => {
                if self.items.is_empty() {
                    ProvidersListAction::Cancel
                } else {
                    ProvidersListAction::SetDefault(self.selected)
                }
            }
            KeyCode::Char('e') | KeyCode::Char('E') => {
                if self.items.is_empty() {
                    ProvidersListAction::Cancel
                } else {
                    ProvidersListAction::Edit(self.selected)
                }
            }
            KeyCode::Up => {
                if self.selected > 0 {
                    self.selected -= 1;
                }
                ProvidersListAction::Continue
            }
            KeyCode::Down => {
                if self.selected + 1 < self.items.len() {
                    self.selected += 1;
                }
                ProvidersListAction::Continue
            }
            _ => ProvidersListAction::Continue,
        }
    }
}

pub enum ProvidersListAction {
    Continue,
    SetDefault(usize),
    Edit(usize),
    Cancel,
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
