use ratatui::prelude::*;
use ratatui::widgets::*;
use unicode_width::UnicodeWidthStr;
use crate::tui::components::syntax_highlighter::SyntaxHighlighter;

pub struct CodeBlock {
    code: String,
    language: String,
    highlighter: SyntaxHighlighter,
}

impl CodeBlock {
    pub fn new(code: String, language: String) -> Self {
        Self {
            code,
            language,
            highlighter: SyntaxHighlighter::new(),
        }
    }

    pub fn render(&self, f: &mut Frame, area: Rect) {
        let title = format!("Code ({})", self.language);
        let block = Block::default()
            .title(title.as_str())
            .borders(Borders::ALL);
        
        let inner = block.inner(area);
        let highlighted = self.highlighter.highlight_code(&self.code, &self.language);
        
        let mut lines = Vec::new();
        let mut current_line = Vec::new();
        let mut current_width = 0;
        
        for grapheme in highlighted {
            let width = grapheme.width();
            if current_width + width > inner.width as usize && !current_line.is_empty() {
                lines.push(Line::from(current_line.clone()));
                current_line.clear();
                current_width = 0;
            }
            current_line.push(Span::styled(grapheme.symbol.clone(), grapheme.style));
            current_width += width;
        }
        if !current_line.is_empty() {
            lines.push(Line::from(current_line));
        }
        
        let paragraph = Paragraph::new(lines)
            .block(block)
            .wrap(Wrap { trim: true });
        
        f.render_widget(paragraph, area);
    }
}
