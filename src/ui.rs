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
    hover_opened_dash: bool,
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

    pub fn watch() {
        // a read is enough to mark the service as used by this window
        drop(Self::read());
    }

    pub fn toggle() {
        let dash = {
            let mut ui = Self::write();

            ui.dash = !ui.dash;
            ui.hover_opened_dash = false;

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
            ui.hover_opened_dash = false;
        }

        crate::overlay::Overlay::close();
    }

    pub fn close() {
        if !Self::read().dash {
            return;
        }

        let mut ui = Self::write();
        ui.dash = false;
        ui.hover_opened_dash = false;
    }

    pub fn select(tab: Tab) {
        Self::write().tab = tab;
    }

    pub fn set_hover(hover: bool) {
        let auto_open = hover && crate::settings::Settings::read().flag("hover_dash");
        let mut ui = Self::write();

        if ui.hover == hover && (!auto_open || ui.dash) {
            return;
        }

        ui.hover = hover;

        if auto_open && !ui.dash {
            ui.dash = true;
            ui.hover_opened_dash = true;
            drop(ui);
            crate::overlay::Overlay::close();
        } else if !hover && ui.hover_opened_dash {
            ui.dash = false;
            ui.hover_opened_dash = false;
        }
    }

    pub fn refresh() {
        let _ = Self::write();
    }
}
