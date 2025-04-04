use crossbeam::channel::{unbounded, Receiver};
use crossterm::{
    event::{
        self, DisableMouseCapture, EnableMouseCapture, Event as CEvent, KeyCode, KeyModifiers,
    },
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use ignore::WalkBuilder;
use std::{error::Error, io, thread, time::Duration};
use tui::{
    backend::CrosstermBackend,
    layout::{Alignment, Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Span, Spans},
    widgets::{Block, BorderType, Borders, List, ListItem, Paragraph},
    Terminal,
};

use lope::{
    input::{Action, InputHandler},
    theme,
    widgets::{
        file_browser,
        options::{self, AppOption},
        prompt_editor,
    },
};

// App state
struct App {
    selected_widget: Widget,
    file_browser_state: file_browser::State,
    prompt_editor_state: prompt_editor::State,
    options_state: options::State,
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
enum Widget {
    FileBrowser,
    PromptEditor,
    Options,
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
        terminal.draw(|f| ui(f, &mut app))?;

        match rx.recv()? {
            Event::Input(event) => {
                if let CEvent::Key(key_event) = event {
                    if let Some(action) = app.process_key(key_event) {
                        match action {
                            Action::Send => {
                                // Handle sending the prompt
                                println!(
                                    "Sending prompt: {}",
                                    app.prompt_editor_state.get_display_text()
                                );
                            }
                            Action::Quit => {
                                break;
                            }
                        }
                    }
                }
            }
            Event::Tick => app.process_tick(),
        }
    }

    // Restore terminal
    disable_raw_mode()?;
    let stdout = terminal.backend_mut();
    execute!(stdout, LeaveAlternateScreen, DisableMouseCapture)?;
    terminal.show_cursor()?;

    Ok(())
}

// Draw the file tree with indentation
fn draw_file_tree<B: tui::backend::Backend>(f: &mut tui::Frame<B>, app: &mut App, area: Rect) {
    let mut block = Block::default().borders(Borders::ALL).title("File Browser");

    if app.selected_widget == Widget::FileBrowser {
        block = block.border_type(BorderType::Thick)
    }

    let visible_idxs = app.file_browser_state.visible_entries();
    let items: Vec<ListItem> = visible_idxs
        .iter()
        .map(|&idx| {
            let entry = &app.file_browser_state.file_list[idx];
            let indentation = " ".repeat(entry.depth * 2);
            let filename = entry.path.split('/').last().unwrap_or("UNKNOWN");
            let checked_or_not = if entry.excluded { "" } else { "* " };
            ListItem::new(format!("{}{}{}", indentation, checked_or_not, filename))
        })
        .collect();

    // 2. Build the `List` widget
    let list = List::new(items)
        .block(block)
        .style(Style::default().bg(theme::GRAY))
        .highlight_style(
            Style::default()
                .fg(theme::YELLOW) // yellow
                .bg(theme::LIGHT_GREY)
                .add_modifier(Modifier::BOLD),
        )
        .highlight_symbol("> ");

    // 3. Render with the `list_state` to track selection
    f.render_stateful_widget(list, area, &mut app.file_browser_state.list_state);
}

// Draw the prompt editor
fn draw_prompt_editor<B: tui::backend::Backend>(f: &mut tui::Frame<B>, app: &App, area: Rect) {
    let mut block = Block::default()
        .borders(Borders::ALL)
        .title("Prompt Editor");
    if app.selected_widget == Widget::PromptEditor {
        block = block.border_type(BorderType::Thick);
    }
    let paragraph = Paragraph::new(app.prompt_editor_state.get_display_text())
        .block(block)
        .style(Style::default().fg(theme::LIGHT_GREEN).bg(theme::GRAY));
    f.render_widget(paragraph, area);
}

fn draw_bottom_options<B: tui::backend::Backend>(
    f: &mut tui::Frame<B>,
    app: &App,
    area: tui::layout::Rect,
) {
    let mut block = Block::default()
        .borders(Borders::ALL)
        .title("Options")
        .style(Style::default().bg(theme::GRAY));
    if app.selected_widget == Widget::Options {
        block = block.border_type(BorderType::Thick)
    }

    let spans = vec![
        if app.options_state.selected_option() == AppOption::Send {
            Span::styled(
                "[Send]",
                Style::default()
                    .fg(theme::BLUE)
                    .add_modifier(Modifier::REVERSED),
            )
        } else {
            Span::raw("[Send]")
        },
        Span::raw("  "), // spacing
        if app.options_state.selected_option() == AppOption::Quit {
            Span::styled(
                "[Quit]",
                Style::default()
                    .fg(theme::PURPLE)
                    .add_modifier(Modifier::REVERSED),
            )
        } else {
            Span::raw("[Quit]")
        },
    ];

    let paragraph = Paragraph::new(Spans::from(spans))
        .block(block)
        .alignment(Alignment::Center)
        .style(Style::default().fg(Color::Gray));

    f.render_widget(paragraph, area);
}

fn ui<B: tui::backend::Backend>(f: &mut tui::Frame<B>, app: &mut App) {
    // First, split the screen vertically so we can have a thin pane at the bottom
    let vertical_chunks = Layout::default()
        .direction(Direction::Vertical)
        // The top pane takes all remaining space (Min), bottom pane has a fixed height of 3
        .constraints([Constraint::Min(0), Constraint::Length(3)])
        .split(f.size());

    // Now, split the top pane horizontally for the file tree and prompt editor
    let main_chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Percentage(30), Constraint::Percentage(70)])
        .split(vertical_chunks[0]);

    draw_file_tree(f, app, main_chunks[0]);
    draw_prompt_editor(f, app, main_chunks[1]);
    draw_bottom_options(f, app, vertical_chunks[1]);
}
