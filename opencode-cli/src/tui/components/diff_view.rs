use ratatui::prelude::*;
use ratatui::widgets::*;

pub struct DiffView {
    old_content: String,
    new_content: String,
}

impl DiffView {
    pub fn new(old_content: String, new_content: String) -> Self {
        Self {
            old_content,
            new_content,
        }
    }

    pub fn render(&self, f: &mut Frame, area: Rect) {
        let chunks = Layout::default()
            .direction(Direction::Horizontal)
            .constraints([Constraint::Percentage(50), Constraint::Percentage(50)])
            .split(area);

        let old_block = Block::default()
            .title("Old")
            .borders(Borders::ALL)
            .border_style(Style::default().fg(Color::Red));
        
        let old_text = Paragraph::new(self.old_content.as_str())
            .block(old_block)
            .wrap(Wrap { trim: true });
        f.render_widget(old_text, chunks[0]);

        let new_block = Block::default()
            .title("New")
            .borders(Borders::ALL)
            .border_style(Style::default().fg(Color::Green));
        
        let new_text = Paragraph::new(self.new_content.as_str())
            .block(new_block)
            .wrap(Wrap { trim: true });
        f.render_widget(new_text, chunks[1]);
    }
}
