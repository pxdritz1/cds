use amane::{
    Audio, Battery, Brightness, Center, Color, Column, Cpu, End, Image, Media, Memory,
    Notifications, Parent, Pointer, Rectangle, Row, ScrollArea, Service, SpaceBetween, Text,
    Weight, Widget,
};

use crate::clock::Clock;
use crate::settings::Settings;
use crate::theme::Scheme;
use crate::ui::{Tab, Ui};
use crate::art::cover;

use super::widgets::{
    BELL, BRIGHTNESS, CLOSE, MUTED, NEXT, PAUSE, PLAY, PREVIOUS, VOLUME, boxed, card, chip,
    icon, label, round_button, slider,
};

pub const WIDTH: f32 = 680.0;
pub const HEIGHT: f32 = 372.0;
pub const BODY: f32 = 320.0;

pub fn view(scheme: &Scheme, tab: Tab) -> Box<dyn Widget> {
    let tabs = [
        (Tab::Overview, "Overview"),
        (Tab::Media, "Media"),
        (Tab::Wallpapers, "Wallpapers"),
        (Tab::Alerts, "Alerts"),
    ];

    let header: Vec<Box<dyn Widget>> = tabs
        .iter()
        .map(|&(target, name)| {
            boxed(chip(name, 108.0, tab == target, scheme, move || {
                Ui::select(target)
            }))
        })
        .collect();

    let body = match tab {
        Tab::Overview => overview(scheme),
        Tab::Media => media(scheme),
        Tab::Wallpapers => wallpapers(scheme),
        Tab::Alerts => alerts(scheme),
    };

    boxed(
        Rectangle::new()
            .width(Parent)
            .height(Parent)
            .padding(20.0)
            .child(
                Column::new(vec![boxed(Row::new(header).gap(6.0).align(Center)), body])
                    .width(Parent)
                    .gap(16.0),
            ),
    )
}

fn overview(scheme: &Scheme) -> Box<dyn Widget> {
    boxed(
        Row::new(vec![
            boxed(time_card(scheme)),
            boxed(media_card(scheme)),
            boxed(controls_card(scheme)),
        ])
        .gap(12.0),
    )
}

fn time_card(scheme: &Scheme) -> Rectangle {
    let clock = Clock::read();

    let color = scheme.on_primary_container;

    card(190.0, scheme.primary_container).child(
        Column::new(vec![
            boxed(label(clock.greeting(), 14.0, color, Weight::Medium, 158.0)),
            boxed(
                Text::new(clock.time())
                    .size(56.0)
                    .weight(Weight::SemiBold)
                    .color(color),
            ),
            boxed(label(clock.date(), 13.0, color, Weight::Regular, 158.0)),
        ])
        .gap(8.0)
        .height(Parent)
        .justify(End),
    )
}

fn media_card(scheme: &Scheme) -> Rectangle {
    let media = Media::read();

    let (title, artist) = if media.active().is_some() {
        (media.title(), media.artist())
    } else {
        ("Nothing playing", "")
    };

    let glyph = if media.playing() { PAUSE } else { PLAY };

    let controls = Row::new(vec![
        boxed(round_button(
            PREVIOUS,
            38.0,
            scheme.secondary_container,
            scheme.on_secondary_container,
            Media::previous,
        )),
        boxed(round_button(
            glyph,
            46.0,
            scheme.primary,
            scheme.on_primary,
            Media::play_pause,
        )),
        boxed(round_button(
            NEXT,
            38.0,
            scheme.secondary_container,
            scheme.on_secondary_container,
            Media::next,
        )),
    ])
    .gap(10.0)
    .align(Center)
    .justify(Center)
    .width(198.0);

    card(230.0, scheme.surface_container_highest).child(
        Column::new(vec![
            boxed(cover(118.0, 24.0, scheme)),
            boxed(label(
                title,
                15.0,
                scheme.on_surface,
                Weight::SemiBold,
                198.0,
            )),
            boxed(label(
                artist,
                12.0,
                scheme.on_surface_variant,
                Weight::Regular,
                198.0,
            )),
            boxed(controls),
        ])
        .gap(8.0)
        .align(Center)
        .width(198.0),
    )
}

fn controls_card(scheme: &Scheme) -> Rectangle {
    let audio = Audio::read();
    let brightness = Brightness::read();
    let battery = Battery::read();

    let volume_glyph = if audio.muted() { MUTED } else { VOLUME };

    let mut rows: Vec<Box<dyn Widget>> = vec![boxed(
        Row::new(vec![
            boxed(
                Rectangle::new()
                    .width(22.0)
                    .height(22.0)
                    .align_child(Center, Center)
                    .cursor(Pointer)
                    .on_click(|_| Audio::toggle_mute())
                    .child(icon(volume_glyph, 18.0, scheme.on_surface)),
            ),
            boxed(slider(audio.volume().into(), 130.0, scheme, |v: f32| Audio::set_volume(v as u8))),
        ])
        .gap(10.0)
        .align(Center),
    )];

    if brightness.present() {
        rows.push(boxed(
            Row::new(vec![
                boxed(
                    Rectangle::new()
                        .width(22.0)
                        .height(22.0)
                        .align_child(Center, Center)
                        .child(icon(BRIGHTNESS, 18.0, scheme.on_surface)),
                ),
                boxed(slider(brightness.percent().into(), 130.0, scheme, |v: f32| Brightness::set(v as u8))),
            ])
            .gap(10.0)
            .align(Center),
        ));
    }

    rows.push(stat("CPU", Cpu::read().percent(), scheme));
    rows.push(stat("RAM", Memory::read().percent(), scheme));

    if battery.present() {
        rows.push(stat("BAT", battery.percent(), scheme));
    }

    card(196.0, scheme.surface_container_highest).child(Column::new(rows).gap(16.0).width(164.0))
}

fn stat(name: &str, percent: u8, scheme: &Scheme) -> Box<dyn Widget> {
    boxed(
        Row::new(vec![
            boxed(label(
                name,
                12.0,
                scheme.on_surface_variant,
                Weight::Medium,
                44.0,
            )),
            boxed(
                Rectangle::new()
                    .width(80.0)
                    .height(6.0)
                    .fill(scheme.secondary_container)
                    .radius(3.0)
                    .clip()
                    .child(
                        Rectangle::new()
                            .width((80.0 * f32::from(percent.min(100)) / 100.0).max(6.0))
                            .height(6.0)
                            .fill(scheme.tertiary)
                            .radius(3.0),
                    ),
            ),
            boxed(label(
                format!("{percent}%"),
                12.0,
                scheme.on_surface,
                Weight::SemiBold,
                36.0,
            )),
        ])
        .gap(6.0)
        .align(Center),
    )
}

fn media(scheme: &Scheme) -> Box<dyn Widget> {
    let media = Media::read();

    let glyph = if media.playing() { PAUSE } else { PLAY };

    let length = media.length().as_secs();
    let position = media.position().as_secs();

    let progress = if length == 0 {
        0
    } else {
        (position * 100 / length).min(100) as u8
    };

    let (title, artist) = if media.active().is_some() {
        (media.title(), media.artist())
    } else {
        ("Nothing playing", "")
    };

    let times = Row::new(vec![
        boxed(label(
            format!("{position}"),
            12.0,
            scheme.on_surface_variant,
            Weight::Medium,
            60.0,
        )),
        boxed(label(
            format!("{length}"),
            12.0,
            scheme.on_surface_variant,
            Weight::Medium,
            60.0,
        )),
    ])
    .justify(SpaceBetween)
    .width(360.0);

    let controls = Row::new(vec![
        boxed(round_button(
            PREVIOUS,
            52.0,
            scheme.secondary_container,
            scheme.on_secondary_container,
            Media::previous,
        )),
        boxed(round_button(
            glyph,
            64.0,
            scheme.primary,
            scheme.on_primary,
            Media::play_pause,
        )),
        boxed(round_button(
            NEXT,
            52.0,
            scheme.secondary_container,
            scheme.on_secondary_container,
            Media::next,
        )),
    ])
    .gap(16.0)
    .align(Center);

    let info = Column::new(vec![
        boxed(label(
            title,
            26.0,
            scheme.on_surface,
            Weight::SemiBold,
            360.0,
        )),
        boxed(label(
            artist,
            16.0,
            scheme.on_surface_variant,
            Weight::Regular,
            360.0,
        )),
        boxed(slider(progress.into(), 360.0, scheme, |_| {})),
        boxed(times),
        boxed(controls),
    ])
    .gap(14.0)
    .width(360.0);

    boxed(
        card(640.0, scheme.surface_container_highest).child(
            Row::new(vec![boxed(cover(240.0, 32.0, scheme)), boxed(info)])
                .gap(24.0)
                .align(Center),
        ),
    )
}

fn wallpapers(scheme: &Scheme) -> Box<dyn Widget> {
    let settings = Settings::read();

    let modes = Row::new(vec![
        boxed(chip("Dark", 84.0, settings.dark(), scheme, || {
            Settings::set_dark(true)
        })),
        boxed(chip("Light", 84.0, !settings.dark(), scheme, || {
            Settings::set_dark(false)
        })),
    ])
    .gap(6.0);

    let current = settings.wallpaper();

    let rows: Vec<Box<dyn Widget>> = settings
        .wallpapers()
        .chunks(3)
        .map(|chunk| {
            let thumbs: Vec<Box<dyn Widget>> = chunk
                .iter()
                .map(|path| boxed(thumb(path, path == current, scheme)))
                .collect();

            boxed(Row::new(thumbs).gap(10.0))
        })
        .collect();

    let grid: Box<dyn Widget> = if rows.is_empty() {
        boxed(label(
            "Put images in ~/Pictures/Wallpapers",
            14.0,
            scheme.on_surface_variant,
            Weight::Regular,
            400.0,
        ))
    } else {
        boxed(Column::new(rows).gap(10.0))
    };

    boxed(
        Column::new(vec![
            boxed(modes),
            boxed(
                ScrollArea::new("wallpapers", Column::new(vec![grid]))
                    .width(640.0)
                    .height(BODY - 52.0),
            ),
        ])
        .gap(16.0),
    )
}

fn thumb(path: &str, selected: bool, scheme: &Scheme) -> Rectangle {
    let owned = String::from(path);

    let thumb = Rectangle::new()
        .width(204.0)
        .height(115.0)
        .radius(18.0)
        .clip()
        .fill(Image::cover(path).thumbnail(408, 230))
        .cursor(Pointer)
        .on_click(move |_| Settings::set_wallpaper(&owned));

    if selected {
        thumb.border(3.0, scheme.primary)
    } else {
        thumb
    }
}

fn alerts(scheme: &Scheme) -> Box<dyn Widget> {
    let notifications = Notifications::read();

    let list = notifications.list();

    let header = Row::new(vec![
        boxed(label(
            format!("{} notifications", list.len()),
            14.0,
            scheme.on_surface_variant,
            Weight::Medium,
            200.0,
        )),
        boxed(chip(
            "Clear all",
            104.0,
            false,
            scheme,
            Notifications::clear,
        )),
    ])
    .justify(SpaceBetween)
    .align(Center)
    .width(640.0);

    let cards: Vec<Box<dyn Widget>> = list
        .iter()
        .rev()
        .map(|notification| {
            let id = notification.id();

            let bell = Rectangle::new()
                .width(40.0)
                .height(40.0)
                .fill(scheme.primary_container)
                .radius(20.0)
                .align_child(Center, Center)
                .child(icon(BELL, 18.0, scheme.on_primary_container));

            let info = Column::new(vec![
                boxed(label(
                    notification.summary(),
                    14.0,
                    scheme.on_surface,
                    Weight::SemiBold,
                    480.0,
                )),
                boxed(label(
                    notification.body(),
                    12.0,
                    scheme.on_surface_variant,
                    Weight::Regular,
                    480.0,
                )),
            ]);

            boxed(
                Rectangle::new()
                    .width(628.0)
                    .height(64.0)
                    .fill(scheme.surface_container_high)
                    .radius(20.0)
                    .padding(12.0)
                    .child(
                        Row::new(vec![
                            boxed(bell),
                            boxed(info),
                            boxed(round_button(
                                CLOSE,
                                32.0,
                                Color::TRANSPARENT,
                                scheme.on_surface_variant,
                                move || Notifications::dismiss(id),
                            )),
                        ])
                        .gap(12.0)
                        .align(Center)
                        .justify(SpaceBetween)
                        .width(Parent),
                    ),
            )
        })
        .collect();

    let body: Box<dyn Widget> = if cards.is_empty() {
        boxed(label(
            "You're all caught up",
            14.0,
            scheme.on_surface_variant,
            Weight::Regular,
            300.0,
        ))
    } else {
        boxed(Column::new(cards).gap(8.0))
    };

    boxed(
        Column::new(vec![
            boxed(header),
            boxed(
                ScrollArea::new("alerts", Column::new(vec![body]))
                    .width(640.0)
                    .height(BODY - 52.0),
            ),
        ])
        .gap(16.0),
    )
}
