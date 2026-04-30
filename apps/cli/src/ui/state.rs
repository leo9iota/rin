#[derive(Default, Debug, Clone, PartialEq, Eq)]
pub enum AppMode {
    #[default]
    Setup,
    Dashboard,
}

use super::forms::SetupFormState;
use rin_core::pipeline::config::ConfigPayload;
use std::collections::VecDeque;
use tokio::sync::mpsc::Sender;

#[derive(Debug)]
pub struct AppState {
    pub mode: AppMode,
    pub setup_form: SetupFormState,
    pub tx: Sender<ConfigPayload>,
    pub logs_fetched: usize,
    pub logs_decoded: usize,
    pub events_inserted: usize,
    pub log_history: VecDeque<String>,
    pub tick_count: usize,
}

impl AppState {
    pub fn new(tx: Sender<ConfigPayload>) -> Self {
        Self {
            mode: AppMode::default(),
            setup_form: SetupFormState::default(),
            tx,
            logs_fetched: 0,
            logs_decoded: 0,
            events_inserted: 0,
            log_history: VecDeque::with_capacity(50),
            tick_count: 0,
        }
    }
}
