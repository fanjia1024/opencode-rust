use ratatui::prelude::*;
use ratatui::widgets::*;

pub fn render(f: &mut Frame, area: Rect, status: &str) {
    let block = Block::default()
        .title(status)
        .borders(Borders::BOTTOM | Borders::LEFT | Borders::RIGHT);
    f.render_widget(block, area);
}
