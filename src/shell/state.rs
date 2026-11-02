use amane::Service;

pub struct ShellState {
    pub dash_open: bool,
}

impl Service for ShellState {
    fn new() -> Self {
        Self { dash_open: false }
    }
}
