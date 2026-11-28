//! Virtual UART / Serial Console driver.

use std::fmt;

pub struct VirtualConsole {
    history: Vec<String>,
}

impl Default for VirtualConsole {
    fn default() -> Self {
        Self::new()
    }
}

impl VirtualConsole {
    pub fn new() -> Self {
        Self {
            history: Vec::new(),
        }
    }

    pub fn write_str(&mut self, s: &str) {
        self.history.push(s.to_string());
    }

    pub fn print_line(&mut self, args: fmt::Arguments) {
        let msg = fmt::format(args);
        self.history.push(msg);
    }

    pub fn history(&self) -> &[String] {
        &self.history
    }

    pub fn clear(&mut self) {
        self.history.clear();
    }
}
