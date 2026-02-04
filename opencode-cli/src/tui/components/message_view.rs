use crate::tui::components::virtual_scroll::VirtualScroll;
use crate::tui::theme::Theme;
use ratatui::prelude::*;
use ratatui::widgets::*;

/// Minimum lines per turn (so empty bubbles still take one row).
const MIN_TURN_LINES: usize = 1;

/// Match provider's strip_think_blocks: content inside <think>...</think>
const THINK_OPEN: &str = "think>";
const THINK_CLOSE: &str = "</think>";

/// Parses streaming buffer into (think_content, visible_content).
/// Think blocks are shown as "AI 思考"; visible is the reply shown in the assistant bubble.
/// Strips leading '<' before think> and trailing '>' from tag remnants so the final reply is clean.
fn parse_think_and_visible(s: &str) -> (String, String) {
    let mut think = String::new();
    let mut visible = String::new();
    let mut rest = s;
    loop {
        if let Some(open_pos) = rest.find(THINK_OPEN) {
            let pre = rest[..open_pos].trim_end();
            let pre = if pre.ends_with('<') {
                pre[..pre.len() - 1].trim_end()
            } else {
                pre
            };
            visible.push_str(pre);
            let after_open = open_pos + THINK_OPEN.len();
            if let Some(close_pos) = rest[after_open..].find(THINK_CLOSE) {
                think.push_str(&rest[after_open..after_open + close_pos]);
                think.push('\n');
                rest = &rest[after_open + close_pos + THINK_CLOSE.len()..];
            } else {
                let remainder = &rest[after_open..];
                if let Some(dbl) = remainder.find("\n\n") {
                    visible.push_str(&remainder[dbl + 2..]);
                }
                break;
            }
        } else {
            visible.push_str(rest);
            break;
        }
    }
    let mut visible = visible.trim().to_string();
    if visible.starts_with('<') {
        visible = visible[1..].trim().to_string();
    }
    if visible.ends_with('>') {
        visible = visible[..visible.len() - 1].trim_end().to_string();
    }
    (think.trim().to_string(), visible)
}

/// One row in the conversation: either a user/assistant pair (left/right), a full-width system/tool/think line, or a round separator.
#[derive(Clone)]
enum TurnRow {
    Pair { user: String, assistant: String },
    FullWidth(String),
    /// Visual separator between conversation rounds; n is the 1-based round index.
    Separator(usize),
}

pub struct MessageView {
    messages: Vec<String>,
    /// When streaming, accumulates the current assistant reply until done.
    streaming_assistant: Option<String>,
    virtual_scroll: VirtualScroll,
}

impl MessageView {
    pub fn new() -> Self {
        Self {
            messages: Vec::new(),
            streaming_assistant: None,
            virtual_scroll: VirtualScroll::new(20),
        }
    }

    pub fn add_message(&mut self, message: String) {
        self.messages.push(message);
        self.virtual_scroll.scroll_to_bottom();
    }

    /// Start accumulating streamed assistant content (call after adding "You: ...").
    pub fn start_streaming_assistant(&mut self) {
        self.streaming_assistant = Some(String::new());
    }

    /// Append a chunk to the current streaming assistant reply.
    pub fn append_streaming_chunk(&mut self, chunk: String) {
        if let Some(ref mut buf) = self.streaming_assistant {
            buf.push_str(&chunk);
        }
    }

    /// Finalize streaming: push "Assistant: {visible_content}" (think blocks stripped) and clear the buffer.
    pub fn finish_streaming_assistant(&mut self) {
        if let Some(content) = self.streaming_assistant.take() {
            if !content.is_empty() {
                let (_, visible) = parse_think_and_visible(&content);
                self.add_message(format!("Assistant: {}", visible));
            }
        }
    }

    pub fn is_streaming(&self) -> bool {
        self.streaming_assistant.is_some()
    }

    pub fn scroll_up(&mut self) {
        self.virtual_scroll.scroll_up(1);
    }

    pub fn scroll_down(&mut self) {
        self.virtual_scroll.scroll_down(1);
    }

    /// Parse flat messages with "You: "/"Assistant: "/"System: "/"Tool: " prefixes into turn rows.
    /// If streaming_assistant is Some, its content is used as the assistant text for the last Pair with empty assistant.
    fn messages_to_turns(&self) -> Vec<TurnRow> {
        let mut turns: Vec<TurnRow> = Vec::new();
        for msg in &self.messages {
            if msg.starts_with("You: ") {
                let content = msg[5..].to_string();
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
        }
        if let Some(ref buf) = self.streaming_assistant {
            if !buf.is_empty() {
                let (think, visible) = parse_think_and_visible(buf);
                let last_is_empty_pair = matches!(
                    turns.last(),
                    Some(TurnRow::Pair { assistant: a, .. }) if a.is_empty()
                );
                if last_is_empty_pair {
                    let last = turns.pop().unwrap();
                    if let TurnRow::Pair { user, .. } = last {
                        if !think.is_empty() {
                            turns.push(TurnRow::FullWidth(format!("AI 思考：{}", think)));
                        }
                        turns.push(TurnRow::Pair { user, assistant: visible });
                    }
                } else {
                    if !think.is_empty() {
                        turns.push(TurnRow::FullWidth(format!("AI 思考：{}", think)));
                    }
                    turns.push(TurnRow::Pair {
                        user: String::new(),
                        assistant: visible,
                    });
                }
            }
        }
        // Insert round separators between Pairs so users can tell "last turn" vs "current turn".
        let mut with_sep: Vec<TurnRow> = Vec::new();
        let mut round_index = 0usize;
        for t in turns {
            if matches!(t, TurnRow::Pair { .. }) {
                round_index += 1;
                if round_index > 1 {
                    with_sep.push(TurnRow::Separator(round_index));
                }
                with_sep.push(t);
            } else {
                with_sep.push(t);
            }
        }
        with_sep
    }

    /// Returns the number of lines needed to wrap `text` at `width` (no truncation).
    fn wrap_text_to_line_count(text: &str, width: u16) -> usize {
        if width == 0 {
            return 0;
        }
        let lines = Self::wrap_text_to_lines_unlimited(text, width, Style::default());
        lines.len().max(if text.trim().is_empty() { 0 } else { 1 })
    }

    /// Wrap text to lines with no line limit; returns all lines for Paragraph.
    fn wrap_text_to_lines_unlimited(text: &str, width: u16, style: Style) -> Vec<Line<'static>> {
        if width == 0 {
            return Vec::new();
        }
        let width_usize = width as usize;
        let mut result: Vec<Line<'static>> = Vec::new();

        fn push_line(s: String, width_usize: usize, result: &mut Vec<Line<'static>>, style: Style) {
            let mut rest = s;
            while !rest.is_empty() {
                let take: String = rest.chars().take(width_usize).collect();
                result.push(Line::from(Span::styled(take, style)));
                rest = rest.chars().skip(width_usize).collect();
            }
        }

        for segment in text.split('\n') {
            let mut line = String::new();
            let mut line_len = 0usize;
            for word in segment.split_whitespace() {
                let need = word.chars().count() + if line.is_empty() { 0 } else { 1 };
                if line_len + need > width_usize && !line.is_empty() {
                    push_line(std::mem::take(&mut line), width_usize, &mut result, style);
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
                push_line(line, width_usize, &mut result, style);
            }
        }
        if result.is_empty() && !text.is_empty() {
            result.push(Line::from(Span::styled("", style)));
        }
        result
    }

    /// Wrap and truncate text to a maximum number of lines; returns lines for Paragraph.
    fn wrap_text_to_lines(
        text: &str,
        width: u16,
        max_lines: u16,
        style: Style,
    ) -> Vec<Line<'static>> {
        let all = Self::wrap_text_to_lines_unlimited(text, width, style);
        let max = max_lines as usize;
        if all.len() <= max {
            all
        } else {
            all.into_iter().take(max).collect()
        }
    }

    /// Compute number of display lines for one turn at the given content widths.
    fn turn_line_count(turn: &TurnRow, user_width: u16, assistant_width: u16, full_width: u16) -> usize {
        match turn {
            TurnRow::Pair { user, assistant } => {
                let u = Self::wrap_text_to_line_count(user, user_width);
                let a = Self::wrap_text_to_line_count(assistant, assistant_width);
                u.max(a).max(MIN_TURN_LINES)
            }
            TurnRow::FullWidth(text) => {
                Self::wrap_text_to_line_count(text, full_width).max(MIN_TURN_LINES)
            }
            TurnRow::Separator(_) => 1,
        }
    }

    /// Build cumulative line starts per turn: turn_line_starts[i] = first global line index of turn i.
    fn turn_line_starts(turns: &[TurnRow], user_width: u16, assistant_width: u16, full_width: u16) -> Vec<usize> {
        let mut starts = vec![0];
        for turn in turns {
            let count = Self::turn_line_count(turn, user_width, assistant_width, full_width);
            starts.push(starts.last().copied().unwrap_or(0) + count);
        }
        starts
    }

    pub fn render(&mut self, f: &mut Frame, area: Rect, theme: &Theme) {
        let block = Block::default()
            .title(vec![
                Span::styled("💬", Style::default().fg(theme.accent)),
                Span::raw(" Messages · Conversation"),
            ])
            .borders(Borders::ALL)
            .border_style(theme.border_style())
            .style(theme.panel_style());

        let inner = block.inner(area);
        f.render_widget(block, area);

        let turns = self.messages_to_turns();
        if turns.is_empty() {
            return;
        }

        const AVATAR_WIDTH: u16 = 2;
        let gap = 1u16;
        let half = (inner.width.saturating_sub(gap)) / 2;
        let user_width = half.saturating_sub(AVATAR_WIDTH);
        let right_content_width = inner.width.saturating_sub(half + gap);
        let assistant_width = right_content_width.saturating_sub(AVATAR_WIDTH);

        let turn_line_starts = Self::turn_line_starts(&turns, user_width, assistant_width, inner.width);
        let total_lines = *turn_line_starts.last().unwrap_or(&0);
        let viewport_height = inner.height as usize;
        self.virtual_scroll.viewport_height = viewport_height.max(1);
        self.virtual_scroll.set_total_items(total_lines);
        let (start_line, end_line) = self.virtual_scroll.visible_range();

        let text_style = Style::default().fg(theme.foreground);
        let user_bubble_style = Style::default()
            .fg(theme.foreground)
            .bg(theme.user_bubble_bg);
        let assistant_bubble_style = Style::default()
            .fg(theme.foreground)
            .bg(theme.assistant_bubble_bg);

        for global_line in start_line..end_line.min(total_lines) {
            let row_y = inner.y + (global_line - start_line) as u16;
            let row_rect = Rect {
                x: inner.x,
                y: row_y,
                width: inner.width,
                height: 1,
            };

            // Find which turn this line belongs to and line index within turn.
            let turn_idx = turn_line_starts
                .iter()
                .position(|&s| s > global_line)
                .map(|p| p - 1)
                .unwrap_or(turns.len() - 1);
            let line_in_turn = global_line - turn_line_starts[turn_idx];
            let turn = &turns[turn_idx];

            match turn {
                TurnRow::Separator(n) => {
                    let label = format!("——— Round {} ———", n);
                    let para = Paragraph::new(label.as_str())
                        .style(Style::default().fg(theme.secondary))
                        .alignment(Alignment::Center);
                    f.render_widget(para, row_rect);
                }
                TurnRow::Pair { user, assistant } => {
                    let user_lines = Self::wrap_text_to_lines_unlimited(user, user_width, text_style);
                    let assistant_lines =
                        Self::wrap_text_to_lines_unlimited(assistant, assistant_width, text_style);
                    let user_line = user_lines.get(line_in_turn).cloned().unwrap_or_else(|| Line::from(Span::raw("")));
                    let assistant_line = assistant_lines
                        .get(line_in_turn)
                        .cloned()
                        .unwrap_or_else(|| Line::from(Span::raw("")));

                    let left_avatar_rect = Rect {
                        x: inner.x,
                        y: row_y,
                        width: AVATAR_WIDTH,
                        height: 1,
                    };
                    let user_bubble_rect = Rect {
                        x: inner.x + AVATAR_WIDTH,
                        y: row_y,
                        width: user_width,
                        height: 1,
                    };
                    let assistant_bubble_rect = Rect {
                        x: inner.x + half + gap,
                        y: row_y,
                        width: assistant_width,
                        height: 1,
                    };
                    let right_avatar_rect = Rect {
                        x: inner.x + half + gap + assistant_width,
                        y: row_y,
                        width: AVATAR_WIDTH,
                        height: 1,
                    };

                    let user_avatar_para = Paragraph::new(if line_in_turn == 0 { "👤" } else { "  " })
                        .style(user_bubble_style)
                        .alignment(Alignment::Center);
                    let assistant_avatar_para =
                        Paragraph::new(if line_in_turn == 0 { "🤖" } else { "  " })
                            .style(assistant_bubble_style)
                            .alignment(Alignment::Center);

                    f.render_widget(user_avatar_para, left_avatar_rect);
                    f.render_widget(
                        Paragraph::new(user_line).style(user_bubble_style),
                        user_bubble_rect,
                    );
                    f.render_widget(
                        Paragraph::new(assistant_line).style(assistant_bubble_style),
                        assistant_bubble_rect,
                    );
                    f.render_widget(assistant_avatar_para, right_avatar_rect);
                }
                TurnRow::FullWidth(text) => {
                    let full_lines = Self::wrap_text_to_lines_unlimited(text, inner.width, text_style);
                    let full_line = full_lines
                        .get(line_in_turn)
                        .cloned()
                        .unwrap_or_else(|| Line::from(Span::raw("")));
                    let style = if text.starts_with("AI 思考：") {
                        Style::default().fg(theme.secondary).bg(theme.panel_bg)
                    } else {
                        Style::default().fg(theme.foreground).bg(theme.panel_bg)
                    };
                    let para = Paragraph::new(full_line).style(style);
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
