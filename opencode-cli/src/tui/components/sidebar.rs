use ratatui::prelude::*;
use ratatui::widgets::*;
use crate::tui::theme::Theme;

pub fn render(f: &mut Frame, area: Rect, theme: &Theme) {
    let items = vec![
        Line::from(Span::styled("OpenCode", theme.primary_style())),
        Line::from(""),
        Line::from(vec![Span::styled("Quick Actions", theme.secondary_style()), Span::raw(":")]),
        Line::from(vec![Span::styled("  • n:", theme.accent_style()), Span::raw(" New session")]),
        Line::from(vec![Span::styled("  • c:", theme.accent_style()), Span::raw(" Config")]),
        Line::from(vec![Span::styled("  • Esc:", theme.accent_style()), Span::raw(" Back to Home")]),
        Line::from(vec![Span::styled("  • q:", theme.accent_style()), Span::raw(" Quit")]),
        Line::from(""),
        Line::from(vec![Span::styled("Providers", theme.secondary_style()), Span::raw(":")]),
        Line::from(vec![Span::styled("  • OpenAI", Style::default().fg(theme.foreground))]),
        Line::from(vec![Span::styled("  • Ollama", Style::default().fg(theme.foreground))]),
        Line::from(vec![Span::styled("  • Qwen", Style::default().fg(theme.foreground))]),
        Line::from(vec![Span::styled("  • Anthropic", Style::default().fg(theme.foreground))]),
        Line::from(""),
        Line::from(vec![Span::styled("Tools", theme.secondary_style()), Span::raw(":")]),
        Line::from(vec![Span::styled("  • read/write", Style::default().fg(theme.foreground))]),
        Line::from(vec![Span::styled("  • grep/ls", Style::default().fg(theme.foreground))]),
        Line::from(vec![Span::styled("  • edit/patch", Style::default().fg(theme.foreground))]),
        Line::from(vec![Span::styled("  • bash", Style::default().fg(theme.foreground))]),
        Line::from(""),
        Line::from(Span::styled("Press '?' for help", theme.secondary_style())),
    ];

    let list = List::new(items)
        .block(
            Block::default()
                .title(vec![Span::styled("☰", theme.accent_style()), Span::raw(" OpenCode")])
                .borders(Borders::ALL)
                .border_style(theme.border_style())
                .style(theme.panel_style()),
        )
        .highlight_style(theme.highlight_style())
        .style(Style::default().fg(theme.foreground));

    f.render_widget(list, area);
}