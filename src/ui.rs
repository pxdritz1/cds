use amane::Service;

#[derive(Clone, Copy, PartialEq, Eq, Default)]
pub enum Tab {
    #[default]
    Home,
    Media,
    Weather,
    Wallpapers,
    Alerts,
}

impl Tab {
    pub fn index(self) -> u64 {
        match self {
            Tab::Home => 0,
            Tab::Media => 1,
            Tab::Weather => 2,
            Tab::Wallpapers => 3,
            Tab::Alerts => 4,
        }
    }

    pub fn parse(name: Option<&str>) -> Tab {
        match name {
            Some("media") => Tab::Media,
            Some("weather") | Some("clima") => Tab::Weather,
            Some("wallpapers") | Some("papeis") => Tab::Wallpapers,
            Some("alerts") | Some("alertas") | Some("notifications") => Tab::Alerts,
            _ => Tab::Home,
        }
    }
}

#[derive(Default)]
pub struct Ui {
    dash: bool,
    tab: Tab,
    hover: bool,
    month: i32,
}

impl Service for Ui {
    fn new() -> Self {
        Self::default()
    }

    fn listen() {}
}

impl Ui {
    pub fn dash(&self) -> bool {
        self.dash
    }

    pub fn tab(&self) -> Tab {
        self.tab
    }

    pub fn hover(&self) -> bool {
        self.hover
    }

    pub fn month(&self) -> i32 {
        self.month
    }

    pub fn watch() {
        // a read is enough to mark the service as used by this window
        drop(Self::read());
    }

    pub fn toggle() {
        let dash = {
            let mut ui = Self::write();

            ui.dash = !ui.dash;
            ui.month = 0;

            ui.dash
        };

        if dash {
            crate::overlay::Overlay::close();
        }
    }

    pub fn open(tab: Tab) {
        {
            let mut ui = Self::write();

            ui.dash = true;
            ui.tab = tab;
            ui.month = 0;
        }

        crate::overlay::Overlay::close();
    }

    pub fn close() {
        if !Self::read().dash {
            return;
        }

        Self::write().dash = false;
    }

    pub fn select(tab: Tab) {
        Self::write().tab = tab;
    }

    pub fn shift_month(delta: i32) {
        let mut ui = Self::write();

        ui.month += delta;
    }

    pub fn reset_month() {
        Self::write().month = 0;
    }

    pub fn set_hover(hover: bool) {
        if Self::read().hover == hover {
            return;
        }

        Self::write().hover = hover;
    }

    pub fn refresh() {
        let _ = Self::write();
    }
}
