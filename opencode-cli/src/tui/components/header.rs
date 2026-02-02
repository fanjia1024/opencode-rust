use ratatui::prelude::*;
use ratatui::widgets::*;

pub fn render(f: &mut Frame, area: Rect, title: &str) {
    let block = Block::default()
        .title(title)
        .borders(Borders::TOP | Borders::LEFT | Borders::RIGHT);
    f.render_widget(block, area);
}
