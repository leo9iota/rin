use ratatui::{
    layout::{Constraint, Direction, Layout},
    style::{Color, Style},
    widgets::{Block, Borders, Paragraph},
    Frame,
};

use super::{AppState, AppMode, forms::FocusedField};

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
        ("RPC URL (ws/wss)", &state.setup_form.rpc_url, FocusedField::RpcUrl),
        ("Contract Address", &state.setup_form.contract, FocusedField::Contract),
        ("Event Signature", &state.setup_form.event, FocusedField::Event),
        ("Start Block", &state.setup_form.start_block, FocusedField::StartBlock),
    ];

    for (i, (title, input, field)) in fields.iter().enumerate() {
        let is_active = state.setup_form.focused == *field;
        
        let border_style = if is_active {
            Style::default().fg(Color::Yellow)
        } else {
            Style::default().fg(Color::DarkGray)
        };

        let widget = Paragraph::new(input.value())
            .block(Block::default().borders(Borders::ALL).title(*title).style(border_style));
        
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

fn render_dashboard(f: &mut Frame, _state: &AppState) {
    let block = Block::default()
        .title(" Rin Dashboard ")
        .borders(Borders::ALL)
        .style(Style::default().fg(Color::Green));
    f.render_widget(block, f.area());
}
