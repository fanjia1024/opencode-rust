use ratatui::prelude::*;
use ratatui::widgets::*;

pub fn render_horizontal_border(f: &mut Frame, area: Rect, style: Style) {
    let block = Block::default()
        .borders(Borders::TOP | Borders::BOTTOM)
        .border_style(style);
    f.render_widget(block, area);
}

pub fn render_vertical_border(f: &mut Frame, area: Rect, style: Style) {
    let block = Block::default()
        .borders(Borders::LEFT | Borders::RIGHT)
        .border_style(style);
    f.render_widget(block, area);
}

pub fn render_separator(f: &mut Frame, area: Rect, style: Style) {
    let line = Line::from("─".repeat(area.width as usize))
        .style(style);
    let paragraph = Paragraph::new(line);
    f.render_widget(paragraph, area);
}
