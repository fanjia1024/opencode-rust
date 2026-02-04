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
  (Home)     - q Quit | n New session
  (Session)  - Letters type into input; use Ctrl+letter for shortcuts:
               Ctrl+Q Quit | Ctrl+C Config | Ctrl+A Agent | Ctrl+P Providers | Ctrl+H Help
  /          - Command palette (Session)
  Esc        - Go back / Close dialog
  Enter      - Confirm / Submit
  Arrow Keys - Navigate

  Home: single key q/c/a/p/h. Session: type letters in input; Ctrl+Q/C/A/P/H for shortcuts.

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
