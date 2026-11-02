use std::path::Path;
use std::sync::OnceLock;
use std::time::Instant;

use amane::{
    Service,
    Apps, Audio, Battery, Bluetooth, Button, Canvas, Cap, Column, Cpu, Image, Layer, LayerWindow,
    Media, Memory, Network, Notifications, Parent, Path as DrawPath, Rectangle, Row, Scroll,
    ScrollArea, Shape, Text, Vertical, Widget, Workspaces, children, request_frame, shapes,
};

use crate::shell::state::{Overlay, ShellState};
use crate::theme;

pub fn view() -> LayerWindow {
    let state = ShellState::read();
    let media = Media::read();
    let audio = Audio::read();
    let notifications = Notifications::read();

    let expanded = state.overlay == Some(Overlay::Dashboard);
    let has_media = !media.title().is_empty();
    let height = if expanded {
        520.0
    } else if has_media {
        94.0
    } else {
        52.0
    };
    let width = if expanded {
        620.0
    } else if has_media {
        420.0
    } else {
        230.0
    };

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
                .radius(26.0)
                .fill(theme::SURFACE)
                .border(1.0, theme::OUTLINE)
                .padding(12.0)
                .on_click(island_clicked)
                .on_scroll(change_volume)
                .child(Column::new(vec![content])),
        )
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
            .radius(18.0)
            .fill(theme::SURFACE_CONTAINER)
            .on_click(toggle_dash)
            .child(
                Row::new(children![
                    Text::new("◈").size(18.0).color(theme::PRIMARY),
                    Text::new(label).size(14.0).color(theme::ON_SURFACE),
                    Text::new("⌄").size(16.0).color(theme::ON_SURFACE_VARIANT),
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
            .radius_top_left(18.0)
            .radius_top_right(10.0)
            .radius_bottom_right(18.0)
            .radius_bottom_left(10.0)
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

    let title = if media.title().is_empty() {
        "sem título"
    } else {
        media.title()
    };
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
                Text::new(artist)
                    .size(12.0)
                    .color(theme::ON_SURFACE_VARIANT),
                progress(media, phase),
            ])
            .gap(3.0),
            Rectangle::new()
                .width(42.0)
                .height(42.0)
                .radius(21.0)
                .fill(theme::SURFACE_CONTAINER)
                .on_click(|_| Media::play_pause())
                .child(
                    Text::new(if media.playing() { "Ⅱ" } else { "▶" })
                        .size(17.0)
                        .color(theme::PRIMARY)
                ),
            Rectangle::new()
                .width(28.0)
                .height(42.0)
                .radius(14.0)
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

    let worm = wave.stroke(3.0, theme::PRIMARY).cap(Cap::Round);
    let tail = DrawPath::new()
        .move_to(end + 4.0, 7.0)
        .line_to(width, 7.0)
        .stroke(2.0, theme::OUTLINE)
        .cap(Cap::Round);

    Canvas::new()
        .width(width)
        .height(14.0)
        .shapes(shapes![tail, worm])
}

fn dash(media: &Media, audio: &Audio, notifications: &Notifications) -> Box<dyn Widget> {
    let mut sections: Vec<Box<dyn Widget>> = Vec::new();

    sections.push(section_title("controles"));
    sections.push(system_controls(audio));
    sections.push(section_title("tray"));
    sections.push(tile(
        "área de notificação",
        "integração status notifier pendente",
    ));
    sections.push(section_title("apps"));
    sections.push(app_launcher());
    sections.push(section_title("clipboard"));
    sections.push(tile("área de transferência", "histórico em integração"));
    sections.push(section_title("clima"));
    sections.push(tile("clima", "provedor meteorológico não configurado"));
    sections.push(section_title("mídia"));
    if media.title().is_empty() {
        sections.push(tile(
            "nenhuma mídia ativa",
            "abra um player compatível com mpris",
        ));
    } else {
        sections.push(player(media));
    }
    sections.push(section_title("notificações"));

    if notifications.list().is_empty() {
        sections.push(tile("tudo tranquilo", "nenhuma notificação"));
    } else {
        for notification in notifications.list().iter().rev().take(3) {
            let id = notification.id();
            let label = format!("{} · {}", notification.app_name(), notification.summary());
            let body = notification.body().to_string();
            sections.push(Box::new(
                Rectangle::new()
                    .width(Parent)
                    .height(52.0)
                    .radius(14.0)
                    .fill(theme::SURFACE_CONTAINER)
                    .padding(10.0)
                    .on_click(move |_| Notifications::click(id))
                    .child(
                        Column::new(children![
                            Text::new(label).size(12.0).color(theme::ON_SURFACE),
                            Text::new(body).size(11.0).color(theme::ON_SURFACE_VARIANT),
                        ])
                        .gap(2.0),
                    ),
            ));
        }
    }

    Box::new(ScrollArea::new(
        "island-dash",
        Column::new(sections).gap(8.0),
    ))
}

fn system_controls(audio: &Audio) -> Box<dyn Widget> {
    let volume = audio.volume();
    let muted = audio.muted();
    let network = Network::read();
    let bluetooth = Bluetooth::read();
    let battery = Battery::read();
    let cpu = Cpu::read();
    let memory = Memory::read();
    let workspaces = Workspaces::read();

    let status = format!(
        "volume {volume}%{} · rede {} · bluetooth {} · bateria {} · cpu {}% · ram {}%",
        if muted { " (mudo)" } else { "" },
        if network.connected() {
            "conectada"
        } else {
            "desconectada"
        },
        if bluetooth.powered() {
            "ativo"
        } else {
            "inativo"
        },
        if battery.present() {
            format!("{}%", battery.percent())
        } else {
            String::from("—")
        },
        cpu.percent(),
        memory.percent(),
    );
    let workspace_count = workspaces.list().len();

    let controls: Vec<Box<dyn Widget>> = vec![
        Box::new(
            Rectangle::new()
                .width(Parent)
                .height(40.0)
                .radius(14.0)
                .fill(theme::SURFACE_CONTAINER)
                .on_scroll(change_volume)
                .on_click(|button| {
                    if button == Button::Left {
                        Audio::toggle_mute();
                    }
                })
                .padding(10.0)
                .child(Text::new(status).size(11.0).color(theme::ON_SURFACE)),
        ),
        tile(
            "áreas de trabalho",
            &format!("{workspace_count} espaços detectados"),
        ),
    ];

    Box::new(Column::new(controls).gap(6.0))
}

fn app_launcher() -> Box<dyn Widget> {
    let apps = Apps::read();
    let mut entries: Vec<Box<dyn Widget>> = Vec::new();

    for app in apps.list().iter().take(8) {
        let launch = app.clone();
        entries.push(Box::new(
            Rectangle::new()
                .width(Parent)
                .height(34.0)
                .radius(10.0)
                .fill(theme::SURFACE_CONTAINER)
                .on_click(move |_| launch.launch())
                .padding(8.0)
                .child(Text::new(app.name()).size(12.0).color(theme::ON_SURFACE)),
        ));
    }

    if entries.is_empty() {
        entries.push(tile(
            "carregando aplicativos",
            "a lista é atualizada em segundo plano",
        ));
    }

    Box::new(Column::new(entries).gap(4.0))
}

fn section_title(label: &str) -> Box<dyn Widget> {
    Box::new(Text::new(label).size(11.0).color(theme::PRIMARY))
}

fn tile(title: &str, detail: &str) -> Box<dyn Widget> {
    Box::new(
        Rectangle::new()
            .width(Parent)
            .height(42.0)
            .radius(12.0)
            .fill(theme::SURFACE_CONTAINER)
            .padding(8.0)
            .child(
                Column::new(children![
                    Text::new(title).size(12.0).color(theme::ON_SURFACE),
                    Text::new(detail)
                        .size(10.0)
                        .color(theme::ON_SURFACE_VARIANT),
                ])
                .gap(1.0),
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
    state.overlay = if state.overlay == Some(Overlay::Dashboard) {
        None
    } else {
        Some(Overlay::Dashboard)
    };
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
