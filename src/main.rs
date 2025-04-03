use crossbeam::channel::{unbounded, Receiver};
use crossterm::{
    event::{self, DisableMouseCapture, EnableMouseCapture, Event as CEvent, KeyCode},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use ignore::WalkBuilder;
use std::{error::Error, io, thread, time::Duration};
use tui::{
    backend::CrosstermBackend,
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Style},
    widgets::{Block, Borders, Paragraph},
    Terminal,
};

// Each tree entry has its path, depth (for indentation), and whether it's a dir
struct TreeEntry {
    path: String,
    depth: usize,
    entry_type: EntryType,
    excluded: bool,
}

enum EntryType {
    Directory(DirectoryState),
    File,
}

struct DirectoryState {
    expanded: bool,
}

// App state
struct App {
    file_list: Vec<TreeEntry>,
    selected_file_idx: usize,
    prompt_text: String,
}

// Event wrapper for crossbeam
enum Event<I> {
    Input(I),
    Tick,
}

impl App {
    fn new() -> Self {
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

            entries.push(TreeEntry {
                path,
                depth,
                entry_type: match is_dir {
                    true => EntryType::Directory(DirectoryState { expanded: true }),
                    false => EntryType::File,
                },
                excluded: false,
            });
        }

        Self {
            file_list: entries,
            selected_file_idx: 0,
            prompt_text: String::new(),
        }
    }

    fn on_up(&mut self) {
        if self.selected_file_idx > 0 {
            self.selected_file_idx -= 1;
        }
    }

    fn on_down(&mut self) {
        if self.selected_file_idx < self.file_list.len().saturating_sub(1) {
            self.selected_file_idx += 1;
        }
    }

    fn on_right(&mut self) {
        if let Some(entry) = self.file_list.get_mut(self.selected_file_idx) {
            if let EntryType::Directory(ref mut dir_state) = entry.entry_type {
                dir_state.expanded = true;
            }
        }
    }

    fn on_left(&mut self) {
        if let Some(entry) = self.file_list.get_mut(self.selected_file_idx) {
            if let EntryType::Directory(ref mut dir_state) = entry.entry_type {
                dir_state.expanded = false;
            }
        }
    }

    fn on_key(&mut self, c: char) {
        self.prompt_text.push(c);
    }

    fn on_backspace(&mut self) {
        self.prompt_text.pop();
    }
}

// Crossbeam channel to capture input events (keyboard)
fn input_events() -> Receiver<Event<CEvent>> {
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

fn main() -> Result<(), Box<dyn Error>> {
    // Setup terminal
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen, EnableMouseCapture)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    // Create app state
    let mut app = App::new();

    // Input events via crossbeam
    let rx = input_events();

    // Main loop
    loop {
        terminal.draw(|f| ui(f, &app))?;

        match rx.recv()? {
            Event::Input(event) => {
                if let CEvent::Key(key_event) = event {
                    match key_event.code {
                        KeyCode::Char('q') => {
                            // Exit on 'q'
                            break;
                        }
                        KeyCode::Up => app.on_up(),
                        KeyCode::Down => app.on_down(),
                        KeyCode::Right => app.on_right(),
                        KeyCode::Left => app.on_left(),
                        KeyCode::Char(c) => app.on_key(c),
                        KeyCode::Backspace => app.on_backspace(),
                        _ => {}
                    }
                }
            }
            Event::Tick => {
                // Periodic tasks if needed
            }
        }
    }

    // Restore terminal
    disable_raw_mode()?;
    let stdout = terminal.backend_mut();
    execute!(stdout, LeaveAlternateScreen, DisableMouseCapture)?;
    terminal.show_cursor()?;

    Ok(())
}

// UI layout
fn ui<B: tui::backend::Backend>(f: &mut tui::Frame<B>, app: &App) {
    let chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Percentage(30), Constraint::Percentage(70)])
        .split(f.size());

    draw_file_tree(f, app, chunks[0]);
    draw_prompt_editor(f, app, chunks[1]);
}

// Draw the file tree with indentation
fn draw_file_tree<B: tui::backend::Backend>(f: &mut tui::Frame<B>, app: &App, area: Rect) {
    let mut collapsed_stack = Vec::new();
    let mut lines = Vec::new();

    for (idx, entry) in app.file_list.iter().enumerate() {
        // Pop from the stack if we've gone back above a collapsed directory's depth.
        while let Some(&collapsed_depth) = collapsed_stack.last() {
            if entry.depth <= collapsed_depth {
                collapsed_stack.pop();
            } else {
                break;
            }
        }

        // If we're still within a collapsed parent, skip
        if !collapsed_stack.is_empty() {
            continue;
        }

        // Otherwise, we render this entry
        let indentation = " ".repeat(entry.depth * 2);
        let filename = entry.path.split('/').last().unwrap_or("UNKNOWN");
        let line = if idx == app.selected_file_idx {
            format!("> {} {}", indentation, filename)
        } else {
            format!("  {} {}", indentation, filename)
        };
        lines.push(line);

        // If *this* entry is a collapsed directory, push its depth
        if let EntryType::Directory(ref dir_state) = entry.entry_type {
            if !dir_state.expanded {
                collapsed_stack.push(entry.depth);
            }
        }
    }

    let text = lines.join("\n");

    let block = Block::default().borders(Borders::ALL).title("File Tree");
    let paragraph = Paragraph::new(text)
        .block(block)
        .style(Style::default().fg(Color::White));
    f.render_widget(paragraph, area);
}

// Draw the prompt editor
fn draw_prompt_editor<B: tui::backend::Backend>(f: &mut tui::Frame<B>, app: &App, area: Rect) {
    let block = Block::default()
        .borders(Borders::ALL)
        .title("Prompt Editor");
    let paragraph = Paragraph::new(app.prompt_text.as_ref())
        .block(block)
        .style(Style::default().fg(Color::Yellow));
    f.render_widget(paragraph, area);
}
