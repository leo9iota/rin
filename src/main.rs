pub mod indexer;
pub mod ui;
pub mod db;

use crossterm::{
    event::{self, DisableMouseCapture, EnableMouseCapture, Event, KeyCode},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use ratatui::{backend::CrosstermBackend, Terminal};
use std::io;
use std::time::Duration;
use tui_input::backend::crossterm::EventHandler;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // 1. Initialize Database
    let _database = db::Database::new().await?;

    // 2. Initialize mpsc channel
    let (tx, mut _rx) = tokio::sync::mpsc::channel::<String>(100);

    // 3. Initialize Indexer Engine
    let _engine = indexer::IndexerEngine::new();

    // 4. Initialize State Machine
    let mut app_state = ui::AppState::new();

    // 5. Setup Terminal
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen, EnableMouseCapture)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    // 6. UI Event Loop
    let mut should_quit = false;
    while !should_quit {
        terminal.draw(|f| ui::render(f, &app_state))?;

        if event::poll(Duration::from_millis(16))? {
            if let Event::Key(key) = event::read()? {
                match app_state.mode {
                    ui::AppMode::Setup => {
                        match key.code {
                            KeyCode::Esc => should_quit = true,
                            KeyCode::Tab => {
                                app_state.setup_form.focused = app_state.setup_form.focused.next();
                            }
                            KeyCode::BackTab => {
                                app_state.setup_form.focused = app_state.setup_form.focused.prev();
                            }
                            KeyCode::Enter => {
                                if app_state.setup_form.focused == ui::FocusedField::StartBlock {
                                    // Submit form & transition
                                    tx.send("Form Submitted".into()).await?;
                                    app_state.mode = ui::AppMode::Dashboard;
                                } else {
                                    app_state.setup_form.focused = app_state.setup_form.focused.next();
                                }
                            }
                            _ => {
                                // Forward key to active tui-input field
                                app_state.setup_form.active_input_mut().handle_event(&Event::Key(key));
                            }
                        }
                    }
                    ui::AppMode::Dashboard => {
                        if let KeyCode::Char('q') = key.code {
                            should_quit = true;
                        }
                    }
                }
            }
        }
    }

    // 7. Restore Terminal
    disable_raw_mode()?;
    execute!(
        terminal.backend_mut(),
        LeaveAlternateScreen,
        DisableMouseCapture
    )?;
    terminal.show_cursor()?;

    Ok(())
}

