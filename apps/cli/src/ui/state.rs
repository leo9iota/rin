#[derive(Default, Debug, Clone, PartialEq, Eq)]
pub enum AppMode {
    #[default]
    Setup,
    Dashboard,
}

use super::forms::SetupFormState;
use rin_core::pipeline::config::ConfigPayload;
use tokio::sync::mpsc::Sender;

#[derive(Debug)]
pub struct AppState {
    pub mode: AppMode,
    pub setup_form: SetupFormState,
    pub tx: Sender<ConfigPayload>,
}

impl AppState {
    pub fn new(tx: Sender<ConfigPayload>) -> Self {
        Self {
            mode: AppMode::default(),
            setup_form: SetupFormState::default(),
            tx,
        }
    }
}
