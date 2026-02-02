use ratatui::prelude::*;
use unicode_width::UnicodeWidthStr;
use syntect::easy::HighlightLines;
use syntect::highlighting::ThemeSet;
use syntect::parsing::SyntaxSet;
use syntect::util::LinesWithEndings;

pub struct SyntaxHighlighter {
    syntax_set: SyntaxSet,
    theme_set: ThemeSet,
    current_theme: String,
}

pub struct StyledGrapheme {
    pub symbol: String,
    pub style: Style,
}

impl StyledGrapheme {
    pub fn width(&self) -> usize {
        self.symbol.width()
    }
}

impl SyntaxHighlighter {
    pub fn new() -> Self {
        let syntax_set = SyntaxSet::load_defaults_newlines();
        let theme_set = ThemeSet::load_defaults();
        
        Self {
            syntax_set,
            theme_set,
            current_theme: "base16-ocean.dark".to_string(),
        }
    }

    pub fn highlight_code(&self, code: &str, language: &str) -> Vec<StyledGrapheme> {
        let syntax = self.syntax_set
            .find_syntax_by_extension(language)
            .or_else(|| self.syntax_set.find_syntax_by_name(language))
            .or_else(|| self.syntax_set.find_syntax_by_extension("txt"))
            .unwrap();

        let theme = &self.theme_set.themes[&self.current_theme];
        let mut highlighter = HighlightLines::new(syntax, theme);

        let mut styled_graphemes = Vec::new();
        for line in LinesWithEndings::from(code) {
            match highlighter.highlight_line(line, &self.syntax_set) {
                Ok(highlighted) => {
                    for (style, text) in highlighted {
                        let fg = self.style_to_color(style.foreground);
                        styled_graphemes.push(StyledGrapheme {
                            symbol: text.to_string(),
                            style: Style::default().fg(fg),
                        });
                    }
                }
                Err(_) => {
                    styled_graphemes.push(StyledGrapheme {
                        symbol: line.to_string(),
                        style: Style::default(),
                    });
                }
            }
        }
        styled_graphemes
    }

    fn style_to_color(&self, color: syntect::highlighting::Color) -> Color {
        Color::Rgb(color.r, color.g, color.b)
    }

    pub fn set_theme(&mut self, theme: &str) {
        if self.theme_set.themes.contains_key(theme) {
            self.current_theme = theme.to_string();
        }
    }
}

impl Default for SyntaxHighlighter {
    fn default() -> Self {
        Self::new()
    }
}
