use ratatui::prelude::*;
use ratatui::widgets::*;
use std::time::{Duration, Instant};

pub struct Spinner {
    pub frames: Vec<&'static str>,
    pub current_frame: usize,
    last_update: Instant,
    interval: Duration,
}

impl Spinner {
    pub fn new() -> Self {
        Self {
            frames: vec!["⠋", "⠙", "⠹", "⠸", "⠼", "⠴", "⠦", "⠧", "⠇", "⠏"],
            current_frame: 0,
            last_update: Instant::now(),
            interval: Duration::from_millis(100),
        }
    }

    pub fn tick(&mut self) -> bool {
        let now = Instant::now();
        if now.duration_since(self.last_update) >= self.interval {
            self.current_frame = (self.current_frame + 1) % self.frames.len();
            self.last_update = now;
            true
        } else {
            false
        }
    }

    pub fn get_frame(&self) -> &'static str {
        self.frames[self.current_frame]
    }

    pub fn render(&self, f: &mut Frame, area: Rect) {
        let frame_char = self.frames[self.current_frame];
        let paragraph = Paragraph::new(frame_char)
            .block(Block::default().borders(Borders::NONE));
        f.render_widget(paragraph, area);
    }
}

impl Default for Spinner {
    fn default() -> Self {
        Self::new()
    }
}
