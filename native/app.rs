use std::collections::HashMap;
use iced::widget::{button, column, container, row, scrollable, text, text_input};
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
    LockAppeared(window::Id),
    AppsQuery(String),
    AppsSearchResults(Vec<String>),
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
}

struct App {
    windows: HashMap<window::Id, WindowKind>,
    active_panel: Option<window::Id>,
    dash_tab: DashTab,
    shell_events: ShellReceiver,
    niri_state: EventStreamState,
    apps: Vec<DesktopApp>,
    app_query: String,
    app_results: Vec<String>,
    emoji_query: String,
    color_picker_open: bool,
    selected_color: Color,
    color_hex: String,
    last_error: Option<String>,
    theme_tokens: Option<crate::services::theme::ThemeTokens>,
}

impl App {
    fn new(shell_events: ShellReceiver) -> Self {
        let apps = crate::services::desktop_apps::scan();
        let app_results = apps.iter().map(|app| app.id.clone()).collect();

        Self {
            windows: HashMap::new(),
            active_panel: None,
            dash_tab: DashTab::Home,
            shell_events,
            niri_state: EventStreamState::default(),
            apps,
            app_query: String::new(),
            app_results,
            emoji_query: String::new(),
            color_picker_open: false,
            selected_color: Color::from_rgb(0.40, 0.32, 0.65),
            color_hex: "#6750A4".into(),
            last_error: None,
            theme_tokens: crate::services::theme::ThemeTokens::load().ok().flatten(),
        }
    }

    fn namespace() -> String {
        "Chloe Dynamic Shell".into()
    }

    fn subscription(&self) -> Subscription<Message> {
        let ipc = Subscription::run(crate::ipc::subscribe).map(Message::Ipc);
        let niri = Subscription::run(crate::services::niri::subscribe).map(Message::NiriEvent);

        Subscription::batch(vec![
            ipc,
            event::listen().map(Message::Event),
            window::close_events().map(Message::WindowClosed),
            self.shell_events.listen().map(Message::Wayland),
            niri,
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
            Message::AppsQuery(query) => {
                self.app_query = query.clone();
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
            Message::AppsSearchResults(results) => {
                self.app_results = results;
                Task::none()
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
            IpcCommand::Tray { .. } => (
                error("The StatusNotifier tray is part of the next service migration step"),
                Task::none(),
            ),
            IpcCommand::Settings { .. } => (
                error("The settings service is part of the next migration step"),
                Task::none(),
            ),
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

        let center = container(
            row![
                button("Search").on_press(Message::OpenPanel(Panel::Spotlight)),
                text(focused_title).size(13),
                text("CDS").size(15),
                button("Dash").on_press(Message::OpenDash(DashTab::Home)),
            ]
            .spacing(10)
            .align_y(Alignment::Center),
        )
        .padding(8)
        .style(self.surface_style(0.96));

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
            row![workspace_group, center, status]
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
            Panel::Apps | Panel::Launcher | Panel::Spotlight => return self.apps_panel_view(panel, id),
            Panel::Emoji => return self.emoji_panel_view(id),
            Panel::Color => return self.color_panel_view(id),
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
                                text(app.name.clone()).size(13),
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
                                    text(app.name.clone()).size(15),
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
                .style(surface_style(0.98))
                .into()
        } else {
            let card = container(body)
                .padding(24)
                .max_width(760)
                .height(Length::Shrink)
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
                            text(entry.glyph()).size(27),
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

    fn surface_style(&self, alpha: f32) -> impl Fn(&iced::Theme) -> container::Style {
        let base = self
            .theme_tokens
            .as_ref()
            .and_then(|theme| parse_hex_color(&theme.dark.surface_container))
            .unwrap_or(Color::from_rgb(0.09, 0.09, 0.12));
        let background = Color::from_rgba(base.r, base.g, base.b, alpha);

        move |_| container::Style {
            background: Some(Background::Color(background)),
            ..Default::default()
        }
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
