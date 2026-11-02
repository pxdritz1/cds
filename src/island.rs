use std::cell::RefCell;
use std::path::Path;
use std::sync::OnceLock;
use std::time::{Duration, Instant};

use amane::{
    Animation, Apps, Audio, Battery, Bluetooth, Brightness, Button, Canvas, Cap, Center, Column,
    Cpu, Easing, Full, Image, Key, Keyboard, Layer, LayerWindow, Media, Memory, Network,
    Notifications, Parent, Path as DrawPath, Rectangle, Row, Scroll, ScrollArea, Service, Shape,
    Size, Start, Text, Vertical, Widget, children, request_frame, shapes,
};

use crate::shell::clock::Clock;
use crate::shell::state::{Overlay, ShellState};
use crate::theme;

const PANEL_WIDTH: f32 = 860.0;
const PANEL_HEIGHT: f32 = 760.0;
const PANEL_PADDING: f32 = 18.0;
const GRID_GAP: f32 = 12.0;

pub fn view() -> LayerWindow {
    let state = ShellState::read();
    let overlay = state.overlay;
    let revealed = state.revealed;
    let media = Media::read();
    let audio = Audio::read();
    let notifications = Notifications::read();

    let expanded = overlay.is_some();
    let has_media = !media.title().is_empty();
    let visible = revealed || expanded || has_media;

    let target_width = if expanded {
        PANEL_WIDTH
    } else if has_media {
        420.0
    } else {
        236.0
    };
    let target_height = if expanded {
        PANEL_HEIGHT
    } else if has_media {
        94.0
    } else {
        56.0
    };

    let width = animated_size(&WIDTH, target_width);
    let height = animated_size(&HEIGHT, target_height);

    let content: Box<dyn Widget> = match overlay {
        Some(Overlay::Dashboard) => dashboard(&media, &audio, &notifications),
        Some(Overlay::Launcher) => launcher_page(),
        Some(Overlay::ControlCenter) => control_center_page(&audio),
        Some(Overlay::Notifications) => notifications_page(&notifications),
        Some(Overlay::Clipboard) => page_shell(
            "área de transferência",
            Overlay::Clipboard,
            boxed(tile("integração pendente", "o histórico de clipboard ainda não está conectado")),
        ),
        Some(Overlay::Power) => page_shell(
            "energia",
            Overlay::Power,
            boxed(tile("opções de energia", "os comandos de sessão serão integrados aqui")),
        ),
        None if has_media => player(&media),
        None if visible => compact(&notifications),
        None => boxed(Rectangle::new().width(Parent).height(Parent).fill(amane::Color::rgba(0, 0, 0, 0))),
    };

    LayerWindow::new()
        .width(width)
        .height(height)
        .anchor_vertical(Vertical::Top)
        .layer(Layer::Overlay)
        .namespace("cds-island")
        .visible(true)
        .space(if expanded { amane::Zone::Ignore } else { amane::Zone::Reserve })
        .keyboard(if expanded { Keyboard::OnDemand } else { Keyboard::None })
        .on_key(key_pressed)
        .margin(amane::Margin {
            top: 8,
            right: 0,
            bottom: 0,
            left: 0,
        })
        .child(
            Rectangle::new()
                .width(Parent)
                .height(Parent)
                .radius(if expanded { 32.0 } else { 28.0 })
                .fill(if visible { theme::SURFACE } else { amane::Color::rgba(0, 0, 0, 0) })
                .border(if visible { 1.0 } else { 0.0 }, theme::OUTLINE)
                .on_hover(set_revealed)
                .padding(if expanded { PANEL_PADDING } else { 10.0 })
                .on_click(island_clicked)
                .on_scroll(change_volume)
                .child(Column::new(vec![content])),
        )
}

pub fn reveal() -> LayerWindow {
    LayerWindow::new()
        .width(Full)
        .height(4.0)
        .anchor_vertical(Vertical::Top)
        .layer(Layer::Overlay)
        .namespace("cds-reveal")
        .child(
            Rectangle::new()
                .width(Parent)
                .height(4.0)
                .fill(amane::Color::rgba(0, 0, 0, 0))
                .on_hover(|inside| {
                    if inside {
                        set_revealed(true);
                    }
                }),
        )
}

fn boxed(widget: impl Widget + 'static) -> Box<dyn Widget> {
    Box::new(widget)
}

fn card(
    width: impl Into<Size>,
    height: impl Into<Size>,
    fill: amane::Color,
    content: impl Widget + 'static,
) -> Rectangle {
    Rectangle::new()
        .width(width)
        .height(height)
        .radius(26.0)
        .fill(fill)
        .padding(16.0)
        .child(content)
}

fn page_shell(title: &str, selected: Overlay, body: Box<dyn Widget>) -> Box<dyn Widget> {
    let clock = Clock::read();
    let header = Row::new(vec![
        boxed(
            Column::new(children![
                Text::new("chloe dynamic shell").size(11.0).color(theme::ON_SURFACE_VARIANT),
                Text::new(title).size(23.0).color(theme::ON_SURFACE),
            ])
            .gap(1.0),
        ),
        boxed(Text::new(clock.time.as_str()).size(27.0).color(theme::PRIMARY)),
    ])
    .width(Parent)
    .height(48.0)
    .gap(12.0)
    .justify(amane::SpaceBetween);

    let navigation = navigation_bar(selected);

    boxed(
        Column::new(vec![boxed(header), boxed(navigation), body])
            .width(Parent)
            .height(Parent)
            .gap(12.0),
    )
}

fn navigation_bar(selected: Overlay) -> Row {
    Row::new(vec![
        boxed(nav_item("◫", "visão geral", Overlay::Dashboard, selected)),
        boxed(nav_item("⌕", "aplicativos", Overlay::Launcher, selected)),
        boxed(nav_item("☷", "controles", Overlay::ControlCenter, selected)),
        boxed(nav_item("◉", "notificações", Overlay::Notifications, selected)),
    ])
    .width(Parent)
    .height(42.0)
    .gap(8.0)
}

fn nav_item(glyph: &str, label: &str, target: Overlay, selected: Overlay) -> Rectangle {
    Rectangle::new()
        .width(Parent)
        .height(42.0)
        .radius(16.0)
        .fill(if target == selected {
            theme::PRIMARY_CONTAINER
        } else {
            theme::SURFACE_CONTAINER
        })
        .align_child(Center, Center)
        .on_click(move |_| set_overlay(target))
        .child(
            Text::new(format!("{glyph}  {label}"))
                .size(12.0)
                .color(if target == selected {
                    theme::ON_PRIMARY_CONTAINER
                } else {
                    theme::ON_SURFACE_VARIANT
                }),
        )
}

fn dashboard(
    media: &Media,
    audio: &Audio,
    notifications: &Notifications,
) -> Box<dyn Widget> {
    let clock = Clock::read();

    let hero = Row::new(vec![
        boxed(clock_card(&clock)),
        boxed(weather_card()),
        boxed(user_card()),
    ])
    .width(Parent)
    .height(126.0)
    .gap(GRID_GAP);

    let main = Row::new(vec![
        boxed(calendar_card(&clock)),
        boxed(media_card(media)),
    ])
    .width(Parent)
    .height(310.0)
    .gap(GRID_GAP);

    let footer = Row::new(vec![
        boxed(system_card()),
        boxed(audio_card(audio)),
        boxed(connectivity_card()),
        boxed(notifications_card(notifications)),
    ])
    .width(Parent)
    .height(112.0)
    .gap(GRID_GAP);

    let grid = Column::new(vec![boxed(hero), boxed(main), boxed(footer)])
        .width(Parent)
        .gap(GRID_GAP);

    page_shell(
        "visão geral",
        Overlay::Dashboard,
        boxed(ScrollArea::new("cds-dashboard", grid)),
    )
}

fn clock_card(clock: &Clock) -> Rectangle {
    card(
        Parent,
        Parent,
        theme::PRIMARY_CONTAINER,
        Column::new(children![
            Text::new("agora").size(11.0).color(theme::ON_PRIMARY_CONTAINER),
            Text::new(clock.time.as_str()).size(43.0).color(theme::ON_PRIMARY_CONTAINER),
            Text::new(clock.date.as_str())
                .size(12.0)
                .color(theme::ON_PRIMARY_CONTAINER)
                .elide(),
        ])
        .gap(2.0),
    )
}

fn weather_card() -> Rectangle {
    card(
        Parent,
        Parent,
        theme::SURFACE_CONTAINER,
        Row::new(children![
            Text::new("☁").size(48.0).color(theme::TERTIARY),
            Column::new(children![
                Text::new("clima").size(12.0).color(theme::ON_SURFACE_VARIANT),
                Text::new("sem previsão").size(19.0).color(theme::ON_SURFACE),
                Text::new("integração ainda não conectada")
                    .size(10.0)
                    .color(theme::ON_SURFACE_VARIANT)
                    .elide(),
            ])
            .gap(2.0),
        ])
        .gap(12.0)
        .align(Center),
    )
}

fn user_card() -> Rectangle {
    let username = std::env::var("USER").unwrap_or_else(|_| "usuária".into());
    let initial = username.chars().next().unwrap_or('U').to_uppercase().to_string();
    let session = if std::env::var_os("NIRI_SOCKET").is_some() {
        "niri · wayland"
    } else {
        "sessão wayland"
    };

    card(
        Parent,
        Parent,
        theme::SURFACE_CONTAINER_HIGH,
        Row::new(children![
            Rectangle::new()
                .width(58.0)
                .height(58.0)
                .radius(29.0)
                .fill(theme::PRIMARY_CONTAINER)
                .align_child(Center, Center)
                .child(Text::new(initial).size(27.0).color(theme::ON_PRIMARY_CONTAINER)),
            Column::new(children![
                Text::new(username).size(18.0).color(theme::ON_SURFACE).elide(),
                Text::new(session).size(11.0).color(theme::ON_SURFACE_VARIANT),
                Text::new(uptime_label()).size(10.0).color(theme::ON_SURFACE_VARIANT),
            ])
            .gap(3.0),
        ])
        .gap(12.0)
        .align(Center),
    )
}

fn calendar_card(clock: &Clock) -> Rectangle {
    let first_weekday = (clock.weekday.saturating_sub(1) + 7 - (clock.day.saturating_sub(1) % 7)) % 7;
    let month_days = days_in_month(clock.month, clock.year);

    let mut content: Vec<Box<dyn Widget>> = vec![
        boxed(
            Row::new(children![
                Text::new(clock.month_label.as_str()).size(16.0).color(theme::ON_SURFACE),
                Text::new("hoje").size(10.0).color(theme::PRIMARY),
            ])
            .width(Parent)
            .height(24.0)
            .justify(amane::SpaceBetween),
        ),
    ];

    let weekdays = ["seg", "ter", "qua", "qui", "sex", "sáb", "dom"];
    let weekday_cells = weekdays
        .iter()
        .map(|day| calendar_cell(Some(day), false, false))
        .collect();

    content.push(boxed(
        Row::new(weekday_cells)
            .width(Parent)
            .height(20.0)
            .gap(4.0),
    ));

    let mut weeks: Vec<Box<dyn Widget>> = Vec::new();
    for week in 0..6 {
        let mut days: Vec<Box<dyn Widget>> = Vec::new();

        for weekday in 0..7 {
            let index = week * 7 + weekday;
            let day = index as i32 - first_weekday as i32 + 1;
            let valid = day > 0 && day <= month_days as i32;
            let value = if valid { Some(day.to_string()) } else { None };
            days.push(boxed(calendar_cell(
                value.as_deref(),
                valid && day as u32 == clock.day,
                weekday >= 5,
            )));
        }

        weeks.push(boxed(
            Row::new(days)
                .width(Parent)
                .height(25.0)
                .gap(4.0),
        ));
    }

    content.push(boxed(Column::new(weeks).width(Parent).gap(4.0)));

    card(
        536.0,
        310.0,
        theme::SURFACE_CONTAINER,
        Column::new(content).width(Parent).gap(8.0),
    )
}

fn calendar_cell(label: Option<&str>, selected: bool, weekend: bool) -> Rectangle {
    let fill = if selected {
        theme::PRIMARY_CONTAINER
    } else if weekend {
        theme::SURFACE_CONTAINER_HIGH
    } else {
        amane::Color::rgba(0, 0, 0, 0)
    };

    Rectangle::new()
        .width(Parent)
        .height(25.0)
        .radius(13.0)
        .fill(fill)
        .align_child(Center, Center)
        .child(Text::new(label.unwrap_or("")).size(11.0).color(if selected {
            theme::ON_PRIMARY_CONTAINER
        } else {
            theme::ON_SURFACE_VARIANT
        }))
}

fn days_in_month(month: u32, year: i32) -> u32 {
    match month {
        1 | 3 | 5 | 7 | 8 | 10 | 12 => 31,
        4 | 6 | 9 | 11 => 30,
        2 if year % 400 == 0 || (year % 4 == 0 && year % 100 != 0) => 29,
        2 => 28,
        _ => 30,
    }
}

fn media_card(media: &Media) -> Rectangle {
    let title = if media.title().is_empty() {
        "nada tocando"
    } else {
        media.title()
    };
    let artist = if media.artist().is_empty() {
        "reproduza algo para começar"
    } else {
        media.artist()
    };

    let cover = if let Some(path) = local_art(media.art_url()) {
        Rectangle::new()
            .width(72.0)
            .height(72.0)
            .radius(20.0)
            .fill(Image::cover(path))
            .border(1.0, theme::OUTLINE)
    } else {
        Rectangle::new()
            .width(72.0)
            .height(72.0)
            .radius(20.0)
            .fill(theme::PRIMARY_CONTAINER)
            .align_child(Center, Center)
            .child(Text::new("♫").size(32.0).color(theme::ON_PRIMARY_CONTAINER))
    };

    let details = Row::new(vec![
        boxed(cover),
        boxed(
            Column::new(children![
                Text::new(title).size(14.0).color(theme::ON_SURFACE).elide(),
                Text::new(artist).size(11.0).color(theme::ON_SURFACE_VARIANT).elide(),
            ])
            .width(Parent)
            .gap(4.0),
        ),
    ])
    .width(Parent)
    .height(76.0)
    .gap(12.0)
    .align(Center);

    let controls = Row::new(children![
        media_button("‹", |_| Media::previous()),
        media_button(if media.playing() { "Ⅱ" } else { "▶" }, |_| Media::play_pause()),
        media_button("›", |_| Media::next()),
    ])
    .width(Parent)
    .height(42.0)
    .gap(12.0)
    .align(Center);

    card(
        276.0,
        310.0,
        theme::SURFACE_CONTAINER_HIGH,
        Column::new(vec![
            boxed(Text::new("reprodução").size(11.0).color(theme::ON_SURFACE_VARIANT)),
            boxed(details),
            boxed(progress(media, 0.0)),
            boxed(controls),
        ])
        .width(Parent)
        .gap(12.0),
    )
}

fn media_button(label: &str, action: impl Fn(Button) + 'static) -> Rectangle {
    Rectangle::new()
        .width(42.0)
        .height(42.0)
        .radius(21.0)
        .fill(theme::SURFACE_CONTAINER)
        .align_child(Center, Center)
        .on_click(action)
        .child(Text::new(label).size(19.0).color(theme::PRIMARY))
}

fn system_card() -> Rectangle {
    let cpu = Cpu::read().percent();
    let memory = Memory::read().percent();

    card(
        Parent,
        Parent,
        theme::SURFACE_CONTAINER,
        Column::new(vec![
            boxed(Text::new("sistema").size(11.0).color(theme::ON_SURFACE_VARIANT)),
            boxed(meter_line("cpu", cpu)),
            boxed(meter_line("memória", memory)),
        ])
        .width(Parent)
        .gap(7.0),
    )
}

fn meter_line(label: &str, value: u8) -> Column {
    let width = 165.0 * f32::from(value.min(100)) / 100.0;

    Column::new(vec![
        boxed(
            Row::new(children![
                Text::new(label).size(10.0).color(theme::ON_SURFACE_VARIANT),
                Text::new(format!("{value}%")).size(10.0).color(theme::ON_SURFACE),
            ])
            .width(Parent)
            .justify(amane::SpaceBetween),
        ),
        boxed(
            Rectangle::new()
                .width(Parent)
                .height(7.0)
                .radius(4.0)
                .fill(theme::SURFACE_CONTAINER_HIGH)
                .align_child(Start, Center)
                .child(
                    Rectangle::new()
                        .width(width)
                        .height(7.0)
                        .radius(4.0)
                        .fill(theme::PRIMARY),
                ),
        ),
    ])
    .width(Parent)
    .gap(3.0)
}

fn audio_card(audio: &Audio) -> Rectangle {
    let volume = audio.volume();
    let mic = audio.microphone_volume();

    card(
        Parent,
        Parent,
        theme::PRIMARY_CONTAINER,
        Column::new(children![
            Text::new("áudio").size(11.0).color(theme::ON_PRIMARY_CONTAINER),
            Text::new(format!("{volume}%")).size(27.0).color(theme::ON_PRIMARY_CONTAINER),
            Text::new(format!("microfone · {mic}%"))
                .size(10.0)
                .color(theme::ON_PRIMARY_CONTAINER),
        ])
        .gap(1.0),
    )
    .on_click(|button| {
        if button == Button::Left {
            Audio::toggle_mute();
        }
    })
    .on_scroll(change_volume)
}

fn connectivity_card() -> Rectangle {
    let network = Network::read();
    let bluetooth = Bluetooth::read();
    let wifi_label = if network.connected() {
        if network.ssid().is_empty() { "conectada" } else { network.ssid() }
    } else {
        "desconectada"
    };
    let bluetooth_label = if bluetooth.powered() { "ativo" } else { "desligado" };

    card(
        Parent,
        Parent,
        theme::SURFACE_CONTAINER,
        Column::new(children![
            Text::new("conexões").size(11.0).color(theme::ON_SURFACE_VARIANT),
            Row::new(children![
                Text::new("⌁").size(17.0).color(theme::PRIMARY),
                Text::new(wifi_label).size(10.0).color(theme::ON_SURFACE).elide(),
            ])
            .gap(6.0),
            Row::new(children![
                Text::new("ᛒ").size(16.0).color(theme::PRIMARY),
                Text::new(format!("bluetooth · {bluetooth_label}"))
                    .size(10.0)
                    .color(theme::ON_SURFACE)
                    .elide(),
            ])
            .gap(6.0),
        ])
        .gap(3.0),
    )
    .on_click(|_| {
        let enabled = Network::read().wifi_enabled();
        Network::set_wifi(!enabled);
    })
}

fn notifications_card(notifications: &Notifications) -> Rectangle {
    let count = notifications.list().len();
    let summary = notifications
        .list()
        .last()
        .map(|notification| notification.summary())
        .unwrap_or("tudo em dia");

    card(
        Parent,
        Parent,
        theme::SURFACE_CONTAINER,
        Column::new(children![
            Text::new("notificações").size(11.0).color(theme::ON_SURFACE_VARIANT),
            Text::new(count.to_string()).size(27.0).color(theme::ON_SURFACE),
            Text::new(summary).size(10.0).color(theme::ON_SURFACE_VARIANT).elide(),
        ])
        .gap(1.0),
    )
    .on_click(|_| set_overlay(Overlay::Notifications))
}

fn launcher_page() -> Box<dyn Widget> {
    let apps = Apps::read();
    let mut rows: Vec<Box<dyn Widget>> = Vec::new();
    let entries = apps.list();

    for chunk in entries.iter().take(24).collect::<Vec<_>>().chunks(3) {
        let mut tiles: Vec<Box<dyn Widget>> = chunk
            .iter()
            .map(|app| {
                let launch = (**app).clone();
                boxed(
                    Rectangle::new()
                        .width(Parent)
                        .height(76.0)
                        .radius(20.0)
                        .fill(theme::SURFACE_CONTAINER)
                        .padding(12.0)
                        .on_click(move |_| launch.launch())
                        .child(
                            Column::new(children![
                                Text::new(app.name()).size(14.0).color(theme::ON_SURFACE).elide(),
                                Text::new(app.description().unwrap_or("aplicativo"))
                                    .size(10.0)
                                    .color(theme::ON_SURFACE_VARIANT)
                                    .elide(),
                            ])
                            .gap(4.0),
                        ),
                )
            })
            .collect();

        while tiles.len() < 3 {
            tiles.push(boxed(Rectangle::new().width(Parent).height(76.0).fill(amane::Color::rgba(0, 0, 0, 0))));
        }

        rows.push(boxed(
            Row::new(tiles)
                .width(Parent)
                .height(76.0)
                .gap(GRID_GAP),
        ));
    }

    if rows.is_empty() {
        rows.push(boxed(tile("carregando aplicativos", "a lista de aplicativos está sendo atualizada")));
    }

    page_shell(
        "aplicativos",
        Overlay::Launcher,
        boxed(ScrollArea::new(
            "cds-launcher",
            Column::new(rows).width(Parent).gap(GRID_GAP),
        )),
    )
}

fn control_center_page(audio: &Audio) -> Box<dyn Widget> {
    let network = Network::read();
    let bluetooth = Bluetooth::read();
    let brightness = Brightness::read();
    let battery = Battery::read();

    let rows = vec![
        boxed(Row::new(vec![
            boxed(control_tile(
                "volume",
                &format!("{}%", audio.volume()),
                "◖",
                !audio.muted(),
                |_| Audio::toggle_mute(),
            )),
            boxed(control_tile(
                "microfone",
                &format!("{}%", audio.microphone_volume()),
                "⌁",
                !audio.microphone_muted(),
                |_| Audio::toggle_microphone_mute(),
            ).on_scroll(change_microphone)),
        ]).width(Parent).height(126.0).gap(GRID_GAP)),
        boxed(Row::new(vec![
            boxed(control_tile(
                "wi-fi",
                if network.wifi_enabled() { "ligado" } else { "desligado" },
                "⌁",
                network.wifi_enabled(),
                |_| {
                    let enabled = Network::read().wifi_enabled();
                    Network::set_wifi(!enabled);
                },
            )),
            boxed(control_tile(
                "bluetooth",
                if bluetooth.powered() { "ligado" } else { "desligado" },
                "ᛒ",
                bluetooth.powered(),
                |_| {
                    let enabled = Bluetooth::read().powered();
                    Bluetooth::set_powered(!enabled);
                },
            )),
        ]).width(Parent).height(126.0).gap(GRID_GAP)),
        boxed(Row::new(vec![
            boxed(control_tile(
                "brilho",
                if brightness.present() { format!("{}%", brightness.percent()) } else { "indisponível".into() },
                "☼",
                brightness.present(),
                |_| {
                    let current = Brightness::read().percent();
                    Brightness::set(if current >= 90 { 25 } else { current.saturating_add(10) });
                },
            )),
            boxed(control_tile(
                "bateria",
                if battery.present() { format!("{}%", battery.percent()) } else { "sem bateria".into() },
                "▰",
                battery.present(),
                |_| {},
            )),
        ]).width(Parent).height(126.0).gap(GRID_GAP)),
    ];

    page_shell(
        "controles",
        Overlay::ControlCenter,
        boxed(ScrollArea::new(
            "cds-controls",
            Column::new(rows).width(Parent).gap(GRID_GAP),
        )),
    )
}

fn control_tile(
    title: &str,
    value: &str,
    glyph: &str,
    active: bool,
    action: impl Fn(Button) + 'static,
) -> Rectangle {
    card(
        Parent,
        Parent,
        if active { theme::PRIMARY_CONTAINER } else { theme::SURFACE_CONTAINER },
        Row::new(children![
            Text::new(glyph).size(31.0).color(if active {
                theme::ON_PRIMARY_CONTAINER
            } else {
                theme::ON_SURFACE_VARIANT
            }),
            Column::new(children![
                Text::new(title).size(13.0).color(if active {
                    theme::ON_PRIMARY_CONTAINER
                } else {
                    theme::ON_SURFACE_VARIANT
                }),
                Text::new(value).size(20.0).color(if active {
                    theme::ON_PRIMARY_CONTAINER
                } else {
                    theme::ON_SURFACE
                }),
            ])
            .gap(3.0),
        ])
        .gap(18.0)
        .align(Center),
    )
    .on_click(action)
}

fn notifications_page(notifications: &Notifications) -> Box<dyn Widget> {
    let mut entries: Vec<Box<dyn Widget>> = Vec::new();

    if !notifications.list().is_empty() {
        entries.push(boxed(
            Rectangle::new()
                .width(Parent)
                .height(42.0)
                .radius(16.0)
                .fill(theme::PRIMARY_CONTAINER)
                .align_child(Center, Center)
                .on_click(|_| Notifications::clear())
                .child(Text::new("limpar notificações").size(12.0).color(theme::ON_PRIMARY_CONTAINER)),
        ));
    }

    for notification in notifications.list().iter().rev() {
        let id = notification.id();
        let title = format!("{} · {}", notification.app_name(), notification.summary());
        let body = notification.body().to_string();

        entries.push(boxed(
            Rectangle::new()
                .width(Parent)
                .height(76.0)
                .radius(20.0)
                .fill(theme::SURFACE_CONTAINER)
                .padding(14.0)
                .on_click(move |_| Notifications::click(id))
                .child(
                    Column::new(children![
                        Text::new(title).size(13.0).color(theme::ON_SURFACE).elide(),
                        Text::new(body).size(11.0).color(theme::ON_SURFACE_VARIANT).elide(),
                    ])
                    .gap(4.0),
                ),
        ));
    }

    if entries.is_empty() {
        entries.push(boxed(tile("tudo em dia", "você não tem notificações novas")));
    }

    page_shell(
        "notificações",
        Overlay::Notifications,
        boxed(ScrollArea::new(
            "cds-notifications",
            Column::new(entries).width(Parent).gap(10.0),
        )),
    )
}

fn tile(title: &str, detail: &str) -> Rectangle {
    card(
        Parent,
        68.0,
        theme::SURFACE_CONTAINER,
        Column::new(children![
            Text::new(title).size(13.0).color(theme::ON_SURFACE),
            Text::new(detail).size(11.0).color(theme::ON_SURFACE_VARIANT).elide(),
        ])
        .gap(4.0),
    )
}

fn compact(notifications: &Notifications) -> Box<dyn Widget> {
    let count = notifications.list().len();
    let label = if count == 0 {
        String::from("chloe dynamic shell")
    } else {
        format!("{count} notificações")
    };

    boxed(
        Rectangle::new()
            .width(Parent)
            .height(Parent)
            .radius(20.0)
            .fill(theme::SURFACE_CONTAINER)
            .on_click(toggle_dash)
            .child(
                Row::new(children![
                    Rectangle::new()
                        .width(34.0)
                        .height(34.0)
                        .radius(17.0)
                        .fill(theme::PRIMARY_CONTAINER)
                        .align_child(Center, Center)
                        .child(Text::new("✦").size(18.0).color(theme::ON_PRIMARY_CONTAINER)),
                    Column::new(children![
                        Text::new("chloe's shell").size(13.0).color(theme::ON_SURFACE),
                        Text::new(label).size(11.0).color(theme::ON_SURFACE_VARIANT),
                    ])
                    .gap(1.0),
                    Text::new("⌄").size(18.0).color(theme::ON_SURFACE_VARIANT),
                ])
                .gap(10.0),
            ),
    )
}

fn player(media: &Media) -> Box<dyn Widget> {
    let started = EPOCH.get_or_init(Instant::now);
    let phase = started.elapsed().as_secs_f32();
    let rotation = if media.playing() {
        request_frame();
        phase * 14.0 % 360.0
    } else {
        0.0
    };

    let cover = if let Some(path) = local_art(media.art_url()) {
        Rectangle::new()
            .width(58.0)
            .height(58.0)
            .radius(18.0)
            .fill(Image::cover(path))
            .rotate(rotation)
            .border(2.0, theme::PRIMARY)
    } else {
        Rectangle::new()
            .width(58.0)
            .height(58.0)
            .radius(18.0)
            .fill(theme::PRIMARY_CONTAINER)
            .rotate(rotation)
            .align_child(Center, Center)
            .child(Text::new("♫").size(24.0).color(theme::ON_PRIMARY_CONTAINER))
    };

    let title = if media.title().is_empty() { "sem título" } else { media.title() };
    let artist = if media.artist().is_empty() {
        media.active().map_or("reprodutor", |p| p.identity())
    } else {
        media.artist()
    };

    boxed(
        Row::new(children![
            cover,
            Column::new(children![
                Text::new(title).size(14.0).color(theme::ON_SURFACE).elide(),
                Text::new(artist).size(12.0).color(theme::ON_SURFACE_VARIANT).elide(),
                progress(media, phase),
            ])
            .gap(3.0),
            media_button(if media.playing() { "Ⅱ" } else { "▶" }, |_| Media::play_pause()),
        ])
        .gap(12.0),
    )
}

fn progress(media: &Media, phase: f32) -> Canvas {
    let width = 220.0;
    let progress = if media.length().is_zero() {
        0.0
    } else {
        (media.position().as_secs_f32() / media.length().as_secs_f32()).clamp(0.0, 1.0)
    };
    let end = (width * progress).max(2.0);
    let mut wave = DrawPath::new().move_to(0.0, 7.0);

    for x in 1..=end as u32 {
        let x = x as f32;
        let y = 7.0 + (x * 0.11 + phase * 2.2).sin() * 3.0;
        wave = wave.line_to(x, y);
    }

    let active = wave.stroke(3.0, theme::PRIMARY).cap(Cap::Round);
    let remainder = DrawPath::new()
        .move_to(end + 4.0, 7.0)
        .line_to(width, 7.0)
        .stroke(2.0, theme::OUTLINE)
        .cap(Cap::Round);

    Canvas::new().width(width).height(14.0).shapes(shapes![remainder, active])
}

fn change_volume(scroll: Scroll) {
    let current = i32::from(Audio::read().volume());
    let step = if scroll.y < 0.0 { 5 } else { -5 };
    Audio::set_volume((current + step).clamp(0, 100) as u8);
}

fn change_microphone(scroll: Scroll) {
    let current = i32::from(Audio::read().microphone_volume());
    let step = if scroll.y < 0.0 { 5 } else { -5 };
    Audio::set_microphone_volume((current + step).clamp(0, 100) as u8);
}

fn uptime_label() -> String {
    let uptime = std::fs::read_to_string("/proc/uptime")
        .ok()
        .and_then(|value| value.split_whitespace().next()?.parse::<f64>().ok())
        .unwrap_or_default() as u64;
    let days = uptime / 86_400;
    let hours = (uptime % 86_400) / 3_600;

    if days > 0 {
        format!("ligada há {days}d {hours}h")
    } else {
        format!("ligada há {hours}h")
    }
}

fn set_revealed(inside: bool) {
    ShellState::write().revealed = inside;
}

fn set_overlay(overlay: Overlay) {
    ShellState::write().overlay = Some(overlay);
}

fn toggle_dash(_: Button) {
    let mut state = ShellState::write();
    state.overlay = if state.overlay == Some(Overlay::Dashboard) {
        None
    } else {
        Some(Overlay::Dashboard)
    };
}

fn key_pressed(key: Key) {
    if key == Key::Escape {
        ShellState::write().overlay = None;
    }
}

fn island_clicked(button: Button) {
    if button == Button::Middle {
        Media::play_pause();
    }
}

fn local_art(url: &str) -> Option<&str> {
    let path = url.strip_prefix("file://").unwrap_or(url);
    if path.starts_with('/') && Path::new(path).exists() {
        Some(path)
    } else {
        None
    }
}

static EPOCH: OnceLock<Instant> = OnceLock::new();

thread_local! {
    static WIDTH: RefCell<Option<Animation<f32>>> = const { RefCell::new(None) };
    static HEIGHT: RefCell<Option<Animation<f32>>> = const { RefCell::new(None) };
}

fn animated_size(
    animation: &'static std::thread::LocalKey<RefCell<Option<Animation<f32>>>>,
    target: f32,
) -> f32 {
    animation.with_borrow_mut(|current| {
        let current = current.get_or_insert_with(|| {
            Animation::new(target)
                .duration(Duration::from_millis(360))
                .easing(Easing::Out)
        });

        current.to(target);
        current.value()
    })
}
