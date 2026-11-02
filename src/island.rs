use std::path::Path;
use std::cell::RefCell;
use std::sync::OnceLock;
use std::time::{Duration, Instant};

use amane::{
    Animation, Apps, Audio, Battery, Bluetooth, Button, Canvas, Cap, Column, Cpu, Easing, Full, Image, Key, Keyboard, Layer,
    LayerWindow, Media, Memory, Network, Notifications, Parent, Path as DrawPath, Rectangle, Row,
    Scroll, ScrollArea, Service, Shape, Text, Vertical, Widget, Workspaces, children, request_frame,
    shapes,
};

use crate::shell::state::{Overlay, ShellState};
use crate::theme;

pub fn view() -> LayerWindow {
    let state = ShellState::read();
    let media = Media::read();
    let audio = Audio::read();
    let notifications = Notifications::read();

    let expanded = state.overlay == Some(Overlay::Dashboard);
    let visible = state.revealed || expanded || has_media;
    let has_media = !media.title().is_empty();
    let target_height = if expanded {
        540.0
    } else if has_media {
        94.0
    } else {
        56.0
    };
    let target_width = if expanded {
        640.0
    } else if has_media {
        420.0
    } else {
        236.0
    };

    let height = animated_size(&HEIGHT, target_height);
    let width = animated_size(&WIDTH, target_width);

    let content: Box<dyn Widget> = if expanded {
        dash(&media, &audio, &notifications)
    } else if has_media {
        player(&media)
    } else {
        compact(&notifications)
    };

    LayerWindow::new()
        .width(width)
        .height(height)
        .anchor_vertical(Vertical::Top)
        .layer(Layer::Overlay)
        .visible(visible)
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
                .fill(theme::SURFACE)
                .border(1.0, theme::OUTLINE)
                .on_hover(set_revealed)
                .padding(if expanded { 18.0 } else { 10.0 })
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
        .child(
            Rectangle::new()
                .width(Parent)
                .height(4.0)
                .fill(amane::Color::rgba(0, 0, 0, 0))
                .on_hover(set_revealed),
        )
}

fn set_revealed(inside: bool) {
    ShellState::write().revealed = inside;
}

fn compact(notifications: &Notifications) -> Box<dyn Widget> {
    let count = notifications.list().len();
    let label = if count == 0 {
        String::from("chloe dynamic shell")
    } else {
        format!("{count} notificações")
    };

    Box::new(
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

    let artwork = local_art(media.art_url());
    let cover = if let Some(path) = artwork {
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
            .child(Text::new("♫").size(24.0).color(theme::ON_PRIMARY_CONTAINER))
    };

    let title = if media.title().is_empty() { "sem título" } else { media.title() };
    let artist = if media.artist().is_empty() {
        media.active().map_or("reprodutor", |p| p.identity())
    } else {
        media.artist()
    };

    Box::new(
        Row::new(children![
            cover,
            Column::new(children![
                Text::new(title).size(14.0).color(theme::ON_SURFACE),
                Text::new(artist).size(12.0).color(theme::ON_SURFACE_VARIANT),
                progress(media, phase),
            ])
            .gap(3.0),
            Rectangle::new()
                .width(42.0)
                .height(42.0)
                .radius(21.0)
                .fill(theme::SURFACE_CONTAINER)
                .on_click(|_| Media::play_pause())
                .child(Text::new(if media.playing() { "Ⅱ" } else { "▶" }).size(17.0).color(theme::PRIMARY)),
            Rectangle::new()
                .width(30.0)
                .height(42.0)
                .radius(15.0)
                .fill(theme::SURFACE_CONTAINER)
                .on_click(toggle_dash)
                .child(Text::new("⌄").size(16.0).color(theme::ON_SURFACE)),
        ])
        .gap(12.0),
    )
}

fn progress(media: &Media, phase: f32) -> Canvas {
    let width = 230.0;
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

fn dash(media: &Media, audio: &Audio, notifications: &Notifications) -> Box<dyn Widget> {
    let mut sections: Vec<Box<dyn Widget>> = Vec::new();

    sections.push(
        Box::new(
            Row::new(children![
                Column::new(children![
                    Text::new("bom te ver").size(12.0).color(theme::ON_SURFACE_VARIANT),
                    Text::new("seu espaço").size(27.0).color(theme::ON_SURFACE),
                    Text::new("tudo em um só lugar").size(12.0).color(theme::ON_SURFACE_VARIANT),
                ])
                .gap(2.0),
                Rectangle::new()
                    .width(54.0)
                    .height(54.0)
                    .radius(27.0)
                    .fill(theme::PRIMARY_CONTAINER)
                    .child(Text::new("✦").size(28.0).color(theme::ON_PRIMARY_CONTAINER)),
            ])
            .gap(12.0),
        )
    );

    sections.push(section_title("visão geral"));
    sections.push(system_cards(audio));

    sections.push(section_title("reprodução"));
    if media.title().is_empty() {
        sections.push(tile("nada tocando agora", "quando iniciar uma música, ela aparece aqui"));
    } else {
        sections.push(player(media));
    }

    sections.push(section_title("acesso rápido"));
    sections.push(app_launcher());

    sections.push(section_title("notificações"));
    if notifications.list().is_empty() {
        sections.push(tile("tudo em dia", "você não tem notificações novas"));
    } else {
        for notification in notifications.list().iter().rev().take(3) {
            let id = notification.id();
            let label = format!("{} · {}", notification.app_name(), notification.summary());
            let body = notification.body().to_string();
            sections.push(Box::new(
                Rectangle::new()
                    .width(Parent)
                    .height(58.0)
                    .radius(18.0)
                    .fill(theme::SURFACE_CONTAINER)
                    .padding(12.0)
                    .on_click(move |_| Notifications::click(id))
                    .child(
                        Column::new(children![
                            Text::new(label).size(12.0).color(theme::ON_SURFACE),
                            Text::new(body).size(11.0).color(theme::ON_SURFACE_VARIANT),
                        ])
                        .gap(3.0),
                    ),
            ));
        }
    }

    Box::new(ScrollArea::new("cds-dashboard", Column::new(sections).gap(12.0)))
}

fn system_cards(audio: &Audio) -> Box<dyn Widget> {
    let network = Network::read();
    let bluetooth = Bluetooth::read();
    let battery = Battery::read();
    let cpu = Cpu::read();
    let memory = Memory::read();
    let workspaces = Workspaces::read();

    let volume = audio.volume();
    let battery_label = if battery.present() { format!("{}%", battery.percent()) } else { "—".into() };
    let network_label = if network.connected() { "conectada" } else { "desconectada" };
    let bluetooth_label = if bluetooth.powered() { "ativo" } else { "inativo" };

    let rows: Vec<Box<dyn Widget>> = vec![
        Box::new(Row::new(children![
            info_card("áudio", &format!("{volume}%{}", if audio.muted() { " · mudo" } else { "" }), "◖", true),
            info_card("rede", network_label, "⌁", false),
        ]).gap(10.0)),
        Box::new(Row::new(children![
            info_card("bluetooth", bluetooth_label, "ᛒ", false),
            info_card("bateria", &battery_label, "▰", false),
        ]).gap(10.0)),
        Box::new(Row::new(children![
            info_card("processador", &format!("{}%", cpu.percent()), "◌", false),
            info_card("memória", &format!("{}%", memory.percent()), "▤", false),
        ]).gap(10.0)),
        Box::new(Row::new(children![
            info_card("espaços de trabalho", &format!("{} ativos", workspaces.list().len()), "▦", false),
            info_card("atalhos rápidos", "launcher · controles · notificações", "⌘", false),
        ]).gap(10.0)),
    ];

    Box::new(Column::new(rows).gap(10.0))
}

fn info_card(title: &str, value: &str, glyph: &str, volume: bool) -> Rectangle {
    let card = Rectangle::new()
        .width(Parent)
        .height(104.0)
        .radius(24.0)
        .fill(if volume { theme::PRIMARY_CONTAINER } else { theme::SURFACE_CONTAINER })
        .padding(14.0)
        .child(
            Column::new(children![
                Text::new(glyph).size(25.0).color(if volume { theme::ON_PRIMARY_CONTAINER } else { theme::PRIMARY }),
                Text::new(title).size(11.0).color(if volume { theme::ON_PRIMARY_CONTAINER } else { theme::ON_SURFACE_VARIANT }),
                Text::new(value).size(18.0).color(if volume { theme::ON_PRIMARY_CONTAINER } else { theme::ON_SURFACE }),
            ])
            .gap(1.0),
        );

    if volume {
        card.on_scroll(change_volume).on_click(|button| {
            if button == Button::Left {
                Audio::toggle_mute();
            }
        })
    } else {
        card
    }
}

fn app_launcher() -> Box<dyn Widget> {
    let apps = Apps::read();
    let mut entries: Vec<Box<dyn Widget>> = Vec::new();

    for app in apps.list().iter().take(6) {
        let launch = app.clone();
        entries.push(Box::new(
            Rectangle::new()
                .width(Parent)
                .height(42.0)
                .radius(14.0)
                .fill(theme::SURFACE_CONTAINER)
                .on_click(move |_| launch.launch())
                .padding(10.0)
                .child(Text::new(app.name()).size(12.0).color(theme::ON_SURFACE)),
        ));
    }

    if entries.is_empty() {
        entries.push(tile("carregando aplicativos", "a lista está sendo atualizada"));
    }

    Box::new(Column::new(entries).gap(6.0))
}

fn section_title(label: &str) -> Box<dyn Widget> {
    Box::new(Text::new(label).size(13.0).color(theme::PRIMARY))
}

fn tile(title: &str, detail: &str) -> Box<dyn Widget> {
    Box::new(
        Rectangle::new()
            .width(Parent)
            .height(58.0)
            .radius(18.0)
            .fill(theme::SURFACE_CONTAINER)
            .padding(12.0)
            .child(
                Column::new(children![
                    Text::new(title).size(12.0).color(theme::ON_SURFACE),
                    Text::new(detail).size(10.0).color(theme::ON_SURFACE_VARIANT),
                ])
                .gap(3.0),
            ),
    )
}

fn change_volume(scroll: Scroll) {
    let current = i32::from(Audio::read().volume());
    let step = if scroll.y < 0.0 { 5 } else { -5 };
    Audio::set_volume((current + step).clamp(0, 100) as u8);
}

fn toggle_dash(_: Button) {
    let mut state = ShellState::write();
    state.overlay = if state.overlay == Some(Overlay::Dashboard) { None } else { Some(Overlay::Dashboard) };
}

fn key_pressed(key: Key) {
    if key == Key::Escape {
        let mut state = ShellState::write();
        if state.overlay == Some(Overlay::Dashboard) {
            state.overlay = None;
        }
    }
}

fn island_clicked(button: Button) {
    if button == Button::Middle {
        Media::play_pause();
    }
}

fn local_art(url: &str) -> Option<&str> {
    let path = url.strip_prefix("file://").unwrap_or(url);
    if path.starts_with('/') && Path::new(path).exists() { Some(path) } else { None }
}

static EPOCH: OnceLock<Instant> = OnceLock::new();

thread_local! {
    static WIDTH: RefCell<Option<Animation<f32>>> = const { RefCell::new(None) };
    static HEIGHT: RefCell<Option<Animation<f32>>> = const { RefCell::new(None) };
}

fn animated_size(animation: &'static std::thread::LocalKey<RefCell<Option<Animation<f32>>>>, target: f32) -> f32 {
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
