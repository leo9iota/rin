#[derive(Default, Debug, Clone, PartialEq, Eq)]
pub enum AppMode {
    #[default]
    Setup,
    Dashboard,
}

use super::forms::SetupFormState;

#[derive(Default, Debug)]
pub struct AppState {
    pub mode: AppMode,
    pub setup_form: SetupFormState,
}

impl AppState {
    pub fn new() -> Self {
        Self::default()
    }
}
