use ratatui::prelude::*;
use ratatui::widgets::*;
use crate::tui::theme::Theme;

pub fn render(f: &mut Frame, area: Rect, title: &str, theme: &Theme) {
    let block = Block::default()
        .title(
            Line::from(vec![
                Span::styled(" █ ", Style::default().fg(theme.accent)),
                Span::styled("OpenCode", theme.primary_style()),
                Span::raw(" "),
                Span::styled(title, theme.secondary_style()),
            ])
        )
        .borders(Borders::BOTTOM)
        .border_style(theme.border_style())
        .style(theme.panel_style());
    
    f.render_widget(Clear, area); // Clear the area first
    f.render_widget(block, area);
}