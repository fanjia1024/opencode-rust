use ratatui::style::{Color, Modifier, Style};

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
    /// High-contrast background for user message bubbles (left column).
    pub user_bubble_bg: Color,
    /// High-contrast background for assistant message bubbles (right column).
    pub assistant_bubble_bg: Color,
}

impl Theme {
    pub fn dark() -> Self {
        Self {
            background: Color::Rgb(15, 15, 20),          // Dark gray-black
            foreground: Color::Rgb(220, 220, 230),       // Light gray
            primary: Color::Rgb(100, 200, 255),          // Light blue
            secondary: Color::Rgb(150, 150, 200),        // Muted purple-blue
            accent: Color::Rgb(255, 150, 100),           // Orange
            error: Color::Rgb(255, 100, 100),            // Red
            success: Color::Rgb(100, 255, 150),          // Green
            warning: Color::Rgb(255, 220, 100),          // Yellow
            border: Color::Rgb(60, 60, 80),              // Darker border color
            highlight: Color::Rgb(255, 215, 100),        // Gold for highlights
            panel_bg: Color::Rgb(25, 25, 35),            // Slightly lighter than background
            user_bubble_bg: Color::Rgb(35, 45, 65),      // Blue-tinted for user
            assistant_bubble_bg: Color::Rgb(35, 40, 50), // Gray-blue for assistant
        }
    }

    pub fn light() -> Self {
        Self {
            background: Color::Rgb(245, 245, 247),     // Very light gray
            foreground: Color::Rgb(30, 30, 40),        // Dark gray
            primary: Color::Rgb(0, 100, 200),          // Deep blue
            secondary: Color::Rgb(100, 100, 150),      // Muted purple
            accent: Color::Rgb(255, 120, 60),          // Orange-red
            error: Color::Rgb(220, 60, 60),            // Red
            success: Color::Rgb(60, 180, 120),         // Green
            warning: Color::Rgb(255, 180, 0),          // Yellow
            border: Color::Rgb(200, 200, 210),         // Light border
            highlight: Color::Rgb(255, 180, 0),        // Gold
            panel_bg: Color::Rgb(235, 235, 240),       // Light panel background
            user_bubble_bg: Color::Rgb(220, 228, 245), // Light blue tint
            assistant_bubble_bg: Color::Rgb(232, 232, 238), // Light gray
        }
    }

    /// High-contrast dark theme: near-white foreground on near-black background for readability.
    pub fn high_contrast_dark() -> Self {
        Self {
            background: Color::Rgb(12, 12, 16),          // Near black
            foreground: Color::Rgb(248, 248, 252),       // Near white, main text
            primary: Color::Rgb(100, 200, 255),          // Light blue
            secondary: Color::Rgb(220, 220, 230),        // Bright gray, section labels
            accent: Color::Rgb(255, 150, 100),           // Orange
            error: Color::Rgb(255, 100, 100),            // Red
            success: Color::Rgb(100, 255, 150),          // Green
            warning: Color::Rgb(255, 220, 100),          // Yellow
            border: Color::Rgb(80, 80, 96),              // Visible border
            highlight: Color::Rgb(255, 215, 100),        // Gold for highlights
            panel_bg: Color::Rgb(24, 24, 28),            // Slightly lighter than background
            user_bubble_bg: Color::Rgb(32, 42, 58),      // Blue-tinted, high contrast
            assistant_bubble_bg: Color::Rgb(38, 40, 48), // Gray-blue, high contrast
        }
    }

    pub fn default_style(&self) -> Style {
        Style::default().fg(self.foreground).bg(self.background)
    }

    pub fn primary_style(&self) -> Style {
        Style::default()
            .fg(self.primary)
            .add_modifier(Modifier::BOLD)
    }

    pub fn secondary_style(&self) -> Style {
        Style::default().fg(self.secondary)
    }

    pub fn accent_style(&self) -> Style {
        Style::default().fg(self.accent)
    }

    pub fn highlight_style(&self) -> Style {
        Style::default()
            .fg(self.highlight)
            .add_modifier(Modifier::BOLD)
    }

    pub fn border_style(&self) -> Style {
        Style::default().fg(self.border)
    }

    pub fn panel_style(&self) -> Style {
        Style::default().bg(self.panel_bg)
    }

    pub fn warning_style(&self) -> Style {
        Style::default().fg(self.warning)
    }
}

impl Default for Theme {
    fn default() -> Self {
        Self::high_contrast_dark()
    }
}
