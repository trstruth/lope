use crate::input::{Action, InputHandler};

use crossterm::event::{KeyCode, KeyEvent};

pub struct State {
    selected_option: AppOptions,
}

impl State {
    pub fn new() -> Self {
        Self {
            selected_option: AppOptions::Send,
        }
    }

    pub fn get_display_text(&self) -> String {
        "[Send]  [Quit]".to_owned()
    }
}

impl Default for State {
    fn default() -> Self {
        Self::new()
    }
}

impl InputHandler for State {
    fn process_key(&mut self, input: KeyEvent) -> Option<Action> {
        match input.code {
            KeyCode::Left => {
                self.selected_option = AppOptions::Send;
                None
            }
            KeyCode::Right => {
                self.selected_option = AppOptions::Quit;
                None
            }
            KeyCode::Enter => {
                // Handle the selected option
                match self.selected_option {
                    AppOptions::Send => Some(Action::Send),
                    AppOptions::Quit => Some(Action::Quit),
                }
            }
            _ => None,
        }
    }

    fn process_tick(&mut self) {}
}

pub enum AppOptions {
    Send,
    Quit,
}
