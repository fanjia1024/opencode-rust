use ratatui::prelude::*;
use ratatui::widgets::*;

pub struct VirtualScroll {
    pub scroll_offset: usize,
    pub viewport_height: usize,
    pub total_items: usize,
}

impl VirtualScroll {
    pub fn new(viewport_height: usize) -> Self {
        Self {
            scroll_offset: 0,
            viewport_height,
            total_items: 0,
        }
    }

    pub fn set_total_items(&mut self, total: usize) {
        self.total_items = total;
        self.scroll_offset = self.scroll_offset.min(total.saturating_sub(self.viewport_height.max(1)));
    }

    pub fn scroll_up(&mut self, amount: usize) {
        self.scroll_offset = self.scroll_offset.saturating_sub(amount);
    }

    pub fn scroll_down(&mut self, amount: usize) {
        let max_offset = self.total_items.saturating_sub(self.viewport_height.max(1));
        self.scroll_offset = (self.scroll_offset + amount).min(max_offset);
    }

    pub fn scroll_to_top(&mut self) {
        self.scroll_offset = 0;
    }

    pub fn scroll_to_bottom(&mut self) {
        self.scroll_offset = self.total_items.saturating_sub(self.viewport_height.max(1));
    }

    pub fn visible_range(&self) -> (usize, usize) {
        let start = self.scroll_offset;
        let end = (start + self.viewport_height).min(self.total_items);
        (start, end)
    }

    pub fn render_scrollbar(&self, f: &mut Frame, area: Rect) {
        if self.total_items <= self.viewport_height || self.viewport_height == 0 {
            return;
        }

        let scrollbar_area = Rect {
            x: area.x + area.width.saturating_sub(1),
            y: area.y,
            width: 1,
            height: area.height,
        };

        let ratio = self.viewport_height as f64 / self.total_items as f64;
        let scrollbar_height = ((area.height as f64) * ratio).max(1.0) as u16;
        let max_scroll = self.total_items.saturating_sub(self.viewport_height);
        let scroll_ratio = if max_scroll > 0 {
            self.scroll_offset as f64 / max_scroll as f64
        } else {
            0.0
        };
        let scrollbar_position = ((area.height.saturating_sub(scrollbar_height)) as f64 * scroll_ratio) as u16;

        let scrollbar_rect = Rect {
            x: scrollbar_area.x,
            y: scrollbar_area.y + scrollbar_position,
            width: 1,
            height: scrollbar_height.max(1),
        };

        let block = Block::default()
            .style(Style::default().fg(Color::Gray));
        f.render_widget(block, scrollbar_rect);
    }
}
