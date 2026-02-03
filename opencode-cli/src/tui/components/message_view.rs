use crate::tui::components::virtual_scroll::VirtualScroll;
use crate::tui::theme::Theme;
use ratatui::prelude::*;
use ratatui::widgets::*;

/// Fixed height (in lines) per conversation turn row in the two-column layout.
const TURN_ROW_HEIGHT: u16 = 4;

/// One row in the conversation: either a user/assistant pair (left/right) or a full-width system/tool line.
#[derive(Clone)]
enum TurnRow {
    Pair { user: String, assistant: String },
    FullWidth(String),
}

pub struct MessageView {
    messages: Vec<String>,
    virtual_scroll: VirtualScroll,
}

impl MessageView {
    pub fn new() -> Self {
        Self {
            messages: Vec::new(),
            virtual_scroll: VirtualScroll::new(20),
        }
    }

    pub fn add_message(&mut self, message: String) {
        self.messages.push(message);
        let turns = Self::messages_to_turns(&self.messages);
        self.virtual_scroll.set_total_items(turns.len());
        self.virtual_scroll.scroll_to_bottom();
    }

    pub fn scroll_up(&mut self) {
        self.virtual_scroll.scroll_up(1);
    }

    pub fn scroll_down(&mut self) {
        self.virtual_scroll.scroll_down(1);
    }

    /// Parse flat messages with "You: "/"Assistant: "/"System: "/"Tool: " prefixes into turn rows.
    fn messages_to_turns(messages: &[String]) -> Vec<TurnRow> {
        let mut turns: Vec<TurnRow> = Vec::new();
        for msg in messages {
            if msg.starts_with("You: ") {
                let content = msg[5..].to_string();
                // If last turn is Pair with empty assistant, complete it with next Assistant or leave empty.
                // We push (user, "") and later Assistant will fill in. So here we always push a new Pair(user, "").
                turns.push(TurnRow::Pair {
                    user: content,
                    assistant: String::new(),
                });
            } else if msg.starts_with("Assistant: ") {
                let content = msg[11..].to_string();
                if let Some(TurnRow::Pair { user: _, assistant }) = turns.last_mut() {
                    if assistant.is_empty() {
                        *assistant = content;
                        continue;
                    }
                }
                turns.push(TurnRow::Pair {
                    user: String::new(),
                    assistant: content,
                });
            } else if msg.starts_with("System: ") || msg.starts_with("Tool: ") {
                turns.push(TurnRow::FullWidth(msg.clone()));
            }
            // else: unknown prefix, skip or could push FullWidth(msg)
        }
        turns
    }

    /// Wrap and truncate text to a maximum number of lines; returns lines for Paragraph.
    fn wrap_text_to_lines(
        text: &str,
        width: u16,
        max_lines: u16,
        style: Style,
    ) -> Vec<Line<'static>> {
        if width == 0 || max_lines == 0 {
            return Vec::new();
        }
        let width_usize = width as usize;
        let max_lines_usize = max_lines as usize;
        let mut result: Vec<Line<'static>> = Vec::new();

        fn push_chunk(
            s: String,
            width_usize: usize,
            max_lines_usize: usize,
            result: &mut Vec<Line<'static>>,
            style: Style,
        ) -> bool {
            let mut rest = s;
            while result.len() < max_lines_usize && !rest.is_empty() {
                let take: String = rest.chars().take(width_usize).collect();
                result.push(Line::from(Span::styled(take, style)));
                rest = rest.chars().skip(width_usize).collect();
            }
            result.len() >= max_lines_usize
        }

        for segment in text.split('\n') {
            let mut line = String::new();
            let mut line_len = 0usize;
            for word in segment.split_whitespace() {
                let need = word.chars().count() + if line.is_empty() { 0 } else { 1 };
                if line_len + need > width_usize && !line.is_empty() {
                    if push_chunk(
                        std::mem::take(&mut line),
                        width_usize,
                        max_lines_usize,
                        &mut result,
                        style,
                    ) {
                        return result;
                    }
                    line_len = 0;
                    line.push_str(word);
                    line_len += word.chars().count();
                } else {
                    if !line.is_empty() {
                        line.push(' ');
                        line_len += 1;
                    }
                    line.push_str(word);
                    line_len += word.chars().count();
                }
            }
            if !line.is_empty() {
                if push_chunk(line, width_usize, max_lines_usize, &mut result, style) {
                    return result;
                }
            }
        }
        result
    }

    pub fn render(&mut self, f: &mut Frame, area: Rect, theme: &Theme) {
        let block = Block::default()
            .title(vec![
                Span::styled("💬", Style::default().fg(theme.accent)),
                Span::raw(" Messages"),
            ])
            .borders(Borders::ALL)
            .border_style(theme.border_style())
            .style(theme.panel_style());

        let inner = block.inner(area);
        f.render_widget(block, area);

        let turns = Self::messages_to_turns(&self.messages);
        let total_turns = turns.len();
        let viewport_turn_count = (inner.height / TURN_ROW_HEIGHT).max(1) as usize;
        self.virtual_scroll.viewport_height = viewport_turn_count;
        self.virtual_scroll.set_total_items(total_turns);
        let (start, end) = self.virtual_scroll.visible_range();

        let text_style = Style::default().fg(theme.foreground);

        for (idx, turn) in turns[start..end.min(total_turns)].iter().enumerate() {
            let row_y = inner.y + idx as u16 * TURN_ROW_HEIGHT;
            let row_rect = Rect {
                x: inner.x,
                y: row_y,
                width: inner.width,
                height: TURN_ROW_HEIGHT,
            };

            match turn {
                TurnRow::Pair { user, assistant } => {
                    const AVATAR_WIDTH: u16 = 2;
                    let gap = 1u16;
                    let half = (inner.width.saturating_sub(gap)) / 2;
                    // Left: [avatar 2 cols][user bubble]
                    let left_avatar_rect = Rect {
                        x: inner.x,
                        y: row_y,
                        width: AVATAR_WIDTH,
                        height: TURN_ROW_HEIGHT,
                    };
                    let user_bubble_rect = Rect {
                        x: inner.x + AVATAR_WIDTH,
                        y: row_y,
                        width: half.saturating_sub(AVATAR_WIDTH),
                        height: TURN_ROW_HEIGHT,
                    };
                    // Right: [assistant bubble][avatar 2 cols]
                    let right_content_width = inner.width.saturating_sub(half + gap);
                    let assistant_bubble_rect = Rect {
                        x: inner.x + half + gap,
                        y: row_y,
                        width: right_content_width.saturating_sub(AVATAR_WIDTH),
                        height: TURN_ROW_HEIGHT,
                    };
                    let right_avatar_rect = Rect {
                        x: inner.x + half + gap + right_content_width.saturating_sub(AVATAR_WIDTH),
                        y: row_y,
                        width: AVATAR_WIDTH,
                        height: TURN_ROW_HEIGHT,
                    };

                    let user_lines = Self::wrap_text_to_lines(
                        user,
                        user_bubble_rect.width,
                        TURN_ROW_HEIGHT,
                        text_style,
                    );
                    let assistant_lines = Self::wrap_text_to_lines(
                        assistant,
                        assistant_bubble_rect.width,
                        TURN_ROW_HEIGHT,
                        text_style,
                    );

                    let user_bubble_style = Style::default()
                        .fg(theme.foreground)
                        .bg(theme.user_bubble_bg);
                    let assistant_bubble_style = Style::default()
                        .fg(theme.foreground)
                        .bg(theme.assistant_bubble_bg);

                    let user_avatar_para = Paragraph::new("👤")
                        .style(user_bubble_style)
                        .alignment(Alignment::Center);
                    let assistant_avatar_para = Paragraph::new("🤖")
                        .style(assistant_bubble_style)
                        .alignment(Alignment::Center);

                    f.render_widget(user_avatar_para, left_avatar_rect);
                    f.render_widget(
                        Paragraph::new(user_lines).style(user_bubble_style),
                        user_bubble_rect,
                    );
                    f.render_widget(
                        Paragraph::new(assistant_lines).style(assistant_bubble_style),
                        assistant_bubble_rect,
                    );
                    f.render_widget(assistant_avatar_para, right_avatar_rect);
                }
                TurnRow::FullWidth(text) => {
                    let full_lines =
                        Self::wrap_text_to_lines(text, inner.width, TURN_ROW_HEIGHT, text_style);
                    let para = Paragraph::new(full_lines)
                        .style(Style::default().fg(theme.foreground).bg(theme.panel_bg));
                    f.render_widget(para, row_rect);
                }
            }
        }

        self.virtual_scroll.render_scrollbar(f, area);
    }
}

impl Default for MessageView {
    fn default() -> Self {
        Self::new()
    }
}
