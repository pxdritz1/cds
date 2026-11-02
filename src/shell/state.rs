use amane::Service;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Overlay {
    Dashboard,
    Launcher,
    ControlCenter,
    Notifications,
    Clipboard,
    Power,
}

pub struct ShellState {
    pub overlay: Option<Overlay>,
    pub revealed: bool,
}

impl Service for ShellState {
    fn new() -> Self {
        Self { overlay: None, revealed: false }
    }
}
