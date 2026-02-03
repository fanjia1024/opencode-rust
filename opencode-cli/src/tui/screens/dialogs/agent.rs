use ratatui::prelude::*;
use ratatui::widgets::*;
use crossterm::event::{KeyCode, KeyEvent};

pub struct AgentDialog {
    pub agents: Vec<String>,
    pub selected: usize,
    pub current_agent: String,
}

impl AgentDialog {
    pub fn new(agents: Vec<String>, current_agent: String) -> Self {
        let selected = agents
            .iter()
            .position(|a| a == &current_agent)
            .unwrap_or(0);
        Self {
            agents,
            selected,
            current_agent,
        }
    }

    pub fn render(&self, f: &mut Frame, area: Rect, theme: &crate::tui::theme::Theme) {
        let popup_area = centered_rect(50, 50, area);
        let block = Block::default()
            .title("Select Agent (Enter to switch, Esc to close)")
            .borders(Borders::ALL)
            .border_style(theme.border_style())
            .style(theme.panel_style());
        let inner = block.inner(popup_area);
        f.render_widget(block, popup_area);

        let items: Vec<ListItem> = self
            .agents
            .iter()
            .enumerate()
            .map(|(i, name)| {
                let is_current = name == &self.current_agent;
                let is_selected = i == self.selected;
                let prefix = if is_current { "[current] " } else { "" };
                let style = if is_selected {
                    theme.highlight_style()
                } else if is_current {
                    theme.primary_style()
                } else {
                    Style::default().fg(theme.foreground)
                };
                ListItem::new(format!("{}{}", prefix, name)).style(style)
            })
            .collect();
        let list = List::new(items);
        f.render_widget(list, inner);
    }

    /// Returns Some(agent_name) to switch, None to cancel/close.
    pub fn handle_key(&mut self, key: KeyEvent) -> AgentDialogAction {
        match key.code {
            KeyCode::Esc => AgentDialogAction::Cancel,
            KeyCode::Enter => {
                if let Some(name) = self.agents.get(self.selected) {
                    AgentDialogAction::Switch(name.clone())
                } else {
                    AgentDialogAction::Cancel
                }
            }
            KeyCode::Up => {
                if self.selected > 0 {
                    self.selected -= 1;
                }
                AgentDialogAction::Continue
            }
            KeyCode::Down => {
                if self.selected + 1 < self.agents.len() {
                    self.selected += 1;
                }
                AgentDialogAction::Continue
            }
            _ => AgentDialogAction::Continue,
        }
    }
}

pub enum AgentDialogAction {
    Continue,
    Switch(String),
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
