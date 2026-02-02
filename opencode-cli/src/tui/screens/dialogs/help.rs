use ratatui::prelude::*;
use ratatui::widgets::*;

pub struct HelpDialog {
    pub content: String,
}

impl HelpDialog {
    pub fn new() -> Self {
        Self {
            content: r#"
OpenCode TUI Help

Keyboard Shortcuts:
  q          - Quit application
  n          - New session
  Esc        - Go back / Close dialog
  Tab        - Switch agent
  Enter      - Confirm / Submit
  Arrow Keys - Navigate

Agents:
  build      - Full access agent for development
  plan       - Read-only agent for analysis
  general    - General purpose agent

Commands:
  Use @general to invoke the general subagent
  Use @plan to switch to plan agent
"#
            .trim()
            .to_string(),
        }
    }

    pub fn render(&self, f: &mut Frame, area: Rect) {
        let block = Block::default()
            .title("Help")
            .borders(Borders::ALL);

        let paragraph = Paragraph::new(self.content.as_str())
            .block(block)
            .wrap(Wrap { trim: true })
            .scroll((0, 0));

        f.render_widget(paragraph, area);
    }
}

impl Default for HelpDialog {
    fn default() -> Self {
        Self::new()
    }
}
