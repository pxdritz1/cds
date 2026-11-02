use amane::Service;

#[derive(Clone, Copy, PartialEq, Eq, Default)]
pub enum Tab {
    #[default]
    Overview,
    Media,
    Wallpapers,
    Alerts,
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
        let mut ui = Self::write();

        ui.dash = !ui.dash;
    }

    pub fn open(tab: Tab) {
        let mut ui = Self::write();

        ui.dash = true;
        ui.tab = tab;
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
}

pub fn command(arguments: &[String]) -> String {
    let [action, rest @ ..] = arguments else {
        return usage();
    };

    match action.as_str() {
        "toggle" => Ui::toggle(),
        "close" => Ui::close(),
        "open" => Ui::open(parse_tab(rest.first())),
        _ => return usage(),
    }

    String::from("ok")
}

fn parse_tab(name: Option<&String>) -> Tab {
    match name.map(String::as_str) {
        Some("media") => Tab::Media,
        Some("wallpapers") => Tab::Wallpapers,
        Some("alerts") => Tab::Alerts,
        _ => Tab::Overview,
    }
}

fn usage() -> String {
    String::from("usage: island toggle | close | open [overview|media|wallpapers|alerts]")
}
