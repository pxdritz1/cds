use std::collections::HashMap;
use iced::widget::{button, column, container, mouse_area, row, scrollable, text, text_input};
use iced::{
    Alignment, Background, Color, Element, Event, Length, Subscription, Task, event, keyboard,
    window,
};
use iced_exwlshell::daemon;
use iced_exwlshell::reexport::{
    Anchor, KeyboardInteractivity, Layer, LayerSize, NewLayerShellSettings, OutputOption,
};
use iced_exwlshell::settings::{ExWlSettings, LayerShellSettings, StartMode};
use iced_exwlshell::to_exwlshell_message;
use iced_wayland_subscriber::shell::{ShellEvent, ShellInfo, ShellReceiver, ShellType};
use iced_runtime::task;
use iced_runtime::window::Action as WindowAction;
use niri_ipc::state::{EventStreamState, EventStreamStatePart};
use niri_ipc::Event as NiriEvent;

use crate::ipc::{self, IpcCommand, IpcEnvelope, IpcResponse};
use crate::services::desktop_apps::DesktopApp;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Panel {
    Launcher,
    Apps,
    Spotlight,
    Clipboard,
    Power,
    Control,
    Settings,
    Color,
    Emoji,
    Dash,
    Tray,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
enum DashTab {
    #[default]
    Home,
    Media,
    Weather,
    Wallpapers,
    Alerts,
}

#[derive(Debug, Clone, Copy)]
enum WindowKind {
    Bar,
    Panel(Panel),
    Lock,
    Osd,
}

#[to_exwlshell_message]
#[derive(Debug, Clone)]
enum Message {
    Ipc(IpcEnvelope),
    NiriEvent(NiriEvent),
    FocusWorkspace(u8),
    Wayland(ShellEvent),
    Event(Event),
    WindowClosed(window::Id),
    OpenPanel(Panel),
    OpenDash(DashTab),
    SelectDashTab(DashTab),
    CloseActive,
    CloseWindow(window::Id),
    ShowOsd(String),
    LockAppeared(window::Id),
    AppsQuery(String),
    LauncherQuery(String),
    AppsSearchResults(Vec<String>),
    AppsMove(isize),
    TogglePinned(String),
    SettingsToggleFlag(String),
    SettingsSetDark(bool),
    SettingsSetLanguage(String),
    SettingsSetMotion(String),
    SettingsSetAlignment(String, String),
    AudioRefresh,
    AudioSnapshot(Result<crate::services::audio::AudioSnapshot, String>),
    AudioAdjust(i8),
    AudioToggleOutputMute,
    AudioToggleInputMute,
    AudioActionResult(Result<(), String>),
    AudioSetVolume(u8),
    SystemRefresh,
    SystemSnapshot(Result<crate::services::system::SystemSnapshot, String>),
    WifiToggle(bool),
    BluetoothToggle(bool),
    BrightnessAdjust(i8),
    SystemActionResult(Result<(), String>),
    TrayRefresh,
    TraySnapshot(Result<Vec<crate::services::tray::TrayItemSnapshot>, String>),
    TrayMenuSnapshot(Result<crate::services::tray::TrayMenuSnapshot, String>),
    TrayMenuActivate(i32),
    TrayMenuDismiss,
    TrayActivate(String),
    TrayContextMenu(String),
    TrayActionResult(Result<(), String>),
    BackgroundTick,
    MediaRefresh,
    MediaSnapshot(Result<crate::services::media::MediaSnapshot, String>),
    MediaControl(String),
    MediaControlResult(Result<(), String>),
    NotificationsRefresh,
    NotificationDismiss(u32),
    NotificationDismissed(Result<(), String>),
    LaunchSelectedApp,
    LaunchApp(String),
    AppLaunchFinished(Result<(), String>),
    EmojiQuery(String),
    CopyEmoji(String),
    ClipboardActionFinished(Result<(), String>),
    ShowColorPicker,
    CancelColorPicker,
    ColorChanged(Color),
    ColorSelected(Color),
    ColorHexChanged(String),
    CopyColor,
    Eyedropper,
    EyedropperResult(Result<String, String>),
    ThemeGenerated(Result<crate::services::theme::ThemeTokens, String>),
    ClipboardQuery(String),
    ClipboardMove(isize),
    CopyClipboard(String),
    ClearClipboard,
    LockPasswordChanged(String),
    UnlockRequested,
    UnlockResult(bool),
}

struct App {
    windows: HashMap<window::Id, WindowKind>,
    active_panel: Option<window::Id>,
    dash_tab: DashTab,
    shell_events: ShellReceiver,
    niri_state: EventStreamState,
    apps: Vec<DesktopApp>,
    settings: crate::services::settings::Settings,
    audio: crate::services::audio::AudioSnapshot,
    audio_loaded: bool,
    system: crate::services::system::SystemSnapshot,
    system_loaded: bool,
    media: crate::services::media::MediaSnapshot,
    notifications: Vec<crate::services::notifications::NotificationRecord>,
    tray_items: Vec<crate::services::tray::TrayItemSnapshot>,
    tray_menu: Option<crate::services::tray::TrayMenuSnapshot>,
    app_query: String,
    app_results: Vec<String>,
    app_selected: usize,
    emoji_query: String,
    color_picker_open: bool,
    selected_color: Color,
    color_hex: String,
    last_error: Option<String>,
    theme_tokens: Option<crate::services::theme::ThemeTokens>,
    clipboard_history: crate::services::clipboard::ClipboardHistory,
    clipboard_query: String,
    clipboard_selected: usize,
    lock_password: String,
    lock_error: Option<String>,
    osd_window: Option<window::Id>,
    osd_text: String,
    osd_since: Option<std::time::Instant>,
}

impl App {
    fn new(shell_events: ShellReceiver) -> Self {
        let apps = crate::services::desktop_apps::scan();
        let settings = crate::services::settings::Settings::load();
        let mut app_results = apps.iter().map(|app| app.id.clone()).collect::<Vec<_>>();
        app_results.sort_by_key(|id| {
            apps.iter().find(|app| &app.id == id)
                .map(|app| !settings.is_pinned(&app.pin_key()))
                .unwrap_or(true)
        });

        Self {
            windows: HashMap::new(),
            active_panel: None,
            dash_tab: DashTab::Home,
            shell_events,
            niri_state: EventStreamState::default(),
            apps,
            settings,
            audio: crate::services::audio::AudioSnapshot::default(),
            audio_loaded: false,
            system: crate::services::system::SystemSnapshot::default(),
            system_loaded: false,
            media: crate::services::media::MediaSnapshot::default(),
            notifications: Vec::new(),
            tray_items: Vec::new(),
            tray_menu: None,
            app_query: String::new(),
            app_results,
            app_selected: 0,
            emoji_query: String::new(),
            color_picker_open: false,
            selected_color: Color::from_rgb(0.40, 0.32, 0.65),
            color_hex: "#6750A4".into(),
            last_error: None,
            theme_tokens: crate::services::theme::ThemeTokens::load().ok().flatten(),
            clipboard_history: crate::services::clipboard::ClipboardHistory::load(),
            clipboard_query: String::new(),
            clipboard_selected: 0,
            lock_password: String::new(),
            lock_error: None,
            osd_window: None,
            osd_text: String::new(),
            osd_since: None,
        }
    }

    fn namespace() -> String {
        "Chloe Dynamic Shell".into()
    }

    fn subscription(&self) -> Subscription<Message> {
        let ipc = Subscription::run(crate::ipc::subscribe).map(Message::Ipc);
        let niri = Subscription::run(crate::services::niri::subscribe).map(Message::NiriEvent);

        let background = iced::time::every(std::time::Duration::from_secs(2))
            .map(|_| Message::BackgroundTick);
        Subscription::batch(vec![
            ipc,
            event::listen().map(Message::Event),
            window::close_events().map(Message::WindowClosed),
            self.shell_events.listen().map(Message::Wayland),
            niri,
            background,
        ])
    }

    fn update(&mut self, message: Message) -> Task<Message> {
        match message {
            Message::Ipc(envelope) => {
                let (response, task) = self.handle_ipc(envelope.request.command);
                let _ = envelope.reply.send_blocking(response);
                task
            }
            Message::NiriEvent(event) => {
                let _ = self.niri_state.apply(event);
                Task::none()
            }
            Message::FocusWorkspace(index) => {
                crate::services::niri::focus_workspace(index);
                Task::none()
            }
            Message::Wayland(event) => {
                match event {
                    ShellEvent::NewShell(info) => self.register_shell(info),
                    ShellEvent::Closed(id) => self.remove_window(id),
                    ShellEvent::OutputAdded(_) | ShellEvent::OutputUpdated(_) | ShellEvent::OutputRemoved(_) => {}
                    _ => {}
                }
                Task::none()
            }
            Message::Event(event) => match event {
                Event::Keyboard(keyboard::Event::KeyPressed {
                    key: keyboard::Key::Named(keyboard::key::Named::Escape),
                    ..
                }) => self.close_active(),
                Event::Keyboard(keyboard::Event::KeyPressed {
                    key: keyboard::Key::Named(keyboard::key::Named::ArrowDown),
                    ..
                }) if self.active_panel_is(Panel::Clipboard) => Task::done(Message::ClipboardMove(1)),
                Event::Keyboard(keyboard::Event::KeyPressed {
                    key: keyboard::Key::Named(keyboard::key::Named::Tab),
                    ..
                }) if self.active_panel_is(Panel::Launcher) => Task::done(Message::AppsMove(1)),
                Event::Keyboard(keyboard::Event::KeyPressed {
                    key: keyboard::Key::Named(keyboard::key::Named::ArrowDown),
                    ..
                }) if self.active_apps_panel() => Task::done(Message::AppsMove(
                    if self.active_panel_is(Panel::Apps) { 6 } else { 1 }
                )),
                Event::Keyboard(keyboard::Event::KeyPressed {
                    key: keyboard::Key::Named(keyboard::key::Named::ArrowUp),
                    ..
                }) if self.active_panel_is(Panel::Clipboard) => Task::done(Message::ClipboardMove(-1)),
                Event::Keyboard(keyboard::Event::KeyPressed {
                    key: keyboard::Key::Named(keyboard::key::Named::ArrowUp),
                    ..
                }) if self.active_apps_panel() => Task::done(Message::AppsMove(
                    if self.active_panel_is(Panel::Apps) { -6 } else { -1 }
                )),
                Event::Keyboard(keyboard::Event::KeyPressed {
                    key: keyboard::Key::Named(keyboard::key::Named::Enter),
                    ..
                }) if self.active_panel_is(Panel::Clipboard) => {
                    let selected = self.clipboard_history
                        .search(&self.clipboard_query, crate::services::clipboard::HISTORY_LIMIT)
                        .get(self.clipboard_selected)
                        .cloned();
                    selected.map_or_else(Task::none, |value| Task::done(Message::CopyClipboard(value)))
                }
                Event::Keyboard(keyboard::Event::KeyPressed {
                    key: keyboard::Key::Named(keyboard::key::Named::ArrowLeft),
                    ..
                }) if self.active_apps_panel() => Task::done(Message::AppsMove(-1)),
                Event::Keyboard(keyboard::Event::KeyPressed {
                    key: keyboard::Key::Named(keyboard::key::Named::ArrowRight),
                    ..
                }) if self.active_apps_panel() => Task::done(Message::AppsMove(1)),
                Event::Keyboard(keyboard::Event::KeyPressed {
                    key: keyboard::Key::Named(keyboard::key::Named::Enter),
                    ..
                }) if self.active_apps_panel() => Task::done(Message::LaunchSelectedApp),
                _ => Task::none(),
            },
            Message::WindowClosed(id) => {
                self.remove_window(id);
                if self.osd_window == Some(id) { self.osd_window = None; self.osd_since = None; }
                Task::none()
            }
            Message::ShowOsd(message) => {
                let previous = self.osd_window.take();
                let id = window::Id::unique();
                self.windows.insert(id, WindowKind::Osd);
                self.osd_window = Some(id);
                self.osd_text = message;
                self.osd_since = Some(std::time::Instant::now());
                let settings = NewLayerShellSettings {
                    anchor: Anchor::Bottom,
                    layer: Layer::Overlay,
                    size: LayerSize::px(340, 96),
                    exclusive_zone: None,
                    margin: Some((0, 0, 36, 0)),
                    keyboard_interactivity: KeyboardInteractivity::None,
                    output_option: OutputOption::LastOutput,
                    namespace: Some("cds-osd".into()),
                    ..Default::default()
                };
                let close_previous = previous.map(|previous| Task::done(Message::CloseWindow(previous))).unwrap_or_else(Task::none);
                Task::batch(vec![close_previous, Task::done(Message::NewLayerShell { settings, id })])
            }
            Message::OpenPanel(panel) => self.open_panel(panel),
            Message::OpenDash(tab) => {
                self.dash_tab = tab;
                self.open_panel(Panel::Dash)
            }
            Message::SelectDashTab(tab) => {
                self.dash_tab = tab;
                Task::none()
            }
            Message::CloseActive => self.close_active(),
            Message::CloseWindow(id) => {
                task::effect(iced_runtime::Action::Window(WindowAction::Close(id)))
            }
            Message::LockAppeared(id) => {
                self.lock_password.clear();
                self.lock_error = None;
                self.windows.insert(id, WindowKind::Lock);
                Task::none()
            }
            Message::LockPasswordChanged(password) => {
                self.lock_password = password;
                Task::none()
            }
            Message::UnlockRequested => {
                if self.lock_password.is_empty() {
                    return Task::none();
                }
                let password = std::mem::take(&mut self.lock_password);
                let username = std::env::var("USER").unwrap_or_default();
                Task::perform(
                    async move {
                        tokio::task::spawn_blocking(move || {
                            let password = zeroize::Zeroizing::new(password);
                            let Ok(mut client) = pam::Client::with_password("login") else {
                                return false;
                            };
                            client
                                .conversation_mut()
                                .set_credentials(&username, password.as_str());
                            client.authenticate().is_ok()
                        })
                        .await
                        .unwrap_or(false)
                    },
                    Message::UnlockResult,
                )
            }
            Message::UnlockResult(authenticated) => {
                if authenticated {
                    self.lock_error = None;
                    Task::done(Message::UnLock)
                } else {
                    self.lock_error = Some("Senha incorreta ou autenticação indisponível.".into());
                    Task::none()
                }
            }
            Message::LauncherQuery(query) => {
                self.app_query = query.clone();
                self.app_selected = 0;
                let apps = self.apps.clone();
                Task::perform(
                    async move {
                        tokio::task::spawn_blocking(move || {
                            let needle = query.trim().to_lowercase();
                            apps.iter().filter(|app| {
                                needle.is_empty()
                                    || app.name.to_lowercase().contains(&needle)
                                    || app.description.to_lowercase().contains(&needle)
                            }).map(|app| app.id.clone()).collect::<Vec<_>>()
                        }).await.unwrap_or_default()
                    },
                    Message::AppsSearchResults,
                )
            }
            Message::AppsQuery(query) => {
                self.app_query = query.clone();
                self.app_selected = 0;
                let apps = self.apps.clone();
                Task::perform(
                    async move {
                        tokio::task::spawn_blocking(move || {
                            crate::services::desktop_apps::search(&apps, &query, 1000)
                                .into_iter()
                                .map(|app| app.id.clone())
                                .collect::<Vec<_>>()
                        })
                        .await
                        .unwrap_or_default()
                    },
                    Message::AppsSearchResults,
                )
            }
            Message::AppsSearchResults(mut results) => {
                results.sort_by_key(|id| {
                    self.apps.iter().find(|app| &app.id == id)
                        .map(|app| !self.settings.is_pinned(&app.pin_key()))
                        .unwrap_or(true)
                });
                self.app_results = results;
                self.app_selected = 0;
                Task::none()
            }
            Message::TogglePinned(key) => {
                if let Err(error) = self.settings.toggle_pinned(&key) {
                    self.last_error = Some(error.to_string());
                } else {
                    self.app_results.sort_by_key(|id| {
                        self.apps.iter().find(|app| &app.id == id)
                            .map(|app| !self.settings.is_pinned(&app.pin_key()))
                            .unwrap_or(true)
                    });
                    self.app_selected = 0;
                }
                Task::none()
            }
            Message::SettingsToggleFlag(name) => {
                let value = !self.settings.flag(&name);
                if let Err(error) = self.settings.set_flag(&name, value) {
                    self.last_error = Some(error.to_string());
                }
                Task::none()
            }
            Message::SettingsSetDark(dark) => {
                self.settings.dark = dark;
                if let Err(error) = self.settings.save() { self.last_error = Some(error.to_string()); }
                Task::none()
            }
            Message::SettingsSetLanguage(language) => {
                self.settings.language = if language == "en" { "en" } else { "pt-BR" }.into();
                if let Err(error) = self.settings.save() { self.last_error = Some(error.to_string()); }
                Task::none()
            }
            Message::SettingsSetMotion(motion) => {
                self.settings.motion_style = if motion == "fluent" { "fluent" } else { "m3" }.into();
                if let Err(error) = self.settings.save() { self.last_error = Some(error.to_string()); }
                Task::none()
            }
            Message::SettingsSetAlignment(target, value) => {
                let value = match (target.as_str(), value.as_str()) {
                    ("workspaces", "right") => "right",
                    ("workspaces", _) => "left",
                    ("center", "left") => "left",
                    ("center", "right") => "right",
                    ("center", _) => "center",
                    ("status", "left") => "left",
                    ("status", _) => "right",
                    _ => return Task::none(),
                }.to_owned();
                match target.as_str() {
                    "workspaces" => self.settings.workspaces_alignment = value,
                    "center" => self.settings.center_alignment = value,
                    "status" => self.settings.status_alignment = value,
                    _ => {}
                }
                if let Err(error) = self.settings.save() { self.last_error = Some(error.to_string()); }
                Task::none()
            }
            Message::AudioRefresh => Task::perform(
                async {
                    tokio::task::spawn_blocking(crate::services::audio::snapshot)
                        .await.map_err(|error| error.to_string())?
                        .map_err(|error| error.to_string())
                },
                Message::AudioSnapshot,
            ),
            Message::AudioSnapshot(result) => match result {
                Ok(snapshot) => {
                    let changed = self.audio_loaded && (snapshot.output_volume, snapshot.output_muted) != (self.audio.output_volume, self.audio.output_muted);
                    let mic_changed = self.audio_loaded && snapshot.input_muted != self.audio.input_muted;
                    self.audio = snapshot.clone();
                    self.audio_loaded = true;
                    if changed {
                        Task::done(Message::ShowOsd(format!(
                            "{} · {}%",
                            if snapshot.output_muted { "Áudio silenciado" } else { "Volume" },
                            snapshot.output_volume
                        )))
                    } else if mic_changed {
                        Task::done(Message::ShowOsd(if snapshot.input_muted { "Microfone silenciado".into() } else { "Microfone ativo".into() }))
                    } else { Task::none() }
                }
                Err(error) => { self.last_error = Some(error); Task::none() }
            },
            Message::AudioAdjust(delta) => Task::perform(
                async move {
                    tokio::task::spawn_blocking(move || crate::services::audio::adjust_output(delta))
                        .await.map_err(|error| error.to_string())?
                        .map_err(|error| error.to_string())
                },
                Message::AudioActionResult,
            ),
            Message::AudioToggleOutputMute => Task::perform(
                async { tokio::task::spawn_blocking(crate::services::audio::toggle_output_mute).await.map_err(|error| error.to_string())?.map_err(|error| error.to_string()) },
                Message::AudioActionResult,
            ),
            Message::AudioToggleInputMute => Task::perform(
                async { tokio::task::spawn_blocking(crate::services::audio::toggle_input_mute).await.map_err(|error| error.to_string())?.map_err(|error| error.to_string()) },
                Message::AudioActionResult,
            ),
            Message::AudioActionResult(result) => match result {
                Ok(()) => Task::done(Message::AudioRefresh),
                Err(error) => { self.last_error = Some(error); Task::none() }
            },
            Message::AudioSetVolume(volume) => Task::perform(
                async move {
                    tokio::task::spawn_blocking(move || crate::services::audio::set_output_volume(volume))
                        .await.map_err(|error| error.to_string())?
                        .map_err(|error| error.to_string())
                },
                Message::AudioActionResult,
            ),
            Message::SystemRefresh => Task::perform(
                async {
                    tokio::task::spawn_blocking(crate::services::system::snapshot)
                        .await.map_err(|error| error.to_string())?
                        .map_err(|error| error.to_string())
                },
                Message::SystemSnapshot,
            ),
            Message::SystemSnapshot(result) => match result {
                Ok(snapshot) => {
                    let brightness_changed = self.system_loaded && snapshot.brightness != self.system.brightness;
                    self.system = snapshot.clone();
                    self.system_loaded = true;
                    if brightness_changed {
                        Task::done(Message::ShowOsd(format!("Brilho · {}%", snapshot.brightness)))
                    } else { Task::none() }
                }
                Err(error) => { self.last_error = Some(error); Task::none() }
            },
            Message::WifiToggle(enabled) => Task::perform(
                async move { tokio::task::spawn_blocking(move || crate::services::system::set_wifi(enabled)).await.map_err(|error| error.to_string())?.map_err(|error| error.to_string()) },
                Message::SystemActionResult,
            ),
            Message::BluetoothToggle(enabled) => Task::perform(
                async move { tokio::task::spawn_blocking(move || crate::services::system::set_bluetooth(enabled)).await.map_err(|error| error.to_string())?.map_err(|error| error.to_string()) },
                Message::SystemActionResult,
            ),
            Message::BrightnessAdjust(delta) => Task::perform(
                async move { tokio::task::spawn_blocking(move || crate::services::system::adjust_brightness(delta)).await.map_err(|error| error.to_string())?.map_err(|error| error.to_string()) },
                Message::SystemActionResult,
            ),
            Message::SystemActionResult(result) => match result {
                Ok(()) => Task::batch(vec![Task::done(Message::SystemRefresh), Task::done(Message::AudioRefresh)]),
                Err(error) => { self.last_error = Some(error); Task::none() }
            },
            Message::TrayRefresh => Task::perform(
                async { crate::services::tray::snapshot().await.map_err(|error| error.to_string()) },
                Message::TraySnapshot,
            ),
            Message::TraySnapshot(result) => match result {
                Ok(items) => { self.tray_items = items; Task::none() }
                Err(error) => { tracing::debug!(%error, "Tray metadata refresh failed"); Task::none() }
            },
            Message::TrayActivate(address) => Task::perform(
                async move { crate::services::tray::activate(&address).await.map_err(|error| error.to_string()) },
                Message::TrayActionResult,
            ),
            Message::TrayContextMenu(address) => {
                let menu_path = self.tray_items.iter()
                    .find(|item| item.address == address)
                    .map(|item| item.menu_path.clone())
                    .unwrap_or_default();
                if menu_path.is_empty() {
                    Task::perform(
                        async move { crate::services::tray::context_menu(&address).await.map_err(|error| error.to_string()) },
                        Message::TrayActionResult,
                    )
                } else {
                    Task::perform(
                        async move { crate::services::tray::menu(&address, &menu_path).await.map_err(|error| error.to_string()) },
                        Message::TrayMenuSnapshot,
                    )
                }
            }
            Message::TrayMenuSnapshot(result) => match result {
                Ok(menu) => { self.tray_menu = Some(menu); Task::none() }
                Err(error) => { self.last_error = Some(error); Task::none() }
            }
            Message::TrayMenuActivate(id) => {
                if let Some(menu) = self.tray_menu.take() {
                    Task::perform(
                        async move { crate::services::tray::menu_event(&menu.service, &menu.path, id).await.map_err(|error| error.to_string()) },
                        Message::TrayActionResult,
                    )
                } else { Task::none() }
            }
            Message::TrayMenuDismiss => {
                self.tray_menu = None;
                Task::none()
            }
            Message::TrayActionResult(result) => match result {
                Ok(()) => Task::none(),
                Err(error) => { self.last_error = Some(error); Task::none() }
            },
            Message::BackgroundTick => {
                let mut tasks = vec![
                    Task::done(Message::MediaRefresh),
                    Task::done(Message::NotificationsRefresh),
                    Task::done(Message::AudioRefresh),
                    Task::done(Message::SystemRefresh),
                ];
                if self.active_panel_is(Panel::Tray) {
                    tasks.push(Task::done(Message::TrayRefresh));
                }
                if self.osd_since.is_some_and(|since| since.elapsed() >= std::time::Duration::from_millis(2600)) {
                    if let Some(id) = self.osd_window.take() {
                        self.osd_since = None;
                        tasks.push(Task::done(Message::CloseWindow(id)));
                    }
                }
                Task::batch(tasks)
            },
            Message::MediaRefresh => Task::perform(
                crate::services::media::snapshot(),
                |result| Message::MediaSnapshot(result.map_err(|error| error.to_string())),
            ),
            Message::MediaSnapshot(result) => match result {
                Ok(snapshot) => { self.media = snapshot; Task::none() }
                Err(error) => { tracing::debug!(%error, "MPRIS refresh unavailable"); Task::none() }
            },
            Message::MediaControl(action) => Task::perform(
                async move { crate::services::media::control(&action).await.map_err(|error| error.to_string()) },
                Message::MediaControlResult,
            ),
            Message::MediaControlResult(result) => match result {
                Ok(()) => Task::done(Message::MediaRefresh),
                Err(error) => { self.last_error = Some(error); Task::none() }
            },
            Message::NotificationsRefresh => {
                self.notifications = crate::services::notifications::snapshot();
                Task::none()
            }
            Message::NotificationDismiss(id) => Task::perform(
                crate::services::notifications::dismiss_remote(id)
                    .await
                    .map_err(|error| error.to_string()),
                Message::NotificationDismissed,
            ),
            Message::NotificationDismissed(result) => {
                if let Err(error) = result { self.last_error = Some(error); }
                self.notifications = crate::services::notifications::snapshot();
                Task::none()
            }
            Message::AppsMove(delta) => {
                let count = self.app_results.len();
                if count > 0 {
                    let next = self.app_selected as isize + delta;
                    self.app_selected = if self.active_panel_is(Panel::Launcher) {
                        next.clamp(0, count.saturating_sub(1) as isize) as usize
                    } else {
                        next.rem_euclid(count as isize) as usize
                    };
                }
                Task::none()
            }
            Message::LaunchSelectedApp => {
                self.app_results
                    .get(self.app_selected)
                    .cloned()
                    .map_or_else(Task::none, |id| Task::done(Message::LaunchApp(id)))
            }
            Message::LaunchApp(id) => {
                let Some(app) = self.apps.iter().find(|app| app.id == id).cloned() else {
                    return Task::none();
                };
                Task::perform(
                    async move { app.launch().map_err(|error| error.to_string()) },
                    Message::AppLaunchFinished,
                )
            }
            Message::AppLaunchFinished(result) => match result {
                Ok(()) => self.close_active(),
                Err(error) => {
                    self.last_error = Some(error);
                    Task::none()
                }
            },
            Message::EmojiQuery(query) => {
                self.emoji_query = query;
                Task::none()
            }
            Message::CopyEmoji(glyph) => Task::perform(
                async move {
                    crate::services::emoji::copy_text(&glyph)
                        .await
                        .map_err(|error| error.to_string())
                },
                Message::ClipboardActionFinished,
            ),
            Message::ClipboardActionFinished(result) => match result {
                Ok(()) => self.close_active(),
                Err(error) => {
                    self.last_error = Some(error);
                    Task::none()
                }
            },
            Message::ShowColorPicker => {
                self.color_picker_open = true;
                Task::none()
            }
            Message::CancelColorPicker => {
                self.color_picker_open = false;
                Task::none()
            }
            Message::ColorChanged(color) => {
                self.selected_color = color;
                self.color_hex = color_to_hex(color);
                Task::none()
            }
            Message::ColorSelected(color) => {
                self.selected_color = color;
                self.color_hex = color_to_hex(color);
                self.color_picker_open = false;
                Task::none()
            }
            Message::ColorHexChanged(hex) => {
                self.color_hex = hex.clone();
                if let Some(color) = parse_hex_color(&hex) {
                    self.selected_color = color;
                }
                Task::none()
            }
            Message::CopyColor => iced::clipboard::write(self.color_hex.clone()),
            Message::Eyedropper => Task::perform(
                async move {
                    let status = tokio::process::Command::new("hyprpicker")
                        .args(["-a", "-f", "hex"])
                        .status()
                        .await
                        .map_err(|error| error.to_string())?;
                    if !status.success() {
                        return Err(format!("hyprpicker exited with status {status}"));
                    }
                    let output = tokio::process::Command::new("wl-paste")
                        .arg("--no-newline")
                        .output()
                        .await
                        .map_err(|error| error.to_string())?;
                    String::from_utf8(output.stdout)
                        .map(|value| value.trim().to_owned())
                        .map_err(|error| error.to_string())
                },
                Message::EyedropperResult,
            ),
            Message::EyedropperResult(result) => match result {
                Ok(hex) => {
                    if let Some(color) = parse_hex_color(&hex) {
                        self.selected_color = color;
                        self.color_hex = color_to_hex(color);
                        self.last_error = None;
                    } else {
                        self.last_error = Some(format!("Invalid color returned by hyprpicker: {hex}"));
                    }
                    Task::none()
                }
                Err(error) => {
                    self.last_error = Some(error);
                    Task::none()
                }
            },
            Message::ClipboardQuery(query) => {
                self.clipboard_query = query;
                self.clipboard_selected = 0;
                Task::none()
            }
            Message::ClipboardMove(delta) => {
                let count = self.clipboard_history.search(&self.clipboard_query, crate::services::clipboard::HISTORY_LIMIT).len();
                if count > 0 {
                    let last = count.saturating_sub(1) as isize;
                    self.clipboard_selected = (self.clipboard_selected as isize + delta)
                        .clamp(0, last) as usize;
                }
                Task::none()
            }
            Message::CopyClipboard(value) => Task::perform(
                async move {
                    crate::services::clipboard::copy_text(&value)
                        .await
                        .map_err(|error| error.to_string())
                },
                Message::ClipboardActionFinished,
            ),
            Message::ClearClipboard => {
                if let Err(error) = self.clipboard_history.clear() {
                    self.last_error = Some(error.to_string());
                } else {
                    self.clipboard_selected = 0;
                    self.last_error = None;
                }
                Task::none()
            }
            Message::ThemeGenerated(result) => match result {
                Ok(theme) => {
                    self.theme_tokens = Some(theme);
                    self.last_error = None;
                    Task::none()
                }
                Err(error) => {
                    self.last_error = Some(error);
                    Task::none()
                }
            },
            _ => Task::none(),
        }
    }

    fn handle_ipc(&mut self, command: IpcCommand) -> (IpcResponse, Task<Message>) {
        match command {
            IpcCommand::Open { panel } => match parse_panel(&panel) {
                Some(panel) => (
                    ok(format!("Opening {panel:?}")),
                    Task::done(Message::OpenPanel(panel)),
                ),
                None => (
                    error(format!("Unknown panel: {panel}")),
                    Task::none(),
                ),
            },
            IpcCommand::Close => (ok("Closing active panel"), Task::done(Message::CloseActive)),
            IpcCommand::Dash { tab } => {
                let tab = parse_tab(tab.as_deref());
                (ok("Opening dashboard"), Task::done(Message::OpenDash(tab)))
            }
            IpcCommand::Lock => (ok("Locking session"), Task::done(Message::Lock)),
            IpcCommand::Tray { args } => match args.first().map(String::as_str) {
                Some("list") | None => {
                    let items = crate::services::tray::registered_items();
                    (ok(if items.is_empty() { "No StatusNotifier items registered".into() } else { items.join("\n") }), Task::none())
                }
                _ => (error("Usage: cds tray list"), Task::none()),
            },
            IpcCommand::Settings { args } => match args.as_slice() {
                [] => {
                    (ok(format!("language={}\ndark={}\nmotion={}\npinned={}\ntrans_meter_colors={}",
                        self.settings.language, self.settings.dark, self.settings.motion_style,
                        self.settings.pinned_apps.len(), self.settings.flag("trans_meter_colors"))), Task::none())
                }
                [operation] if operation == "get" => {
                    (ok(format!("language={}\\ndark={}\\nmotion={}\\npinned={}\\ntrans_meter_colors={}",
                        self.settings.language, self.settings.dark, self.settings.motion_style,
                        self.settings.pinned_apps.len(), self.settings.flag("trans_meter_colors"))), Task::none())
                }
                [operation, key, value] if operation == "set" => {
                    match key.as_str() {
                        "language" => self.settings.language = if value == "en" { "en" } else { "pt-BR" }.into(),
                        "dark" => self.settings.dark = value == "true",
                        "motion" => self.settings.motion_style = if value == "fluent" { "fluent" } else { "m3" }.into(),
                        _ => return (error("Supported settings: language, dark, motion"), Task::none()),
                    }
                    match self.settings.save() {
                        Ok(()) => (ok("Settings saved"), Task::none()),
                        Err(err) => (error(err.to_string()), Task::none()),
                    }
                }
                [operation, name, value] if operation == "flag" => {
                    match self.settings.set_flag(name, value == "true") {
                        Ok(()) => (ok(format!("flag.{name}={value}")), Task::none()),
                        Err(err) => (error(err.to_string()), Task::none()),
                    }
                }
                _ => (error("Usage: cds settings [get] | set <language|dark|motion> <value> | flag <name> <true|false>"), Task::none()),
            },
            IpcCommand::Theme { args } => match args.as_slice() {
                [operation, value, ..] if operation == "color" => {
                    let color = value.clone();
                    (
                        ok("Generating Material 3 color scheme"),
                        Task::perform(
                            async move {
                                tokio::task::spawn_blocking(move || {
                                    let theme = crate::services::theme::generate_from_hex(&color)
                                        .map_err(|error| error.to_string())?;
                                    theme.save().map_err(|error| error.to_string())?;
                                    Ok(theme)
                                })
                                .await
                                .map_err(|error| error.to_string())?
                            },
                            Message::ThemeGenerated,
                        ),
                    )
                }
                [operation, image, ..] if operation == "set" => {
                    let image = std::path::PathBuf::from(image);
                    (
                        ok("Generating Material 3 scheme from wallpaper"),
                        Task::perform(
                            async move {
                                tokio::task::spawn_blocking(move || {
                                    let theme = crate::services::theme::generate_from_wallpaper(&image)
                                        .map_err(|error| error.to_string())?;
                                    theme.save().map_err(|error| error.to_string())?;
                                    Ok(theme)
                                })
                                .await
                                .map_err(|error| error.to_string())?
                            },
                            Message::ThemeGenerated,
                        ),
                    )
                }
                _ => (error("Usage: cds theme color <hex> | cds theme set <image>"), Task::none()),
            },
            IpcCommand::ClipboardPush { content } => match self.clipboard_history.push(&content) {
                Ok(true) => (ok("Clipboard history updated"), Task::none()),
                Ok(false) => (ok("Clipboard selection ignored"), Task::none()),
                Err(err) => (error(err.to_string()), Task::none()),
            },
            IpcCommand::Clipboard { args } => match args.first().map(String::as_str) {
                Some("clear") => match self.clipboard_history.clear() {
                    Ok(()) => (ok("Clipboard history cleared"), Task::none()),
                    Err(err) => (error(err.to_string()), Task::none()),
                },
                Some("list") | None => {
                    let message = self.clipboard_history.entries().iter().take(20)
                        .enumerate()
                        .map(|(index, value)| format!("{}. {}", index + 1, clipboard_preview(value)))
                        .collect::<Vec<_>>()
                        .join("\n");
                    (ok(message), Task::none())
                }
                _ => (error("Usage: cds clipboard list | cds clipboard clear"), Task::none()),
            },
            IpcCommand::Weather => (
                error("The weather service is part of the next migration step"),
                Task::none(),
            ),
        }
    }

    fn register_shell(&mut self, info: ShellInfo) {
        match info.shell {
            ShellType::LayerShell => {
                self.windows.entry(info.window).or_insert(WindowKind::Bar);
            }
            ShellType::SessionLock => {
                self.windows.insert(info.window, WindowKind::Lock);
            }
            _ => {}
        }
    }

    fn remove_window(&mut self, id: window::Id) {
        self.windows.remove(&id);
        if self.active_panel == Some(id) {
            self.active_panel = None;
        }
    }

    fn open_panel(&mut self, panel: Panel) -> Task<Message> {
        let close = self.close_active();
        let id = window::Id::unique();
        self.windows.insert(id, WindowKind::Panel(panel));
        self.active_panel = Some(id);

        let settings = if panel == Panel::Launcher {
            NewLayerShellSettings {
                anchor: Anchor::Left | Anchor::Right | Anchor::Top,
                layer: Layer::Overlay,
                size: LayerSize::px(640, 460),
                exclusive_zone: None,
                margin: Some((58, 0, 0, 0)),
                keyboard_interactivity: KeyboardInteractivity::Exclusive,
                output_option: OutputOption::LastOutput,
                namespace: Some("cds-launcher".into()),
                ..Default::default()
            }
        } else {
            NewLayerShellSettings {
                anchor: Anchor::Left | Anchor::Right | Anchor::Top | Anchor::Bottom,
                layer: Layer::Overlay,
                size: LayerSize::FILL,
                keyboard_interactivity: KeyboardInteractivity::Exclusive,
                output_option: OutputOption::LastOutput,
                namespace: Some("cds-panel".into()),
                ..Default::default()
            }
        };

        let audio_refresh = if panel == Panel::Control {
            Task::batch(vec![Task::done(Message::AudioRefresh), Task::done(Message::SystemRefresh)])
        } else if panel == Panel::Tray {
            Task::done(Message::TrayRefresh)
        } else { Task::none() };
        Task::batch(vec![
            close,
            Task::done(Message::NewLayerShell { settings, id }),
            audio_refresh,
        ])
    }

    fn close_active(&mut self) -> Task<Message> {
        let Some(id) = self.active_panel.take() else {
            return Task::none();
        };
        Task::done(Message::CloseWindow(id))
    }

    fn view(&self, id: window::Id) -> Element<'_, Message> {
        match self.windows.get(&id).copied() {
            Some(WindowKind::Bar) => self.bar_view(),
            Some(WindowKind::Panel(panel)) => self.panel_view(panel, id),
            Some(WindowKind::Lock) => self.lock_view(id),
            Some(WindowKind::Osd) => self.osd_view(),
            None => container(text("")).into(),
        }
    }

    fn bar_view(&self) -> Element<'_, Message> {
        let mut workspaces = self
            .niri_state
            .workspaces
            .workspaces
            .values()
            .collect::<Vec<_>>();
        workspaces.sort_by_key(|workspace| {
            (workspace.output.clone().unwrap_or_default(), workspace.idx)
        });

        let workspace_buttons = workspaces
            .into_iter()
            .map(|workspace| {
                let label = workspace
                    .name
                    .clone()
                    .unwrap_or_else(|| workspace.idx.to_string());
                let label = if workspace.is_focused {
                    format!("● {label}")
                } else {
                    label
                };
                button(text(label).size(13))
                    .on_press(Message::FocusWorkspace(workspace.idx))
                    .into()
            })
            .collect::<Vec<Element<'_, Message>>>();

        let focused_title = self
            .niri_state
            .windows
            .windows
            .values()
            .find(|window| window.is_focused)
            .and_then(|window| window.title.as_deref())
            .unwrap_or("Desktop");

        let workspace_group = container(
            row![
                text("◉").size(17),
                iced::widget::row(workspace_buttons)
                    .spacing(4)
                    .align_y(Alignment::Center),
                button("Apps").on_press(Message::OpenPanel(Panel::Apps)),
            ]
            .spacing(10)
            .align_y(Alignment::Center),
        )
        .padding(10)
        .style(self.surface_style(0.90));

        let island_label = if let Some(notification) = self.notifications.first() {
            format!("{} · {}", notification.app_name, notification.summary)
        } else if self.media.available() && self.media.is_playing() {
            if self.media.artist.is_empty() { self.media.title.clone() }
            else { format!("{} — {}", self.media.title, self.media.artist) }
        } else {
            focused_title.to_owned()
        };
        let island_artwork: Element<'_, Message> = if let Some(notification) = self.notifications.first() {
            if let Some(path) = resolve_icon_path(&notification.app_icon) {
                app_icon_sized(Some(&path), 28.0)
            } else {
                text("●").size(18).into()
            }
        } else if let Some(path) = self.media.artwork_path.as_ref() {
            app_icon_sized(Some(path), 28.0)
        } else {
            text("♫").size(18).into()
        };
        let center = container(
            row![
                button("⌕").on_press(Message::OpenPanel(Panel::Spotlight)),
                text("CDS").size(14),
                island_artwork,
                container(text(island_label.clone()).size(12))
                    .width(Length::Fill)
                    .center_x(Length::Fill),
                text(chrono::Local::now().format("%H:%M").to_string()).size(12),
                button("Dash").on_press(Message::OpenDash(DashTab::Home)),
            ]
            .spacing(9)
            .align_y(Alignment::Center),
        )
        .padding(10)
        .width(Length::Fixed(if self.notifications.first().is_some() {
            (180.0 + island_label.chars().count() as f32 * 7.2).clamp(280.0, 620.0)
        } else if self.media.available() && self.media.is_playing() {
            (220.0 + island_label.chars().count() as f32 * 6.2).clamp(300.0, 620.0)
        } else {
            260.0
        }))
        .style(self.surface_style(0.98));

        let tray_count = crate::services::tray::registered_items().len();
        let status = container(
            row![
                text(format!("◖ {}%", self.audio.output_volume)).size(12),
                button("Clip").on_press(Message::OpenPanel(Panel::Clipboard)),
                button(format!("◈ {tray_count}")).on_press(Message::OpenPanel(Panel::Tray)),
                button("Control").on_press(Message::OpenPanel(Panel::Control)),
                button("Settings").on_press(Message::OpenPanel(Panel::Settings)),
            ]
            .spacing(8)
            .align_y(Alignment::Center),
        )
        .padding(10)
        .style(self.surface_style(0.90));

        let mut left_groups = Vec::<Element<'_, Message>>::new();
        let mut center_groups = Vec::<Element<'_, Message>>::new();
        let mut right_groups = Vec::<Element<'_, Message>>::new();

        if self.settings.flag("show_workspaces") {
            match self.settings.workspaces_alignment.as_str() {
                "right" => right_groups.push(workspace_group.into()),
                _ => left_groups.push(workspace_group.into()),
            }
        }
        if self.settings.flag("show_center_island") {
            match self.settings.center_alignment.as_str() {
                "left" => left_groups.push(center.into()),
                "right" => right_groups.push(center.into()),
                _ => center_groups.push(center.into()),
            }
        }
        if self.settings.flag("show_status") {
            match self.settings.status_alignment.as_str() {
                "left" => left_groups.push(status.into()),
                _ => right_groups.push(status.into()),
            }
        }

        let left = iced::widget::row(left_groups).spacing(10).align_y(Alignment::Center);
        let middle = iced::widget::row(center_groups).spacing(10).align_y(Alignment::Center);
        let right = iced::widget::row(right_groups).spacing(10).align_y(Alignment::Center);

        container(
            row![
                left,
                iced::widget::Space::with_width(Length::Fill),
                middle,
                iced::widget::Space::with_width(Length::Fill),
                right,
            ]
                .spacing(10)
                .align_y(Alignment::Center)
                .width(Length::Fill)
                .height(Length::Fill),
        )
        .padding(6)
        .width(Length::Fill)
        .height(Length::Fill)
        .into()
    }

    fn panel_view(&self, panel: Panel, id: window::Id) -> Element<'_, Message> {
        match panel {
            Panel::Launcher => return self.launcher_panel_view(id),
            Panel::Apps | Panel::Spotlight => return self.apps_panel_view(panel, id),
            Panel::Settings => return self.settings_panel_view(id),
            Panel::Control => return self.control_panel_view(id),
            Panel::Tray => return self.tray_panel_view(id),
            Panel::Emoji => return self.emoji_panel_view(id),
            Panel::Color => return self.color_panel_view(id),
            Panel::Clipboard => return self.clipboard_panel_view(id),
            _ => {}
        }

        let title = match panel {
            Panel::Launcher => "Application launcher",
            Panel::Apps => "Applications",
            Panel::Spotlight => "Spotlight",
            Panel::Clipboard => "Clipboard history",
            Panel::Power => "Power",
            Panel::Control => "Control center",
            Panel::Settings => "Settings",
            Panel::Color => "HEX color picker",
            Panel::Emoji => "Emoji picker",
            Panel::Dash => "Dashboard",
            Panel::Tray => "System tray",
        };

        let body: Element<'_, Message> = if panel == Panel::Dash {
            let page: Element<'_, Message> = match self.dash_tab {
                DashTab::Media => column![
                    if let Some(path) = self.media.artwork_path.as_ref() {
                        app_icon_sized(Some(path), 104.0)
                    } else {
                        text("♫").size(56).into()
                    },
                    text(if self.media.available() { self.media.title.clone() } else { "Nenhuma mídia em reprodução" }).size(20),
                    text(self.media.artist.clone()).size(13),
                    text(self.media.album.clone()).size(11),
                    iced::widget::progress_bar(
                        0.0..=1.0,
                        if self.media.length_ms == 0 { 0.0 } else {
                            (self.media.position_ms as f32 / self.media.length_ms as f32).clamp(0.0, 1.0)
                        },
                    ),
                    text(format!(
                        "{} / {}",
                        format_media_time(self.media.position_ms),
                        format_media_time(self.media.length_ms),
                    )).size(10),
                    row![
                        button("⏮").on_press(Message::MediaControl("Previous".into())),
                        button(if self.media.is_playing() { "Ⅱ" } else { "▶" }).on_press(Message::MediaControl("PlayPause".into())),
                        button("⏭").on_press(Message::MediaControl("Next".into())),
                    ].spacing(10),
                    button("Atualizar player").on_press(Message::MediaRefresh),
                ].spacing(10).into(),
                DashTab::Alerts => column![
                    text(format!("{} notificações", self.notifications.len())).size(18),
                    iced::widget::Column::with_children(self.notifications.iter().map(|item| {
                        row![
                            column![text(item.summary.clone()).size(14), text(item.body.clone()).size(11)],
                            button("×").on_press(Message::NotificationDismiss(item.id)),
                        ].spacing(10).into()
                    }).collect::<Vec<Element<'_, Message>>>()).spacing(8),
                ].spacing(10).into(),
                _ => column![
                    text(format!("Selected tab: {:?}", self.dash_tab)),
                    text("Este módulo será preenchido com os serviços nativos correspondentes."),
                ].spacing(8).into(),
            };
            column![
                text(title).size(24),
                row![
                    button("Home").on_press(Message::SelectDashTab(DashTab::Home)),
                    button("Media").on_press(Message::SelectDashTab(DashTab::Media)),
                    button("Weather").on_press(Message::SelectDashTab(DashTab::Weather)),
                    button("Wallpapers").on_press(Message::SelectDashTab(DashTab::Wallpapers)),
                    button("Alerts").on_press(Message::SelectDashTab(DashTab::Alerts)),
                ].spacing(6),
                page,
                button("Close").on_press(Message::CloseWindow(id)),
            ].spacing(16).into()
        } else {
            column![
                text(title).size(24),
                text("Native Iced surface initialized."),
                text("This panel is a migration scaffold; feature parity is tracked in docs/porting.md."),
                button("Close").on_press(Message::CloseWindow(id)),
            ]
            .spacing(16)
            .into()
        };

        let card = container(body)
            .padding(28)
            .max_width(680)
            .style(self.surface_style(0.98));

        container(card)
            .width(Length::Fill)
            .height(Length::Fill)
            .center_x(Length::Fill)
            .center_y(Length::Fill)
            .style(|_| container::Style {
                background: Some(Background::Color(Color::from_rgba(0.02, 0.02, 0.03, 0.48))),
                ..Default::default()
            })
            .into()
    }

    fn apps_panel_view(&self, panel: Panel, id: window::Id) -> Element<'_, Message> {
        let full_grid = panel == Panel::Apps;
        let title = match panel {
            Panel::Apps => "Aplicativos",
            Panel::Launcher => "Launcher",
            _ => "Spotlight",
        };

        let search = text_input("Buscar aplicativos…", &self.app_query)
            .on_input(Message::AppsQuery)
            .padding(12)
            .size(16);

        let visible_limit = if full_grid { 1000 } else { 16 };
        let results = self
            .app_results
            .iter()
            .take(visible_limit)
            .filter_map(|id| self.apps.iter().find(|app| &app.id == id))
            .collect::<Vec<_>>();

        let columns = if full_grid { 6 } else { 1 };
        let mut rows: Vec<Element<'_, Message>> = Vec::new();

        for chunk in results.chunks(columns) {
            let cells = chunk
                .iter()
                .map(|app| {
                    let icon = app_icon(app.icon.as_ref());
                    if full_grid {
                        button(
                            column![
                                icon,
                                text(format!(
                                    "{}{}",
                                    if self.app_results.get(self.app_selected) == Some(&app.id) { "● " } else { "" },
                                    app.name
                                )).size(13),
                            ]
                            .spacing(8)
                            .align_x(Alignment::Center),
                        )
                        .width(Length::Fixed(116.0))
                        .height(Length::Fixed(112.0))
                        .on_press(Message::LaunchApp(app.id.clone()))
                        .into()
                    } else {
                        button(
                            row![
                                icon,
                                column![
                                    text(format!(
                                        "{}{}",
                                        if self.app_results.get(self.app_selected) == Some(&app.id) { "● " } else { "" },
                                        app.name
                                    )).size(15),
                                    text(app.description.clone()).size(11),
                                ]
                                .spacing(3),
                            ]
                            .spacing(12)
                            .align_y(Alignment::Center),
                        )
                        .width(Length::Fill)
                        .on_press(Message::LaunchApp(app.id.clone()))
                        .into()
                    }
                })
                .collect::<Vec<Element<'_, Message>>>();

            rows.push(iced::widget::row(cells).spacing(8).into());
        }

        let results_view: Element<'_, Message> = if rows.is_empty() {
            container(
                column![
                    text("Nenhum aplicativo encontrado").size(17),
                    text("Tente outro nome ou palavra-chave.").size(12),
                ]
                .spacing(8)
                .align_x(Alignment::Center),
            )
            .width(Length::Fill)
            .padding(24)
            .into()
        } else {
            scrollable(column(rows).spacing(8))
                .height(Length::Fill)
                .into()
        };

        let error_view: Element<'_, Message> = self
            .last_error
            .as_ref()
            .map(|error| text(error.clone()).size(12).into())
            .unwrap_or_else(|| text("").into());

        let body = column![
            text(title).size(24),
            search,
            error_view,
            results_view,
            button("Fechar").on_press(Message::CloseWindow(id)),
        ]
        .spacing(14)
        .width(Length::Fill)
        .height(Length::Fill);

        if full_grid {
            container(body)
                .padding(24)
                .width(Length::Fill)
                .height(Length::Fill)
                .style(self.surface_style(0.98))
                .into()
        } else {
            let card = container(body)
                .padding(24)
                .max_width(760)
                .height(Length::Shrink)
                .style(self.surface_style(0.98));

            container(card)
                .width(Length::Fill)
                .height(Length::Fill)
                .center_x(Length::Fill)
                .center_y(Length::Fill)
                .style(|_| container::Style {
                    background: Some(Background::Color(Color::from_rgba(0.02, 0.02, 0.03, 0.48))),
                    ..Default::default()
                })
                .into()
        }
    }

    /// Original CDS launcher: a compact 640×460 vertical result list below the notch.
    /// This intentionally stays separate from the full app grid and Spotlight.
    fn settings_panel_view(&self, id: window::Id) -> Element<'_, Message> {
        let body = column![
            row![text("Configurações do CDS").size(23), iced::widget::Space::with_width(Length::Fill), button("Fechar").on_press(Message::CloseWindow(id))].align_y(Alignment::Center),
            text("Preferências persistentes · importadas do settings.conf antigo").size(11),
            row![
                text("Tema").width(Length::Fill),
                button(if self.settings.dark { "Escuro" } else { "Claro" })
                    .on_press(Message::SettingsSetDark(!self.settings.dark)),
            ].align_y(Alignment::Center),
            row![
                text("Idioma").width(Length::Fill),
                button("Português").on_press(Message::SettingsSetLanguage("pt-BR".into())),
                button("English").on_press(Message::SettingsSetLanguage("en".into())),
            ].align_y(Alignment::Center),
            row![
                text("Movimento").width(Length::Fill),
                button("Material 3").on_press(Message::SettingsSetMotion("m3".into())),
                button("Fluent").on_press(Message::SettingsSetMotion("fluent".into())),
            ].align_y(Alignment::Center),
            text("Alinhamento da barra").size(16),
            row![
                text("Workspaces").width(Length::Fill),
                button("Esquerda").on_press(Message::SettingsSetAlignment("workspaces".into(), "left".into())),
                button("Direita").on_press(Message::SettingsSetAlignment("workspaces".into(), "right".into())),
            ].align_y(Alignment::Center),
            row![
                text("Dynamic Island").width(Length::Fill),
                button("Esquerda").on_press(Message::SettingsSetAlignment("center".into(), "left".into())),
                button("Centro").on_press(Message::SettingsSetAlignment("center".into(), "center".into())),
                button("Direita").on_press(Message::SettingsSetAlignment("center".into(), "right".into())),
            ].align_y(Alignment::Center),
            row![
                text("Status").width(Length::Fill),
                button("Esquerda").on_press(Message::SettingsSetAlignment("status".into(), "left".into())),
                button("Direita").on_press(Message::SettingsSetAlignment("status".into(), "right".into())),
            ].align_y(Alignment::Center),
            text("Elementos da interface").size(16),
            settings_flag_row("Workspaces", "show_workspaces", self.settings.flag("show_workspaces")),
            settings_flag_row("Centro / Dynamic Island", "show_center_island", self.settings.flag("show_center_island")),
            settings_flag_row("Indicadores de status", "show_status", self.settings.flag("show_status")),
            settings_flag_row("Cores do medidor trans", "trans_meter_colors", self.settings.flag("trans_meter_colors")),
            settings_flag_row("Relógio analógico", "clock_analog", self.settings.flag("clock_analog")),
        ].spacing(12);
        container(container(body).padding(24).max_width(620).style(self.surface_style(0.98)))
            .width(Length::Fill).height(Length::Fill).center_x(Length::Fill).center_y(Length::Fill)
            .style(|_| container::Style { background: Some(Background::Color(Color::from_rgba(0.02,0.02,0.03,0.48))), ..Default::default() }).into()
    }

    fn control_panel_view(&self, id: window::Id) -> Element<'_, Message> {
        let body = column![
            row![text("Central de controle").size(23), iced::widget::Space::with_width(Length::Fill), button("↻").on_press(Message::AudioRefresh), button("Fechar").on_press(Message::CloseWindow(id))].align_y(Alignment::Center),
            text("Áudio").size(17),
            row![
                text(format!("Saída · {}{}", self.audio.output_volume, if self.audio.output_muted { "% · mutado" } else { "%" })).width(Length::Fill),
                button("−").on_press(Message::AudioAdjust(-5)),
                button("+").on_press(Message::AudioAdjust(5)),
                button(if self.audio.output_muted { "Ativar" } else { "Mudo" }).on_press(Message::AudioToggleOutputMute),
            ].align_y(Alignment::Center),
            iced::widget::slider(0..=100, self.audio.output_volume.min(100), Message::AudioSetVolume),
            row![
                text(format!("Microfone · {}{}", self.audio.input_volume, if self.audio.input_muted { "% · mutado" } else { "%" })).width(Length::Fill),
                button(if self.audio.input_muted { "Ativar" } else { "Silenciar" }).on_press(Message::AudioToggleInputMute),
            ].align_y(Alignment::Center),
            text("Conectividade e brilho").size(17),
            row![
                button(if self.system.wifi_enabled { "Wi-Fi ligado" } else { "Wi-Fi desligado" })
                    .on_press(Message::WifiToggle(!self.system.wifi_enabled)),
                button(if self.system.bluetooth_blocked { "Bluetooth desligado" } else { "Bluetooth ligado" })
                    .on_press(Message::BluetoothToggle(self.system.bluetooth_blocked)),
            ].spacing(8),
            row![
                text(format!("Brilho · {}%", self.system.brightness)).width(Length::Fill),
                button("−").on_press(Message::BrightnessAdjust(-5)),
                button("+").on_press(Message::BrightnessAdjust(5)),
            ].align_y(Alignment::Center),
            text("Os controles usam pactl, nmcli, rfkill e brightnessctl.").size(11),
            self.last_error.as_ref().map(|error| text(error.clone()).size(11).into()).unwrap_or_else(|| text("").into()),
        ].spacing(14);
        container(container(body).padding(24).max_width(520).style(self.surface_style(0.98)))
            .width(Length::Fill).height(Length::Fill).center_x(Length::Fill).center_y(Length::Fill)
            .style(|_| container::Style { background: Some(Background::Color(Color::from_rgba(0.02,0.02,0.03,0.48))), ..Default::default() }).into()
    }

    fn tray_panel_view(&self, id: window::Id) -> Element<'_, Message> {
        let content: Element<'_, Message> = if let Some(menu) = &self.tray_menu {
            let menu_rows = menu.entries.iter().map(|entry| {
                if entry.separator {
                    text("────────────────────────").size(10).into()
                } else {
                    let label = format!("{}{}", "  ".repeat(entry.depth.min(5)), entry.label);
                    if entry.enabled {
                        button(text(label).size(13)).width(Length::Fill)
                            .on_press(Message::TrayMenuActivate(entry.id)).into()
                    } else {
                        text(label).size(13).into()
                    }
                }
            }).collect::<Vec<Element<'_, Message>>>();
            column![
                row![text("Menu do aplicativo").size(17), iced::widget::Space::with_width(Length::Fill), button("Voltar").on_press(Message::TrayMenuDismiss)],
                scrollable(column(menu_rows).spacing(4)).height(Length::Fixed(320.0)),
            ].spacing(8).into()
        } else {
            let rows = self.tray_items.iter().map(|item| {
                let icon = resolve_icon_path(&item.icon_name)
                    .map(|path| app_icon_sized(Some(&path), 28.0))
                    .unwrap_or_else(|| text("◈").size(24).into());
                let title = if item.title.is_empty() { item.address.clone() } else { item.title.clone() };
                let content = container(row![
                    icon,
                    column![text(title).size(13), text(item.status.clone()).size(10)]
                        .spacing(2).width(Length::Fill),
                    text("›").size(18),
                ].spacing(10).align_y(Alignment::Center))
                    .padding(8).width(Length::Fill);
                mouse_area(content)
                    .on_press(Message::TrayActivate(item.address.clone()))
                    .on_right_press(Message::TrayContextMenu(item.address.clone()))
                    .into()
            }).collect::<Vec<Element<'_, Message>>>();
            if rows.is_empty() {
                text("Nenhum StatusNotifierItem registrado.").into()
            } else {
                scrollable(column(rows).spacing(6)).height(Length::Fixed(320.0)).into()
            }
        };
        let body = column![
            row![text("Bandeja do sistema").size(22), iced::widget::Space::with_width(Length::Fill), button("↻").on_press(Message::TrayRefresh), button("Fechar").on_press(Message::CloseWindow(id))].align_y(Alignment::Center),
            content,
        ].spacing(12);
        container(container(body).padding(22).max_width(540).style(self.surface_style(0.98)))
            .width(Length::Fill).height(Length::Fill).center_x(Length::Fill).center_y(Length::Fill)
            .style(|_| container::Style { background: Some(Background::Color(Color::from_rgba(0.02,0.02,0.03,0.48))), ..Default::default() }).into()
    }

    fn launcher_panel_view(&self, id: window::Id) -> Element<'_, Message> {
        let mut results = self.app_results.iter()
            .take(40)
            .filter_map(|app_id| self.apps.iter().find(|app| &app.id == app_id))
            .collect::<Vec<_>>();
        results.sort_by_key(|app| !self.settings.is_pinned(&app.pin_key()));
        let selected = self.app_selected.min(results.len().saturating_sub(1));
        let rows = results.iter().enumerate().map(|(index, app)| {
            let pinned = self.settings.is_pinned(&app.pin_key());
            let marker = if index == selected { "›" } else { " " };
            let pin = if pinned { "◆" } else { "" };
            let content = container(
                row![
                    container(app_icon_sized(app.icon.as_ref(), 36.0))
                        .width(Length::Fixed(36.0))
                        .height(Length::Fixed(36.0)),
                    column![
                        text(app.name.clone()).size(14),
                        text(if app.description.is_empty() { "Aplicativo" } else { &app.description }).size(11),
                    ].spacing(2).width(Length::Fill),
                    text(pin).size(12),
                    text(marker).size(18),
                ]
                .spacing(12)
                .align_y(Alignment::Center)
            )
            .padding(8)
            .width(Length::Fill)
            .style(if index == selected { self.surface_style(0.88) } else { self.surface_style(0.58) });
            mouse_area(content)
                .on_press(Message::LaunchApp(app.id.clone()))
                .on_right_press(Message::TogglePinned(app.pin_key()))
                .into()
        }).collect::<Vec<Element<'_, Message>>>();

        let results_view: Element<'_, Message> = if rows.is_empty() {
            container(column![
                text("Nada encontrado").size(16),
                text("Tente outro nome").size(12),
            ].spacing(6).align_x(Alignment::Center))
                .width(Length::Fill)
                .height(Length::Fixed(316.0))
                .center_x(Length::Fill)
                .center_y(Length::Fill)
                .into()
        } else {
            scrollable(column(rows).spacing(4))
                .height(Length::Fixed(316.0))
                .into()
        };

        let count = self.app_results.len();
        let hint = if count == 0 {
            "Nenhum aplicativo encontrado".to_owned()
        } else if self.settings.language == "en" {
            format!("{count} applications  ·  ↑↓ choose  ·  Enter opens  ·  right-click pins")
        } else {
            format!("{count} aplicativos  ·  ↑↓ selecionar  ·  Enter abrir  ·  clique direito fixa")
        };
        let footer = row![
            text(hint).size(11),
            text("Esc fecha").size(11),
        ]
        .spacing(12)
        .align_y(Alignment::Center);

        let body = column![
            text_input("Buscar aplicativos", &self.app_query)
                .on_input(Message::LauncherQuery)
                .padding(12)
                .size(15),
            results_view,
            footer,
        ]
        .spacing(12)
        .width(Length::Fill)
        .height(Length::Fill);

        container(body)
            .padding(16)
            .width(Length::Fill)
            .height(Length::Fill)
            .style(self.surface_style(0.98))
            .into()
    }

    fn emoji_panel_view(&self, id: window::Id) -> Element<'_, Message> {
        let entries = crate::services::emoji::search(&self.emoji_query, 128);
        let mut rows: Vec<Element<'_, Message>> = Vec::new();

        for chunk in entries.chunks(8) {
            let cells = chunk
                .iter()
                .map(|entry| {
                    button(
                        column![
                            emoji_artwork(*entry),
                            text(entry.name()).size(10),
                        ]
                        .spacing(4)
                        .align_x(Alignment::Center),
                    )
                    .width(Length::Fixed(78.0))
                    .height(Length::Fixed(76.0))
                    .on_press(Message::CopyEmoji(entry.glyph().to_owned()))
                    .into()
                })
                .collect::<Vec<Element<'_, Message>>>();

            rows.push(iced::widget::row(cells).spacing(6).into());
        }

        let results: Element<'_, Message> = if rows.is_empty() {
            text("Nenhum emoji encontrado.").into()
        } else {
            scrollable(column(rows).spacing(6))
                .height(Length::Fixed(420.0))
                .into()
        };

        let body = column![
            text("Emoji picker").size(24),
            text_input("Pesquisar emoji…", &self.emoji_query)
                .on_input(Message::EmojiQuery)
                .padding(12),
            results,
            button("Fechar").on_press(Message::CloseWindow(id)),
        ]
        .spacing(12);

        let card = container(body)
            .padding(24)
            .max_width(760)
            .style(self.surface_style(0.98));

        container(card)
            .width(Length::Fill)
            .height(Length::Fill)
            .center_x(Length::Fill)
            .center_y(Length::Fill)
            .style(|_| container::Style {
                background: Some(Background::Color(Color::from_rgba(0.02, 0.02, 0.03, 0.48))),
                ..Default::default()
            })
            .into()
    }

    fn clipboard_panel_view(&self, id: window::Id) -> Element<'_, Message> {
        let entries = self
            .clipboard_history
            .search(&self.clipboard_query, crate::services::clipboard::HISTORY_LIMIT);
        let selected = self.clipboard_selected.min(entries.len().saturating_sub(1));
        let mut rows: Vec<Element<'_, Message>> = Vec::new();

        for (index, value) in entries.iter().enumerate() {
            let marker = if index == selected { "●" } else { "○" };
            rows.push(
                button(
                    row![
                        text(marker).size(14),
                        column![
                            text(clipboard_preview(value)).size(14),
                            text(clipboard_meta(value)).size(11),
                        ]
                        .spacing(2),
                    ]
                    .spacing(12)
                    .align_y(Alignment::Center),
                )
                .width(Length::Fill)
                .on_press(Message::CopyClipboard(value.clone()))
                .into(),
            );
        }

        let list: Element<'_, Message> = if rows.is_empty() {
            container(
                column![
                    text("O clipboard está vazio").size(16),
                    text("Copie algo para aparecer aqui.").size(12),
                ]
                .spacing(8)
                .align_x(Alignment::Center),
            )
            .width(Length::Fill)
            .padding(28)
            .into()
        } else {
            scrollable(column(rows).spacing(4))
                .height(Length::Fixed(330.0))
                .into()
        };

        let body = column![
            row![
                text("Área de transferência").size(22),
                button("Limpar").on_press(Message::ClearClipboard),
            ]
            .spacing(12)
            .align_y(Alignment::Center),
            text_input("Buscar no clipboard…", &self.clipboard_query)
                .on_input(Message::ClipboardQuery)
                .padding(12),
            list,
            text(format!("{} itens · ↑↓ selecionar · Enter copiar · Esc fechar", entries.len())).size(11),
            button("Fechar").on_press(Message::CloseWindow(id)),
        ]
        .spacing(12);

        let card = container(body)
            .padding(24)
            .max_width(700)
            .style(self.surface_style(0.98));

        container(card)
            .width(Length::Fill)
            .height(Length::Fill)
            .center_x(Length::Fill)
            .center_y(Length::Fill)
            .style(|_| container::Style {
                background: Some(Background::Color(Color::from_rgba(0.02, 0.02, 0.03, 0.48))),
                ..Default::default()
            })
            .into()
    }

    fn active_panel_is(&self, panel: Panel) -> bool {
        self.active_panel
            .and_then(|id| self.windows.get(&id))
            .is_some_and(|kind| matches!(kind, WindowKind::Panel(active) if *active == panel))
    }

    fn active_apps_panel(&self) -> bool {
        self.active_panel
            .and_then(|id| self.windows.get(&id))
            .is_some_and(|kind| matches!(
                kind,
                WindowKind::Panel(Panel::Launcher | Panel::Apps | Panel::Spotlight)
            ))
    }

    fn color_panel_view(&self, id: window::Id) -> Element<'_, Message> {
        let picker = iced_aw::widget::color_picker::ColorPicker::new(
            self.color_picker_open,
            self.selected_color,
            button("Abrir seletor de cores").on_press(Message::ShowColorPicker),
            Message::CancelColorPicker,
            Message::ColorSelected,
        )
        .on_color_change(Message::ColorChanged);

        let preview = container(text(self.color_hex.clone()).size(18))
            .padding(16)
            .width(Length::Fill)
            .style(|_| container::Style {
                background: Some(Background::Color(self.selected_color)),
                ..Default::default()
            });

        let error_view: Element<'_, Message> = self
            .last_error
            .as_ref()
            .map(|error| text(error.clone()).size(12).into())
            .unwrap_or_else(|| text("").into());

        let body = column![
            text("HEX color picker").size(24),
            preview,
            picker,
            text_input("#RRGGBB", &self.color_hex)
                .on_input(Message::ColorHexChanged)
                .padding(12),
            row![
                button("Copiar HEX").on_press(Message::CopyColor),
                button("Capturar cor da tela").on_press(Message::Eyedropper),
            ]
            .spacing(8),
            error_view,
            button("Fechar").on_press(Message::CloseWindow(id)),
        ]
        .spacing(14);

        let card = container(body)
            .padding(24)
            .max_width(560)
            .style(self.surface_style(0.98));

        container(card)
            .width(Length::Fill)
            .height(Length::Fill)
            .center_x(Length::Fill)
            .center_y(Length::Fill)
            .style(|_| container::Style {
                background: Some(Background::Color(Color::from_rgba(0.02, 0.02, 0.03, 0.48))),
                ..Default::default()
            })
            .into()
    }

    fn surface_style(&self, alpha: f32) -> impl Fn(&iced::Theme) -> container::Style {
        let base = self
            .theme_tokens
            .as_ref()
            .and_then(|theme| {
                parse_hex_color(if self.settings.dark {
                    &theme.dark.surface_container
                } else {
                    &theme.light.surface_container
                })
            })
            .unwrap_or(if self.settings.dark {
                Color::from_rgb(0.09, 0.09, 0.12)
            } else {
                Color::from_rgb(0.94, 0.93, 0.96)
            });
        let background = Color::from_rgba(base.r, base.g, base.b, alpha);

        move |_| container::Style {
            background: Some(Background::Color(background)),
            ..Default::default()
        }
    }

    fn osd_view(&self) -> Element<'_, Message> {
        container(
            row![
                text(if self.osd_text.to_lowercase().contains("brilho") { "☼" } else { "◖" }).size(30),
                column![
                    text(self.osd_text.clone()).size(16),
                    text("CDS · OSD").size(10),
                ].spacing(2),
            ].spacing(16).align_y(Alignment::Center)
        )
        .padding(18)
        .width(Length::Fill)
        .height(Length::Fill)
        .style(self.surface_style(0.98))
        .into()
    }

    fn lock_view(&self, _id: window::Id) -> Element<'_, Message> {
        let error_view: Element<'_, Message> = self
            .lock_error
            .as_ref()
            .map(|error| text(error.clone()).size(12).into())
            .unwrap_or_else(|| text("").into());

        container(
            column![
                text("CDS").size(28),
                text("Sessão bloqueada").size(16),
                text_input("Senha", &self.lock_password)
                    .secure(true)
                    .on_input(Message::LockPasswordChanged)
                    .on_submit(Message::UnlockRequested)
                    .padding(14)
                    .width(Length::Fixed(320.0)),
                button("Desbloquear").on_press(Message::UnlockRequested),
                error_view,
            ]
            .spacing(16)
            .align_x(Alignment::Center),
        )
        .width(Length::Fill)
        .height(Length::Fill)
        .center_x(Length::Fill)
        .center_y(Length::Fill)
        .into()
    }
}

fn emoji_artwork(entry: crate::services::emoji::EmojiEntry) -> Element<'static, Message> {
    if let Some(path) = crate::services::emoji::asset_path(entry.emoji) {
        iced::widget::svg::Svg::new(iced::widget::svg::Handle::from_path(path))
            .width(Length::Fixed(34.0))
            .height(Length::Fixed(34.0))
            .into()
    } else {
        text(entry.glyph()).size(30).into()
    }
}

fn format_media_time(milliseconds: u64) -> String {
    let seconds = milliseconds / 1000;
    format!("{}:{:02}", seconds / 60, seconds % 60)
}

fn clipboard_preview(value: &str) -> String {
    let flat = value.split_whitespace().collect::<Vec<_>>().join(" ");
    if flat.chars().count() > 78 {
        let mut preview = flat.chars().take(77).collect::<String>();
        preview.push('…');
        preview
    } else {
        flat
    }
}

fn clipboard_meta(value: &str) -> String {
    format!(
        "{} caracteres · {} linhas",
        value.chars().count(),
        value.lines().count().max(1)
    )
}

fn settings_flag_row<'a>(label: &'a str, name: &'static str, enabled: bool) -> Element<'a, Message> {
    row![
        text(label).width(Length::Fill),
        button(if enabled { "Ligado" } else { "Desligado" })
            .on_press(Message::SettingsToggleFlag(name.to_owned())),
    ].align_y(Alignment::Center).into()
}

fn resolve_icon_path(value: &str) -> Option<std::path::PathBuf> {
    let path = std::path::PathBuf::from(value);
    if path.is_file() { return Some(path); }
    if value.is_empty() { return None; }
    freedesktop_icons::lookup(value).with_size(64).with_cache().find()
}

fn app_icon_sized(path: Option<&std::path::PathBuf>, size: f32) -> Element<'static, Message> {
    let Some(path) = path else { return text("◉").size(size * 0.75).into(); };
    if path.extension().is_some_and(|extension| extension == "svg") {
        iced::widget::svg::Svg::new(iced::widget::svg::Handle::from_path(path.clone()))
            .width(Length::Fixed(size)).height(Length::Fixed(size)).into()
    } else {
        iced::widget::image::Image::new(iced::widget::image::Handle::from_path(path.clone()))
            .width(Length::Fixed(size)).height(Length::Fixed(size))
            .content_fit(iced::ContentFit::Contain).into()
    }
}

fn app_icon(path: Option<&std::path::PathBuf>) -> Element<'static, Message> {
    let Some(path) = path else {
        return text("◉").size(38).into();
    };

    if path.extension().is_some_and(|extension| extension == "svg") {
        iced::widget::svg::Svg::new(iced::widget::svg::Handle::from_path(path.clone()))
            .width(Length::Fixed(48.0))
            .height(Length::Fixed(48.0))
            .into()
    } else {
        iced::widget::image::Image::new(iced::widget::image::Handle::from_path(path.clone()))
            .width(Length::Fixed(48.0))
            .height(Length::Fixed(48.0))
            .content_fit(iced::ContentFit::Contain)
            .into()
    }
}

fn parse_hex_color(value: &str) -> Option<Color> {
    let value = value.trim().trim_start_matches('#');
    if value.len() != 6 {
        return None;
    }

    let red = u8::from_str_radix(&value[0..2], 16).ok()?;
    let green = u8::from_str_radix(&value[2..4], 16).ok()?;
    let blue = u8::from_str_radix(&value[4..6], 16).ok()?;

    Some(Color::from_rgb8(red, green, blue))
}

fn color_to_hex(color: Color) -> String {
    format!(
        "#{:02X}{:02X}{:02X}",
        (color.r * 255.0).round() as u8,
        (color.g * 255.0).round() as u8,
        (color.b * 255.0).round() as u8,
    )
}

fn ok(message: impl Into<String>) -> IpcResponse {
    IpcResponse {
        ok: true,
        message: message.into(),
    }
}

fn error(message: impl Into<String>) -> IpcResponse {
    IpcResponse {
        ok: false,
        message: message.into(),
    }
}

fn parse_panel(name: &str) -> Option<Panel> {
    match name {
        "launcher" | "menu" => Some(Panel::Launcher),
        "apps" | "grade" => Some(Panel::Apps),
        "spotlight" => Some(Panel::Spotlight),
        "clipboard" | "clip" => Some(Panel::Clipboard),
        "power" => Some(Panel::Power),
        "control" => Some(Panel::Control),
        "config" | "settings" => Some(Panel::Settings),
        "color" | "color-picker" => Some(Panel::Color),
        "emoji" | "emoji-picker" => Some(Panel::Emoji),
        "dash" => Some(Panel::Dash),
        "tray" => Some(Panel::Tray),
        _ => None,
    }
}

fn parse_tab(name: Option<&str>) -> DashTab {
    match name {
        Some("media") => DashTab::Media,
        Some("weather") | Some("clima") => DashTab::Weather,
        Some("wallpapers") | Some("papeis") => DashTab::Wallpapers,
        Some("alerts") | Some("alertas") | Some("notifications") => DashTab::Alerts,
        _ => DashTab::Home,
    }
}

fn start_clipboard_watcher() -> Option<std::process::Child> {
    let executable = std::env::current_exe().ok()?;
    match std::process::Command::new("wl-paste")
        .args(["--type", "text", "--watch"])
        .arg(executable)
        .args(["clipboard", "ingest"])
        .spawn()
    {
        Ok(child) => Some(child),
        Err(error) => {
            tracing::warn!(%error, "Could not start the wl-paste clipboard watcher");
            None
        }
    }
}

pub fn run() -> anyhow::Result<()> {
    let paths = crate::paths::Paths::discover()?;
    paths.create_user_directories()?;

    let (ipc_rx, _socket_guard) = ipc::start_server()?;
    ipc::set_subscription_receiver(ipc_rx);
    let (shell_broadcast, shell_events) = iced_wayland_subscriber::shell::channel();
    let mut clipboard_watcher = start_clipboard_watcher();

    crate::services::notifications::start_server()?;
    crate::services::tray::start_server()?;

    let result = daemon(
        move || App::new(shell_events.clone()),
        App::namespace,
        App::update,
        App::view,
    )
    .subscription(App::subscription)
    .on_new_shell(|info: ShellInfo| {
        matches!(info.shell, ShellType::SessionLock)
            .then_some(Message::LockAppeared(info.window))
    })
    .wl_settings(ExWlSettings {
        layer_settings: LayerShellSettings {
            size: LayerSize::fill_width(48),
            exclusive_zone: 0,
            anchor: Anchor::Left | Anchor::Right | Anchor::Top,
            start_mode: StartMode::AllScreens,
            ..Default::default()
        },
        shell_broadcast,
        ..Default::default()
    })
    .run();

    if let Some(watcher) = clipboard_watcher.as_mut() {
        let _ = watcher.kill();
    }

    result.map_err(Into::into)
}
