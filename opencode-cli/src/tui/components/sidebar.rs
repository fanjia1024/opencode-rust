use ratatui::prelude::*;
use ratatui::widgets::*;

pub fn render(f: &mut Frame, area: Rect) {
    let block = Block::default()
        .title("Sidebar")
        .borders(Borders::ALL);
    f.render_widget(block, area);
}
