use crate::input::{Action, InputHandler};

use crossterm::event::{KeyCode, KeyEvent};

pub struct State {
    selected_option: AppOption,
}

impl State {
    pub fn new() -> Self {
        Self {
            selected_option: AppOption::Send,
        }
    }

    pub fn selected_option(&self) -> AppOption {
        self.selected_option
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
            KeyCode::Char('h') => {
                self.selected_option = AppOption::Send;
                None
            }
            KeyCode::Char('l') => {
                self.selected_option = AppOption::Quit;
                None
            }
            KeyCode::Enter => {
                // Handle the selected option
                match self.selected_option {
                    AppOption::Send => Some(Action::Send),
                    AppOption::Quit => Some(Action::Quit),
                }
            }
            _ => None,
        }
    }

    fn process_tick(&mut self) {}
}

#[derive(PartialEq, Clone, Copy)]
pub enum AppOption {
    Send,
    Quit,
}
