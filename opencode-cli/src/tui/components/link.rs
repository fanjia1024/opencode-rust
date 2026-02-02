use ratatui::prelude::*;
use ratatui::widgets::*;

pub struct Link {
    text: String,
    url: String,
    hovered: bool,
}

impl Link {
    pub fn new(text: String, url: String) -> Self {
        Self {
            text,
            url,
            hovered: false,
        }
    }

    pub fn render(&self, f: &mut Frame, area: Rect) {
        let style = if self.hovered {
            Style::default()
                .fg(Color::Blue)
                .add_modifier(Modifier::UNDERLINED)
        } else {
            Style::default().fg(Color::Blue)
        };

        let text = format!("{} ({})", self.text, self.url);
        let paragraph = Paragraph::new(text.as_str())
            .style(style);
        
        f.render_widget(paragraph, area);
    }

    pub fn set_hovered(&mut self, hovered: bool) {
        self.hovered = hovered;
    }

    pub fn url(&self) -> &str {
        &self.url
    }
}
