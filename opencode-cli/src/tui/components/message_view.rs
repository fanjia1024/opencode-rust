use ratatui::prelude::*;
use ratatui::widgets::*;
use crate::tui::components::{syntax_highlighter::SyntaxHighlighter, virtual_scroll::VirtualScroll};

pub struct MessageView {
    messages: Vec<String>,
    virtual_scroll: VirtualScroll,
    highlighter: SyntaxHighlighter,
}

impl MessageView {
    pub fn new() -> Self {
        Self {
            messages: Vec::new(),
            virtual_scroll: VirtualScroll::new(20),
            highlighter: SyntaxHighlighter::new(),
        }
    }

    pub fn add_message(&mut self, message: String) {
        self.messages.push(message);
        self.virtual_scroll.set_total_items(self.messages.len());
        self.virtual_scroll.scroll_to_bottom();
    }

    pub fn scroll_up(&mut self) {
        self.virtual_scroll.scroll_up(1);
    }

    pub fn scroll_down(&mut self) {
        self.virtual_scroll.scroll_down(1);
    }

    pub fn render(&mut self, f: &mut Frame, area: Rect) {
        let block = Block::default()
            .title("Messages")
            .borders(Borders::ALL);
        
        let inner = block.inner(area);
        self.virtual_scroll.viewport_height = inner.height as usize;
        self.virtual_scroll.set_total_items(self.messages.len());

        let (start, end) = self.virtual_scroll.visible_range();
        let visible_messages: Vec<String> = if start < self.messages.len() {
            self.messages[start..end.min(self.messages.len())].to_vec()
        } else {
            Vec::new()
        };

        let items: Vec<ListItem> = visible_messages
            .iter()
            .map(|msg| {
                let lines: Vec<Line> = if self.is_code_block(msg) {
                    let (code, lang) = self.extract_code_block(msg);
                    let highlighted = self.highlighter.highlight_code(&code, &lang);
                    let mut result = Vec::new();
                    let mut current_line = Vec::new();
                    let mut current_width = 0;
                    
                    for grapheme in highlighted {
                        let width = grapheme.width();
                        if current_width + width > inner.width as usize && !current_line.is_empty() {
                            result.push(Line::from(current_line.clone()));
                            current_line.clear();
                            current_width = 0;
                        }
                        current_line.push(Span::styled(grapheme.symbol.clone(), grapheme.style));
                        current_width += width;
                    }
                    if !current_line.is_empty() {
                        result.push(Line::from(current_line));
                    }
                    result
                } else {
                    msg.lines()
                        .map(|line| Line::from(line.to_string()))
                        .collect()
                };
                ListItem::new(lines)
            })
            .collect();

        let list = List::new(items)
            .block(block);
        
        f.render_widget(list, area);
        self.virtual_scroll.render_scrollbar(f, area);
    }

    fn is_code_block(&self, text: &str) -> bool {
        text.starts_with("```") && text.contains('\n')
    }

    fn extract_code_block(&self, text: &str) -> (String, String) {
        let lines: Vec<&str> = text.lines().collect();
        if lines.is_empty() {
            return (String::new(), "text".to_string());
        }
        
        let first_line = lines[0];
        let language = if first_line.starts_with("```") {
            first_line.strip_prefix("```").unwrap_or("").trim().to_string()
        } else {
            "text".to_string()
        };
        
        let code = if lines.len() > 2 {
            lines[1..lines.len()-1].join("\n")
        } else {
            String::new()
        };
        
        (code, language)
    }
}

impl Default for MessageView {
    fn default() -> Self {
        Self::new()
    }
}
