use crossterm::event::KeyCode;

use crate::input::InputHandler;

pub struct State {
    prompt_text: String,
}

impl State {
    pub fn new() -> Self {
        Self {
            prompt_text: String::new(),
        }
    }

    pub fn get_display_text(&self) -> &str {
        &self.prompt_text
    }
}

impl Default for State {
    fn default() -> Self {
        Self::new()
    }
}

impl InputHandler for State {
    fn process_key(&mut self, input: crossterm::event::KeyEvent) -> Option<crate::input::Action> {
        match input.code {
            KeyCode::Char(c) => {
                self.prompt_text.push(c);
                None
            }
            KeyCode::Backspace => {
                self.prompt_text.pop();
                None
            }
            KeyCode::Enter => {
                self.prompt_text.push('\n');
                None
            }
            _ => None,
        }
    }

    fn process_tick(&mut self) {}
}
