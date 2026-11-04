use amane::Service;
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::{mpsc, OnceLock};
use std::thread;
use std::time::{Duration, Instant};

static HOVER_GENERATION: AtomicU64 = AtomicU64::new(0);
static HOVER_TIMER: OnceLock<mpsc::Sender<(u64, u64)>> = OnceLock::new();

fn hover_timer() -> &'static mpsc::Sender<(u64, u64)> {
    HOVER_TIMER.get_or_init(|| {
        let (sender, receiver) = mpsc::channel();

        if let Err(error) = thread::Builder::new()
            .name(String::from("cds-hover-timer"))
            .spawn(move || {
                while let Ok((mut generation, mut delay)) = receiver.recv() {
                    let mut deadline = Instant::now() + Duration::from_millis(delay);

                    loop {
                        match receiver.recv_timeout(deadline.saturating_duration_since(Instant::now())) {
                            Ok((next_generation, next_delay)) => {
                                generation = next_generation;
                                delay = next_delay;
                                deadline = Instant::now() + Duration::from_millis(delay);
                            }
                            Err(mpsc::RecvTimeoutError::Timeout) => {
                                if HOVER_GENERATION.load(Ordering::Relaxed) == generation {
                                    let should_open = {
                                        let mut ui = Ui::write();

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
                                }

                                break;
                            }
                            Err(mpsc::RecvTimeoutError::Disconnected) => return,
                        }
                    }
                }
            })
        {
            eprintln!("CDS: não foi possível iniciar o temporizador de hover: {error}");
        }

        sender
    })
}

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
        // Avoid notifying every window when the sheet is already closed.
        if !Self::read().media_sheet {
            return;
        }

        Self::write().media_sheet = false;
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
            // One reusable worker debounces pointer movement. Rapid enter/leave
            // events replace the pending deadline instead of spawning sleeping threads.
            let _ = hover_timer().send((generation, delay));
        }
    }

    pub fn refresh() {
        let _ = Self::write();
    }
}
