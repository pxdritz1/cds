use amane::Service;
use std::sync::atomic::{AtomicU64, Ordering};
use std::time::Duration;

static HOVER_GENERATION: AtomicU64 = AtomicU64::new(0);

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
    media_sheet: bool,
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

    pub fn media_sheet(&self) -> bool {
        self.media_sheet
    }

    pub fn toggle_media_sheet() {
        {
            let mut ui = Self::write();
            ui.media_sheet = !ui.media_sheet;
        }
        Self::refresh();
    }

    pub fn reset_media_sheet() {
        let mut ui = Self::write();
        ui.media_sheet = false;
    }

    pub fn watch() {
        // a read is enough to mark the service as used by this window
        drop(Self::read());
    }

    pub fn toggle() {
        HOVER_GENERATION.fetch_add(1, Ordering::Relaxed);
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
        HOVER_GENERATION.fetch_add(1, Ordering::Relaxed);
        {
            let mut ui = Self::write();

            ui.dash = true;
            ui.tab = tab;
            ui.hover_opened_dash = false;
        }

        crate::overlay::Overlay::close();
    }

    pub fn close() {
        HOVER_GENERATION.fetch_add(1, Ordering::Relaxed);
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
        let auto_open = hover
            && crate::settings::Settings::read().flag("hover_dash")
            && !crate::island::notification_preview_active();
        let generation = HOVER_GENERATION.fetch_add(1, Ordering::Relaxed) + 1;

        {
            let mut ui = Self::write();

            if ui.hover == hover && (!auto_open || ui.dash) {
                return;
            }

            ui.hover = hover;

            if !hover && ui.hover_opened_dash {
                ui.dash = false;
                ui.hover_opened_dash = false;
            }
        }

        if auto_open {
            let delay = crate::settings::Settings::read().hover_delay_ms();
            std::thread::spawn(move || {
                std::thread::sleep(Duration::from_millis(delay));
                if HOVER_GENERATION.load(Ordering::Relaxed) != generation {
                    return;
                }

                let should_open = {
                    let mut ui = Self::write();
                    if ui.hover && !ui.dash {
                        ui.dash = true;
                        ui.hover_opened_dash = true;
                        true
                    } else {
                        false
                    }
                };

                if should_open {
                    crate::overlay::Overlay::close();
                }
            });
        }
    }

    pub fn refresh() {
        let _ = Self::write();
    }
}
