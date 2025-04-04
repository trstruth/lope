use crossterm::event::{KeyCode, KeyEvent};
use tui::widgets::ListState;

use crate::input::InputHandler;

pub struct State {
    pub file_list: Vec<TreeEntry>,
    pub list_state: ListState,
}

impl State {
    pub fn new(entries: Vec<TreeEntry>) -> Self {
        let mut list_state = ListState::default();
        list_state.select(Some(0));
        Self {
            file_list: entries,
            list_state,
        }
    }

    fn get_selected_idx(&mut self) -> Option<usize> {
        let visible = self.visible_entries();
        // Find the offset selected in the "visible" slice
        let selected_offset = match self.list_state.selected() {
            Some(offset) => offset,
            None => {
                self.list_state.select(Some(0));
                return visible.first().copied();
            }
        };
        // Return the real index in the file_list
        visible.get(selected_offset).copied()
    }

    fn increment_selected(&mut self) {
        let visible = self.visible_entries();
        if let Some(offset) = self.list_state.selected() {
            if offset < visible.len().saturating_sub(1) {
                self.list_state.select(Some(offset + 1));
            }
        } else {
            self.list_state.select(Some(0));
        }
    }

    fn decrement_selected(&mut self) {
        let visible = self.visible_entries();
        if let Some(offset) = self.list_state.selected() {
            if offset > 0 {
                self.list_state.select(Some(offset - 1));
            }
        } else {
            self.list_state
                .select(Some(visible.len().saturating_sub(1)));
        }
    }

    pub fn visible_entries(&self) -> Vec<usize> {
        let mut visible = Vec::new();
        let mut collapsed_stack = Vec::new();

        for (idx, entry) in self.file_list.iter().enumerate() {
            // if we're inside a collapsed ancestor, skip
            while let Some(&collapsed_depth) = collapsed_stack.last() {
                if entry.depth <= collapsed_depth {
                    collapsed_stack.pop();
                } else {
                    // still inside collapsed parent
                    break;
                }
            }
            if !collapsed_stack.is_empty() {
                continue; // skip
            }

            // this entry is visible
            visible.push(idx);

            // if this entry is a collapsed directory, note its depth
            if let EntryType::Directory(dir_state) = &entry.entry_type {
                if !dir_state.expanded {
                    collapsed_stack.push(entry.depth);
                }
            }
        }
        visible
    }
}

impl InputHandler for State {
    fn process_key(&mut self, input: KeyEvent) -> Option<crate::input::Action> {
        match input.code {
            KeyCode::Up => {
                self.decrement_selected();
            }
            KeyCode::Down => {
                self.increment_selected();
            }
            KeyCode::Right => {
                let selected_idx = self.get_selected_idx().unwrap_or(0);
                if let Some(entry) = self.file_list.get_mut(selected_idx) {
                    if let EntryType::Directory(ref mut dir_state) = entry.entry_type {
                        dir_state.expanded = true;
                    }
                }
            }
            KeyCode::Left => {
                let selected_idx = self.get_selected_idx().unwrap_or(0);
                if let Some(entry) = self.file_list.get_mut(selected_idx) {
                    if let EntryType::Directory(ref mut dir_state) = entry.entry_type {
                        dir_state.expanded = false;
                    }
                }
            }
            _ => {}
        }
        None
    }

    fn process_tick(&mut self) {}
}

// Each tree entry has its path, depth (for indentation), and whether it's a dir
pub struct TreeEntry {
    pub path: String,
    pub depth: usize,
    pub entry_type: EntryType,
    pub excluded: bool,
}

impl TreeEntry {
    pub fn new(path: String, depth: usize, is_dir: bool) -> Self {
        Self {
            path,
            depth,
            entry_type: match is_dir {
                true => EntryType::Directory(DirectoryState { expanded: true }),
                false => EntryType::File,
            },
            excluded: false,
        }
    }
}

pub enum EntryType {
    Directory(DirectoryState),
    File,
}

pub struct DirectoryState {
    expanded: bool,
}
