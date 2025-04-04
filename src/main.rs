use anyhow::Context;
use crossterm::{
    event::{DisableMouseCapture, EnableMouseCapture, Event as CEvent},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use std::{env, error::Error, fs, io, path::Path};
use tui::{backend::CrosstermBackend, Terminal};

use lope::{
    app::{input_events, App, Event},
    display::ui,
    input::{Action, InputHandler},
    openai::call_gpt,
};

const TOKEN_PATH: &str = ".sgpt/token";

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    let token = get_token()?;

    // Setup terminal
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen, EnableMouseCapture)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    // Create app state
    let mut app = App::default();

    // Input events via crossbeam
    let rx = input_events();

    let exit_reason: Option<Action>;

    // Main loop
    loop {
        terminal.draw(|f| ui(f, &mut app))?;

        match rx.recv()? {
            Event::Input(event) => {
                if let CEvent::Key(key_event) = event {
                    if let Some(action) = app.process_key(key_event) {
                        match action {
                            Action::Send => {
                                exit_reason = Some(Action::Send);
                                break;
                            }
                            Action::Quit => {
                                exit_reason = Some(Action::Quit);
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

    if let Some(Action::Send) = exit_reason {
        println!("{}", call_gpt(token.as_str(), &app).await?)
    }

    Ok(())
}

// fetch the token from the filesystem
fn get_token() -> Result<String, Box<dyn Error>> {
    let home_dir = env::var("HOME")?;
    let token_path = Path::new(&home_dir).join(TOKEN_PATH);
    let token = fs::read_to_string(&token_path).context(format!(
        "Failed to read token from {}",
        token_path.to_string_lossy()
    ))?;
    let token = token.strip_suffix('\n').unwrap_or(&token);
    Ok(token.to_owned())
}
