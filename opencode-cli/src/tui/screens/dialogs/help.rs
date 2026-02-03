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
  n          - New session (Home)
  Esc        - Go back / Close dialog
  Enter      - Confirm / Submit
  Arrow Keys - Navigate

  A          - Agents: list and switch current agent (build, plan, general)
  C          - Provider config: edit default provider (type, model, API key)
  P          - Providers: list configured providers, set default, edit (e)
  H / ?      - Help (this screen)

Agents:
  build      - Full access agent for development
  plan       - Read-only agent for analysis
  general    - General purpose agent

Providers / Model:
  Current model is set in Provider config (C). List providers with P.

Commands:
  Sessions list / Config show: use CLI (opencode sessions list, opencode config show).
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
