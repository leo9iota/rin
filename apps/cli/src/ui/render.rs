use ratatui::{
    Frame,
    layout::{Constraint, Direction, Layout},
    style::{Color, Style},
    widgets::{Block, Borders, Paragraph},
};

use super::{AppMode, AppState, forms::FocusedField};

pub fn render(f: &mut Frame, state: &AppState) {
    match state.mode {
        AppMode::Setup => render_setup(f, state),
        AppMode::Dashboard => render_dashboard(f, state),
    }
}

fn render_setup(f: &mut Frame, state: &AppState) {
    let size = f.area();

    let block = Block::default()
        .title(" Rin EVM Indexer Setup ")
        .borders(Borders::ALL)
        .style(Style::default().fg(Color::Cyan));

    // Create a centered area for the form
    let vertical_chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Percentage(30),
            Constraint::Length(16),
            Constraint::Percentage(30),
        ])
        .split(size);

    let horizontal_chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Percentage(20),
            Constraint::Percentage(60),
            Constraint::Percentage(20),
        ])
        .split(vertical_chunks[1]);

    let form_area = horizontal_chunks[1];
    f.render_widget(block, form_area);

    let form_chunks = Layout::default()
        .direction(Direction::Vertical)
        .margin(2)
        .constraints([
            Constraint::Length(3),
            Constraint::Length(3),
            Constraint::Length(3),
            Constraint::Length(3),
        ])
        .split(form_area);

    let fields = [
        (
            "RPC URL (ws/wss)",
            &state.setup_form.rpc_url,
            FocusedField::RpcUrl,
        ),
        (
            "Contract Address",
            &state.setup_form.contract,
            FocusedField::Contract,
        ),
        (
            "Event Signature",
            &state.setup_form.event,
            FocusedField::Event,
        ),
        (
            "Start Block",
            &state.setup_form.start_block,
            FocusedField::StartBlock,
        ),
    ];

    for (i, (title, input, field)) in fields.iter().enumerate() {
        let is_active = state.setup_form.focused == *field;

        let border_style = if is_active {
            Style::default().fg(Color::Yellow)
        } else {
            Style::default().fg(Color::DarkGray)
        };

        let widget = Paragraph::new(input.value()).block(
            Block::default()
                .borders(Borders::ALL)
                .title(*title)
                .style(border_style),
        );

        f.render_widget(widget, form_chunks[i]);

        if is_active {
            // Render cursor
            f.set_cursor_position((
                form_chunks[i].x + input.visual_cursor() as u16 + 1,
                form_chunks[i].y + 1,
            ));
        }
    }
}

use ratatui::widgets::{List, ListItem};

const BORDER_COLOR: Color = Color::Rgb(76, 86, 106);
const TITLE_COLOR: Color = Color::Rgb(238, 237, 231);
const VALUE_COLOR: Color = Color::Rgb(144, 229, 154);
const TEXT_COLOR: Color = Color::Rgb(229, 233, 240);
const HIGHLIGHT_COLOR: Color = Color::Rgb(180, 142, 173);

fn render_dashboard(f: &mut Frame, state: &AppState) {
    let size = f.area();

    // Split main area vertically
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(3),  // Header
            Constraint::Length(5),  // Metrics
            Constraint::Min(5),     // Log Stream
            Constraint::Length(3),  // Footer
        ])
        .split(size);

    // Header Spinner
    let spinner_frames = ["⠋", "⠙", "⠹", "⠸", "⠼", "⠴", "⠦", "⠧", "⠇", "⠏"];
    let spinner_idx = (state.tick_count / 2) % spinner_frames.len();
    let spinner = spinner_frames[spinner_idx];
    
    let header_text = format!(" {} RIN EVM INDEXER [ACTIVE] ", spinner);
    let header = Paragraph::new(header_text)
        .style(Style::default().fg(HIGHLIGHT_COLOR).bg(Color::Reset))
        .block(Block::default().borders(Borders::ALL).border_style(Style::default().fg(BORDER_COLOR)).style(Style::default()));
    f.render_widget(header, chunks[0]);

    // Metrics Row
    let metrics_chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Percentage(33),
            Constraint::Percentage(34),
            Constraint::Percentage(33),
        ])
        .split(chunks[1]);

    let fetch_metric = Paragraph::new(format!("\n  {}", state.logs_fetched))
        .style(Style::default().fg(VALUE_COLOR))
        .block(Block::default().title(" Logs Fetched ").title_style(Style::default().fg(TITLE_COLOR)).borders(Borders::ALL).border_style(Style::default().fg(BORDER_COLOR)));
    f.render_widget(fetch_metric, metrics_chunks[0]);

    let decode_metric = Paragraph::new(format!("\n  {}", state.logs_decoded))
        .style(Style::default().fg(VALUE_COLOR))
        .block(Block::default().title(" Logs Decoded ").title_style(Style::default().fg(TITLE_COLOR)).borders(Borders::ALL).border_style(Style::default().fg(BORDER_COLOR)));
    f.render_widget(decode_metric, metrics_chunks[1]);

    let insert_metric = Paragraph::new(format!("\n  {}", state.events_inserted))
        .style(Style::default().fg(VALUE_COLOR))
        .block(Block::default().title(" Events Inserted ").title_style(Style::default().fg(TITLE_COLOR)).borders(Borders::ALL).border_style(Style::default().fg(BORDER_COLOR)));
    f.render_widget(insert_metric, metrics_chunks[2]);

    // Rolling Log Stream
    let items: Vec<ListItem> = state.log_history.iter().map(|log| {
        ListItem::new(log.as_str()).style(Style::default().fg(TEXT_COLOR))
    }).collect();

    let list = List::new(items)
        .block(Block::default().title(" Real-time Event Stream ").title_style(Style::default().fg(TITLE_COLOR)).borders(Borders::ALL).border_style(Style::default().fg(BORDER_COLOR)));
    
    f.render_widget(list, chunks[2]);

    // Footer
    let footer = Paragraph::new(" Press 'q' to quit ")
        .style(Style::default().fg(BORDER_COLOR))
        .block(Block::default().borders(Borders::ALL).border_style(Style::default().fg(BORDER_COLOR)));
    f.render_widget(footer, chunks[3]);
}
