use ratatui::prelude::*;
use ratatui::widgets::*;

pub fn render(f: &mut Frame, area: Rect) {
    let logo_text = r#"
  ___                   ____            _ 
 / _ \ _ __   ___ _ __ / ___|___   ___ | |
| | | | '_ \ / _ \ '_ \| |   / _ \ / _ \| |
| |_| | |_) |  __/ | | | |__| (_) | (_) | |
 \___/| .__/ \___|_| |_|\____\___/ \___/|_|
      |_|                                  
    "#;

    let paragraph = Paragraph::new(logo_text)
        .block(Block::default().borders(Borders::NONE))
        .style(Style::default().fg(Color::Cyan))
        .alignment(Alignment::Center);

    f.render_widget(paragraph, area);
}
