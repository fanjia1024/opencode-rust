use crossterm::event::KeyCode;
use std::collections::HashMap;

pub type KeyHandler = Box<dyn Fn() -> bool + Send + Sync>;

pub struct KeybindManager {
    bindings: HashMap<KeyCode, Vec<KeyHandler>>,
}

impl KeybindManager {
    pub fn new() -> Self {
        Self {
            bindings: HashMap::new(),
        }
    }

    pub fn register(&mut self, key: KeyCode, handler: KeyHandler) {
        self.bindings.entry(key).or_insert_with(Vec::new).push(handler);
    }

    pub fn handle(&self, key: KeyCode) -> bool {
        if let Some(handlers) = self.bindings.get(&key) {
            for handler in handlers {
                if handler() {
                    return true;
                }
            }
        }
        false
    }
}

impl Default for KeybindManager {
    fn default() -> Self {
        Self::new()
    }
}
