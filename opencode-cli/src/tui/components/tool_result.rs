use ratatui::prelude::*;
use ratatui::widgets::*;

pub struct ToolResultView {
    title: String,
    output: String,
    metadata: String,
}

impl ToolResultView {
    pub fn new(title: String, output: String, metadata: String) -> Self {
        Self {
            title,
            output,
            metadata,
        }
    }

    pub fn render(&self, f: &mut Frame, area: Rect) {
        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Length(3),
                Constraint::Min(1),
                Constraint::Length(3),
            ])
            .split(area);

        let title_block = Block::default()
            .title(self.title.as_str())
            .borders(Borders::ALL);
        
        let title_text = Paragraph::new(self.title.as_str())
            .block(title_block)
            .style(Style::default().fg(Color::Cyan));
        f.render_widget(title_text, chunks[0]);

        let output_block = Block::default()
            .title("Output")
            .borders(Borders::ALL);
        
        let output_text = Paragraph::new(self.output.as_str())
            .block(output_block)
            .wrap(Wrap { trim: true });
        f.render_widget(output_text, chunks[1]);

        let metadata_block = Block::default()
            .title("Metadata")
            .borders(Borders::ALL);
        
        let metadata_text = Paragraph::new(self.metadata.as_str())
            .block(metadata_block)
            .style(Style::default().fg(Color::Gray));
        f.render_widget(metadata_text, chunks[2]);
    }
}
