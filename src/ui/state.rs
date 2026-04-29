#[derive(Default, Debug, Clone, PartialEq, Eq)]
pub enum AppMode {
    #[default]
    Setup,
    Dashboard,
}

#[derive(Default, Debug)]
pub struct AppState {
    pub mode: AppMode,
    // Add setup form fields and dashboard state here
}

impl AppState {
    pub fn new() -> Self {
        Self::default()
    }
}
