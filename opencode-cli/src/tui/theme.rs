use ratatui::style::{Color, Style};

#[derive(Clone)]
pub struct Theme {
    pub background: Color,
    pub foreground: Color,
    pub primary: Color,
    pub secondary: Color,
    pub error: Color,
    pub success: Color,
}

impl Theme {
    pub fn dark() -> Self {
        Self {
            background: Color::Black,
            foreground: Color::White,
            primary: Color::Cyan,
            secondary: Color::Gray,
            error: Color::Red,
            success: Color::Green,
        }
    }

    pub fn light() -> Self {
        Self {
            background: Color::White,
            foreground: Color::Black,
            primary: Color::Blue,
            secondary: Color::DarkGray,
            error: Color::Red,
            success: Color::Green,
        }
    }

    pub fn default_style(&self) -> Style {
        Style::default()
            .fg(self.foreground)
            .bg(self.background)
    }
}

impl Default for Theme {
    fn default() -> Self {
        Self::dark()
    }
}
