use amane::Service;

#[derive(Clone, Copy, PartialEq, Eq, Default)]
pub enum Tab {
    #[default]
    Overview,
    Media,
    Wallpapers,
    Alerts,
    Weather,
    Settings,
}

impl Tab {
    // a stable number for the motion layer, so it can tell tabs apart
    pub fn index(self) -> u64 {
        match self {
            Tab::Overview => 0,
            Tab::Media => 1,
            Tab::Wallpapers => 2,
            Tab::Alerts => 3,
            Tab::Weather => 4,
            Tab::Settings => 5,
        }
    }
}

#[derive(Default)]
pub struct Ui {
    dash: bool,
    tab: Tab,
    hover: bool,
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

    pub fn toggle() {
        let dash = {
            let mut ui = Self::write();

            ui.dash = !ui.dash;

            ui.dash
        };

        // an overlay and the dash would both want the keyboard
        if dash {
            crate::overlay::Overlay::close();
        }
    }

    pub fn open(tab: Tab) {
        {
            let mut ui = Self::write();

            ui.dash = true;
            ui.tab = tab;
        }

        crate::overlay::Overlay::close();
    }

    pub fn close() {
        let mut ui = Self::write();

        ui.dash = false;
    }

    pub fn select(tab: Tab) {
        Self::write().tab = tab;
    }

    pub fn set_hover(hover: bool) {
        if Self::read().hover == hover {
            return;
        }

        Self::write().hover = hover;
    }

    // redraws the windows that show the ui, for input handlers that
    // keep their state outside a service, like the motion layer
    pub fn refresh() {
        let _ = Self::write();
    }
}

pub fn command(arguments: &[String]) -> String {
    let [action, rest @ ..] = arguments else {
        return usage();
    };

    match action.as_str() {
        "toggle" => Ui::toggle(),
        "close" => Ui::close(),
        "open" => Ui::open(parse_tab(rest.first())),
        "tray" => crate::tray::Tray::toggle(),
        _ => return usage(),
    }

    String::from("ok")
}

fn parse_tab(name: Option<&String>) -> Tab {
    match name.map(String::as_str) {
        Some("media") => Tab::Media,
        Some("wallpapers") => Tab::Wallpapers,
        Some("alerts") => Tab::Alerts,
        Some("weather") | Some("clima") => Tab::Weather,
        Some("settings") | Some("ajustes") => Tab::Settings,
        _ => Tab::Overview,
    }
}

fn usage() -> String {
    String::from("usage: island toggle | close | open [overview|media|wallpapers|alerts|clima|ajustes] | tray")
}
