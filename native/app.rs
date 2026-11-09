use std::collections::HashMap;
use std::sync::Arc;

use iced::widget::{button, column, container, row, text};
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
use futures_util::stream;

use crate::ipc::{self, IpcCommand, IpcEnvelope, IpcResponse};

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
}

#[to_exwlshell_message]
#[derive(Debug, Clone)]
enum Message {
    Ipc(IpcEnvelope),
    Wayland(ShellEvent),
    Event(Event),
    WindowClosed(window::Id),
    OpenPanel(Panel),
    OpenDash(DashTab),
    SelectDashTab(DashTab),
    CloseActive,
    CloseWindow(window::Id),
    LockAppeared(window::Id),
}

struct App {
    windows: HashMap<window::Id, WindowKind>,
    active_panel: Option<window::Id>,
    dash_tab: DashTab,
    ipc_rx: async_channel::Receiver<IpcEnvelope>,
    shell_events: ShellReceiver,
}

impl App {
    fn new(
        ipc_rx: async_channel::Receiver<IpcEnvelope>,
        shell_events: ShellReceiver,
    ) -> Self {
        Self {
            windows: HashMap::new(),
            active_panel: None,
            dash_tab: DashTab::Home,
            ipc_rx,
            shell_events,
        }
    }

    fn namespace() -> String {
        "Chloe Dynamic Shell".into()
    }

    fn subscription(&self) -> Subscription<Message> {
        let ipc_rx = self.ipc_rx.clone();
        let ipc = Subscription::run_with_id("cds-ipc", move || {
            stream::unfold(ipc_rx.clone(), |receiver| async move {
                receiver
                    .recv()
                    .await
                    .ok()
                    .map(|envelope| (Message::Ipc(envelope), receiver))
            })
        });

        Subscription::batch(vec![
            ipc,
            event::listen().map(Message::Event),
            window::close_events().map(Message::WindowClosed),
            self.shell_events.listen().map(Message::Wayland),
        ])
    }

    fn update(&mut self, message: Message) -> Task<Message> {
        match message {
            Message::Ipc(envelope) => {
                let (response, task) = self.handle_ipc(envelope.request.command);
                let _ = envelope.reply.send_blocking(response);
                task
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
                _ => Task::none(),
            },
            Message::WindowClosed(id) => {
                self.remove_window(id);
                Task::none()
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
                self.windows.insert(id, WindowKind::Lock);
                Task::none()
            }
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
            IpcCommand::Tray { .. } => (
                error("The StatusNotifier tray is part of the next service migration step"),
                Task::none(),
            ),
            IpcCommand::Settings { .. } => (
                error("The settings service is part of the next migration step"),
                Task::none(),
            ),
            IpcCommand::Theme { .. } => (
                error("The Matugen theme service is part of the next migration step"),
                Task::none(),
            ),
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

        let settings = NewLayerShellSettings {
            anchor: Anchor::Left | Anchor::Right | Anchor::Top | Anchor::Bottom,
            layer: Layer::Overlay,
            size: LayerSize::FILL,
            keyboard_interactivity: KeyboardInteractivity::Exclusive,
            output_option: OutputOption::LastOutput,
            namespace: Some("cds-panel".into()),
            ..Default::default()
        };

        Task::batch(vec![
            close,
            Task::done(Message::NewLayerShell { settings, id }),
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
            None => container(text("")).into(),
        }
    }

    fn bar_view(&self) -> Element<'_, Message> {
        let workspaces = container(
            row![
                text("◉").size(17),
                text("Workspaces").size(13),
                button("Apps").on_press(Message::OpenPanel(Panel::Apps)),
            ]
            .spacing(10)
            .align_y(Alignment::Center),
        )
        .padding(10)
        .style(surface_style(0.90));

        let center = container(
            row![
                button("Search").on_press(Message::OpenPanel(Panel::Spotlight)),
                text("CDS").size(15),
                button("Dash").on_press(Message::OpenDash(DashTab::Home)),
            ]
            .spacing(10)
            .align_y(Alignment::Center),
        )
        .padding(8)
        .style(surface_style(0.96));

        let status = container(
            row![
                button("Clip").on_press(Message::OpenPanel(Panel::Clipboard)),
                button("Control").on_press(Message::OpenPanel(Panel::Control)),
                button("Settings").on_press(Message::OpenPanel(Panel::Settings)),
            ]
            .spacing(8)
            .align_y(Alignment::Center),
        )
        .padding(10)
        .style(surface_style(0.90));

        container(
            row![workspaces, center, status]
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
        };

        let body: Element<'_, Message> = if panel == Panel::Dash {
            column![
                text(title).size(24),
                row![
                    button("Home").on_press(Message::SelectDashTab(DashTab::Home)),
                    button("Media").on_press(Message::SelectDashTab(DashTab::Media)),
                    button("Weather").on_press(Message::SelectDashTab(DashTab::Weather)),
                    button("Wallpapers").on_press(Message::SelectDashTab(DashTab::Wallpapers)),
                    button("Alerts").on_press(Message::SelectDashTab(DashTab::Alerts)),
                ]
                .spacing(6),
                text(format!("Selected tab: {:?}", self.dash_tab)),
                text("The dashboard modules will be ported with their existing behavior."),
                button("Close").on_press(Message::CloseWindow(id)),
            ]
            .spacing(16)
            .into()
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
            .style(surface_style(0.98));

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

    fn lock_view(&self, id: window::Id) -> Element<'_, Message> {
        container(
            column![
                text("Chloe Dynamic Shell").size(28),
                text("Session locked").size(16),
                button("Unlock").on_press(Message::UnLock),
                button("Close preview").on_press(Message::CloseWindow(id)),
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

fn surface_style(alpha: f32) -> impl Fn(&iced::Theme) -> container::Style {
    move |_| container::Style {
        background: Some(Background::Color(Color::from_rgba(0.09, 0.09, 0.12, alpha))),
        ..Default::default()
    }
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

pub fn run() -> anyhow::Result<()> {
    let paths = crate::paths::Paths::discover()?;
    paths.create_user_directories()?;

    let (ipc_rx, _socket_guard) = ipc::start_server()?;
    let (shell_broadcast, shell_events) = iced_wayland_subscriber::shell::channel();

    daemon(
        move || App::new(ipc_rx.clone(), shell_events.clone()),
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
    .run()
    .map_err(Into::into)
}
