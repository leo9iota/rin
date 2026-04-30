use rin_core::{db, indexer};
pub mod ui;
use crossterm::{
    event::{self, DisableMouseCapture, EnableMouseCapture, Event, KeyCode},
    execute,
    terminal::{EnterAlternateScreen, LeaveAlternateScreen, disable_raw_mode, enable_raw_mode},
};
use ratatui::{Terminal, backend::CrosstermBackend};
use std::io;
use std::time::Duration;
use tui_input::backend::crossterm::EventHandler;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // init tracing to a file to avoid TUI corruption
    let log_file = std::fs::File::create("rin.log")?;
    tracing_subscriber::fmt()
        .with_writer(std::sync::Mutex::new(log_file))
        .with_ansi(false)
        .init();
    // init db
    let _database = db::Database::new().await?;

    // init mpsc channel
    let (tx, mut rx) = tokio::sync::mpsc::channel::<rin_core::pipeline::config::ConfigPayload>(32);

    // spawn background worker spawner
    tokio::spawn(async move {
        if let Some(config) = rx.recv().await {
            tracing::info!("Background engine spinning up with config: {:?}", config);

            let fetcher = rin_core::indexer::LogFetcher::new(config);
            match fetcher.fetch_logs().await {
                Ok(logs) => {
                    tracing::info!(count = logs.len(), "Fetched logs from RPC node");
                }
                Err(err) => {
                    tracing::error!(?err, "Log fetching failed");
                }
            }
        }
    });

    // init indexer engine (mock for now)
    let _engine = indexer::IndexerEngine::new();

    // init state machine
    let mut app_state = ui::AppState::new(tx);

    // setup TUI
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen, EnableMouseCapture)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    // TUI event loop
    let mut should_quit = false;
    while !should_quit {
        terminal.draw(|f| ui::render(f, &app_state))?;

        if event::poll(Duration::from_millis(16))?
            && let Event::Key(key) = event::read()?
        {
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
                                // Construct Payload
                                let payload = rin_core::pipeline::config::ConfigPayload {
                                    rpc_url: app_state.setup_form.rpc_url.value().to_string(),
                                    contract_address: app_state
                                        .setup_form
                                        .contract
                                        .value()
                                        .to_string(),
                                    event_signature: app_state
                                        .setup_form
                                        .event
                                        .value()
                                        .to_string(),
                                    start_block: app_state
                                        .setup_form
                                        .start_block
                                        .value()
                                        .parse()
                                        .unwrap_or(0),
                                };
                                // Submit form & transition
                                app_state.tx.send(payload).await?;
                                app_state.mode = ui::AppMode::Dashboard;
                            } else {
                                app_state.setup_form.focused = app_state.setup_form.focused.next();
                            }
                        }
                        _ => {
                            // Forward key to active tui-input field
                            app_state
                                .setup_form
                                .active_input_mut()
                                .handle_event(&Event::Key(key));
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

    // restore terminal
    disable_raw_mode()?;
    execute!(
        terminal.backend_mut(),
        LeaveAlternateScreen,
        DisableMouseCapture
    )?;
    terminal.show_cursor()?;

    Ok(())
}
