//! Scrollable log panel for agent lifecycle and tool-call entries.

use crate::tui::app::{LogEntry, LogLevel};
use crate::tui::components::virtual_scroll::VirtualScroll;
use crate::tui::theme::Theme;
use ratatui::prelude::*;
use ratatui::widgets::*;

const MAX_LINE_LEN: usize = 200;

fn level_prefix(level: LogLevel) -> &'static str {
    match level {
        LogLevel::Info => "[INFO] ",
        LogLevel::Warn => "[WARN] ",
        LogLevel::Error => "[ERR]  ",
    }
}

pub struct LogPanel;

impl LogPanel {
    /// Render log entries in the given area with line-based scrolling.
    pub fn render(
        f: &mut Frame,
        area: Rect,
        entries: &[LogEntry],
        scroll: &mut VirtualScroll,
        theme: &Theme,
    ) {
        let block = Block::default()
            .title(vec![
                Span::styled("📋", Style::default().fg(theme.accent)),
                Span::raw(" Agent log · Tools and lifecycle"),
            ])
            .borders(Borders::ALL)
            .border_style(theme.border_style())
            .style(theme.panel_style());

        let inner = block.inner(area);
        f.render_widget(block, area);

        if entries.is_empty() {
            return;
        }

        let total_lines = entries.len();
        let viewport_height = inner.height as usize;
        scroll.viewport_height = viewport_height.max(1);
        scroll.set_total_items(total_lines);
        let (start, end) = scroll.visible_range();

        let text_style = Style::default().fg(theme.foreground);
        let warn_style = Style::default().fg(theme.warning);
        let error_style = Style::default().fg(theme.error);

        for (i, entry) in entries[start..end.min(total_lines)].iter().enumerate() {
            let y = inner.y + i as u16;
            let prefix = level_prefix(entry.level);
            let line_style = match entry.level {
                LogLevel::Info => text_style,
                LogLevel::Warn => warn_style,
                LogLevel::Error => error_style,
            };
            let mut line = format!("{}{}", prefix, entry.message);
            if line.len() > MAX_LINE_LEN {
                line.truncate(MAX_LINE_LEN);
                line.push_str("…");
            }
            let para = Paragraph::new(line.as_str())
                .style(line_style)
                .wrap(Wrap { trim: false });
            let rect = Rect {
                x: inner.x,
                y,
                width: inner.width,
                height: 1,
            };
            f.render_widget(para, rect);
        }

        scroll.render_scrollbar(f, area);
    }
}
