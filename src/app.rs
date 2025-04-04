use std::{thread, time::Duration};

use crate::{
    input::{Action, InputHandler},
    widgets::{file_browser, options, prompt_editor},
};

use crossbeam::channel::{unbounded, Receiver};
use crossterm::event::{self, Event as CEvent, KeyCode, KeyModifiers};
use ignore::WalkBuilder;

pub struct App {
    pub selected_widget: Widget,
    pub file_browser_state: file_browser::State,
    pub prompt_editor_state: prompt_editor::State,
    pub options_state: options::State,
}

impl InputHandler for App {
    fn process_key(&mut self, input: crossterm::event::KeyEvent) -> Option<Action> {
        // switch widget if control + arrow key was pressed
        if input.modifiers.contains(KeyModifiers::CONTROL) {
            match input.code {
                KeyCode::Char('h') => {
                    if let Widget::PromptEditor = self.selected_widget {
                        self.selected_widget = Widget::FileBrowser;
                    }
                }
                KeyCode::Char('l') => {
                    if let Widget::FileBrowser = self.selected_widget {
                        self.selected_widget = Widget::PromptEditor;
                    }
                }
                KeyCode::Char('k') => {
                    if let Widget::Options = self.selected_widget {
                        self.selected_widget = Widget::PromptEditor;
                    }
                }
                KeyCode::Char('j') => {
                    self.selected_widget = match self.selected_widget {
                        Widget::PromptEditor | Widget::FileBrowser => Widget::Options,
                        _ => self.selected_widget.clone(),
                    };
                }
                KeyCode::Char('c') => {
                    return Some(Action::Quit);
                }
                _ => {}
            }
            return None;
        }

        match self.selected_widget {
            Widget::FileBrowser => self.file_browser_state.process_key(input),
            Widget::PromptEditor => self.prompt_editor_state.process_key(input),
            Widget::Options => self.options_state.process_key(input),
        }
    }

    fn process_tick(&mut self) {
        match self.selected_widget {
            Widget::FileBrowser => self.file_browser_state.process_tick(),
            Widget::PromptEditor => self.prompt_editor_state.process_tick(),
            Widget::Options => self.options_state.process_tick(),
        }
    }
}

#[derive(Clone, PartialEq)]
pub enum Widget {
    FileBrowser,
    PromptEditor,
    Options,
}

// Event wrapper for crossbeam
pub enum Event<I> {
    Input(I),
    Tick,
}

impl App {
    pub fn new() -> Self {
        let mut entries = Vec::new();

        // Build an iterator with .gitignore rules enabled
        // Increase max_depth if you want deeper traversal
        let walker = WalkBuilder::new(".").standard_filters(true).build();

        for entry in walker.flatten() {
            let depth = entry.depth();
            // If the file_type is None, skip
            let file_type = match entry.file_type() {
                Some(ft) => ft,
                None => continue,
            };

            let is_dir = file_type.is_dir();
            // Convert to string for TUI display
            let path = entry.path().display().to_string();

            entries.push(file_browser::TreeEntry::new(path, depth, is_dir));
        }

        Self {
            selected_widget: Widget::PromptEditor,
            file_browser_state: file_browser::State::new(entries),
            prompt_editor_state: prompt_editor::State::default(),
            options_state: options::State::default(),
        }
    }
}

impl Default for App {
    fn default() -> Self {
        Self::new()
    }
}

// Crossbeam channel to capture input events (keyboard)
pub fn input_events() -> Receiver<Event<CEvent>> {
    let (tx, rx) = unbounded();
    thread::spawn(move || {
        loop {
            // Poll for user input
            if event::poll(Duration::from_millis(50)).unwrap() {
                if let Ok(ev) = event::read() {
                    tx.send(Event::Input(ev)).unwrap();
                }
            }
            tx.send(Event::Tick).unwrap();
        }
    });
    rx
}
