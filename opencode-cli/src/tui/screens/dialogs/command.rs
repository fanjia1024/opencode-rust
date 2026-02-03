use crossterm::event::{KeyCode, KeyEvent};
use ratatui::prelude::*;
use ratatui::widgets::*;

/// A slash command with id (for branching), label (for display/insert), and description.
pub struct SlashCommand {
    pub id: String,
    pub label: String,
    pub description: String,
}

/// Result of selecting a command: (id, label). App branches on id (e.g. run init) and inserts label for others.
pub type CommandSelection = (String, String);

pub struct CommandDialog {
    commands: Vec<SlashCommand>,
    selected: usize,
    filter: String,
}

impl CommandDialog {
    pub fn new() -> Self {
        Self {
            commands: vec![
                SlashCommand {
                    id: "init".to_string(),
                    label: "/init".to_string(),
                    description: "create/update AGENTS.md".to_string(),
                },
                SlashCommand {
                    id: "review".to_string(),
                    label: "/review".to_string(),
                    description: "review changes (commit/branch/pr)".to_string(),
                },
                SlashCommand {
                    id: "plan".to_string(),
                    label: "/plan".to_string(),
                    description: "plan mode / break down requirements".to_string(),
                },
                SlashCommand {
                    id: "code-review".to_string(),
                    label: "/code-review".to_string(),
                    description: "start code review with code-reviewer agent".to_string(),
                },
                SlashCommand {
                    id: "build-fix".to_string(),
                    label: "/build-fix".to_string(),
                    description: "fix build errors with fix agent".to_string(),
                },
                SlashCommand {
                    id: "tdd".to_string(),
                    label: "/tdd".to_string(),
                    description: "start TDD workflow with tdd-guide agent".to_string(),
                },
                SlashCommand {
                    id: "learn".to_string(),
                    label: "/learn".to_string(),
                    description: "extract patterns from session".to_string(),
                },
            ],
            selected: 0,
            filter: String::new(),
        }
    }

    fn filtered_commands(&self) -> Vec<&SlashCommand> {
        if self.filter.is_empty() {
            self.commands.iter().collect()
        } else {
            let f = self.filter.to_lowercase();
            self.commands
                .iter()
                .filter(|c| {
                    c.label.to_lowercase().contains(&f) || c.description.to_lowercase().contains(&f)
                })
                .collect()
        }
    }

    pub fn render(&self, f: &mut Frame, area: Rect) {
        let popup_area = centered_rect(60, 50, area);

        let block = Block::default()
            .title("Select Command")
            .borders(Borders::ALL);

        let filtered = self.filtered_commands();
        if filtered.is_empty() {
            let empty = Paragraph::new("No commands match").block(block);
            f.render_widget(empty, popup_area);
            return;
        }

        // Two columns: label (left), description (right)
        let label_width = filtered
            .iter()
            .map(|c| c.label.len().min(24))
            .max()
            .unwrap_or(12)
            .min(popup_area.width.saturating_sub(4) as usize / 2);
        let desc_width = popup_area.width.saturating_sub(4) as usize - label_width - 2;

        let rows: Vec<Row> = filtered
            .iter()
            .enumerate()
            .map(|(i, cmd)| {
                let label = if cmd.label.len() > label_width {
                    format!("{}…", &cmd.label[..label_width.saturating_sub(1)])
                } else {
                    cmd.label.clone()
                };
                let desc = if cmd.description.len() > desc_width {
                    format!("{}…", &cmd.description[..desc_width.saturating_sub(1)])
                } else {
                    cmd.description.clone()
                };
                let style = if i == self.selected {
                    Style::default().fg(Color::Yellow)
                } else {
                    Style::default()
                };
                Row::new(vec![
                    Cell::from(label).style(style),
                    Cell::from(desc).style(style),
                ])
            })
            .collect();

        let widths = [
            Constraint::Length(label_width as u16 + 1),
            Constraint::Min(10),
        ];
        let table = Table::new(rows, widths).block(block).column_spacing(1);

        f.render_widget(table, popup_area);
    }

    /// Returns Some((id, label)) when user selects a command (Enter), None on Esc or other keys.
    pub fn handle_key(&mut self, key: KeyEvent) -> Option<CommandSelection> {
        let filtered = self.filtered_commands();
        let len = filtered.len();
        if len == 0 {
            if key.code == KeyCode::Esc {
                return None;
            }
            return None;
        }

        match key.code {
            KeyCode::Up => {
                if self.selected > 0 {
                    self.selected -= 1;
                }
                None
            }
            KeyCode::Down => {
                if self.selected < len - 1 {
                    self.selected += 1;
                }
                None
            }
            KeyCode::Enter => {
                tklog::info!("command_dialog Enter", self.selected, filtered.len());
                let out = filtered
                    .get(self.selected)
                    .map(|c| (c.id.clone(), c.label.clone()));
                if let Some((ref id, ref label)) = out {
                    tklog::info!("command_dialog returning selection", id, label);
                }
                out
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
