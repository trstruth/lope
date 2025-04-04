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
            crossterm::event::KeyCode::Char(c) => {
                self.prompt_text.push(c);
                None
            }
            crossterm::event::KeyCode::Backspace => {
                self.prompt_text.pop();
                None
            }
            _ => None,
        }
    }

    fn process_tick(&mut self) {}
}
