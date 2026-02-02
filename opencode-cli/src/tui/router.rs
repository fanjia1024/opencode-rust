use crate::tui::state::Screen;

pub struct Router {
    current: Screen,
    history: Vec<Screen>,
}

impl Router {
    pub fn new() -> Self {
        Self {
            current: Screen::Home,
            history: Vec::new(),
        }
    }

    pub fn navigate(&mut self, screen: Screen) {
        self.history.push(self.current.clone());
        self.current = screen;
    }

    pub fn back(&mut self) -> Option<Screen> {
        self.history.pop().map(|prev| {
            std::mem::replace(&mut self.current, prev)
        })
    }

    pub fn current(&self) -> &Screen {
        &self.current
    }
}

impl Default for Router {
    fn default() -> Self {
        Self::new()
    }
}
