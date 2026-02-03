use ratatui::prelude::*;
use ratatui::widgets::*;
use crate::tui::components::{syntax_highlighter::SyntaxHighlighter, virtual_scroll::VirtualScroll};
use crate::tui::theme::Theme;

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

    pub fn render(&mut self, f: &mut Frame, area: Rect, theme: &Theme) {
        let block = Block::default()
            .title(vec![Span::styled("💬", Style::default().fg(theme.accent)), Span::raw(" Messages")])
            .borders(Borders::ALL)
            .border_style(theme.border_style())
            .style(theme.panel_style());
        
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
                    // Format message with role prefixes
                    let formatted_lines = if msg.starts_with("You: ") {
                        vec![Line::from(vec![
                            Span::styled("👤 You: ", Style::default().fg(theme.primary).add_modifier(Modifier::BOLD)),
                            Span::raw(&msg[5..]) // Skip "You: " prefix
                        ])]
                    } else if msg.starts_with("Assistant: ") {
                        vec![Line::from(vec![
                            Span::styled("🤖 Assistant: ", Style::default().fg(theme.secondary).add_modifier(Modifier::BOLD)),
                            Span::raw(&msg[12..]) // Skip "Assistant: " prefix
                        ])]
                    } else if msg.starts_with("System: ") {
                        vec![Line::from(vec![
                            Span::styled("⚙️ System: ", Style::default().fg(theme.warning).add_modifier(Modifier::BOLD)),
                            Span::raw(&msg[8..]) // Skip "System: " prefix
                        ])]
                    } else if msg.starts_with("Tool: ") {
                        vec![Line::from(vec![
                            Span::styled("🛠️ Tool: ", Style::default().fg(theme.accent).add_modifier(Modifier::BOLD)),
                            Span::raw(&msg[6..]) // Skip "Tool: " prefix
                        ])]
                    } else {
                        vec![Line::from(msg.as_str())]
                    };
                    
                    formatted_lines
                };
                
                ListItem::new(lines).style(Style::default().bg(if msg.starts_with("You: ") { 
                    Color::Rgb(30, 30, 45) 
                } else if msg.starts_with("Assistant: ") { 
                    Color::Rgb(25, 35, 45) 
                } else { 
                    theme.panel_bg 
                }))
            })
            .collect();

        let list = List::new(items)
            .block(block)
            .highlight_style(theme.highlight_style());
        
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