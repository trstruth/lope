use crossterm::event::KeyEvent;

pub enum Action {
    Send,
    Quit,
}

pub trait InputHandler {
    fn process_key(&mut self, input: KeyEvent) -> Option<Action>;
    fn process_tick(&mut self);
}
