use ratatui::style::{Color, Style, Modifier};

#[derive(Clone)]
pub struct Theme {
    pub background: Color,
    pub foreground: Color,
    pub primary: Color,
    pub secondary: Color,
    pub accent: Color,
    pub error: Color,
    pub success: Color,
    pub warning: Color,
    pub border: Color,
    pub highlight: Color,
    pub panel_bg: Color,
}

impl Theme {
    pub fn dark() -> Self {
        Self {
            background: Color::Rgb(15, 15, 20),      // Dark gray-black
            foreground: Color::Rgb(220, 220, 230),  // Light gray
            primary: Color::Rgb(100, 200, 255),     // Light blue
            secondary: Color::Rgb(150, 150, 200),   // Muted purple-blue
            accent: Color::Rgb(255, 150, 100),      // Orange
            error: Color::Rgb(255, 100, 100),       // Red
            success: Color::Rgb(100, 255, 150),     // Green
            warning: Color::Rgb(255, 220, 100),     // Yellow
            border: Color::Rgb(60, 60, 80),         // Darker border color
            highlight: Color::Rgb(255, 215, 100),   // Gold for highlights
            panel_bg: Color::Rgb(25, 25, 35),       // Slightly lighter than background
        }
    }

    pub fn light() -> Self {
        Self {
            background: Color::Rgb(245, 245, 247),  // Very light gray
            foreground: Color::Rgb(30, 30, 40),     // Dark gray
            primary: Color::Rgb(0, 100, 200),       // Deep blue
            secondary: Color::Rgb(100, 100, 150),   // Muted purple
            accent: Color::Rgb(255, 120, 60),       // Orange-red
            error: Color::Rgb(220, 60, 60),         // Red
            success: Color::Rgb(60, 180, 120),      // Green
            warning: Color::Rgb(255, 180, 0),       // Yellow
            border: Color::Rgb(200, 200, 210),      // Light border
            highlight: Color::Rgb(255, 180, 0),     // Gold
            panel_bg: Color::Rgb(235, 235, 240),    // Light panel background
        }
    }

    pub fn default_style(&self) -> Style {
        Style::default()
            .fg(self.foreground)
            .bg(self.background)
    }

    pub fn primary_style(&self) -> Style {
        Style::default()
            .fg(self.primary)
            .add_modifier(Modifier::BOLD)
    }

    pub fn secondary_style(&self) -> Style {
        Style::default()
            .fg(self.secondary)
    }

    pub fn accent_style(&self) -> Style {
        Style::default()
            .fg(self.accent)
    }

    pub fn highlight_style(&self) -> Style {
        Style::default()
            .fg(self.highlight)
            .add_modifier(Modifier::BOLD)
    }

    pub fn border_style(&self) -> Style {
        Style::default()
            .fg(self.border)
    }

    pub fn panel_style(&self) -> Style {
        Style::default()
            .bg(self.panel_bg)
    }

    pub fn warning_style(&self) -> Style {
        Style::default()
            .fg(self.warning)
    }
}

impl Default for Theme {
    fn default() -> Self {
        Self::dark()
    }
}