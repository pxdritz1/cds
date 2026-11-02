use amane::{
    Apps, Audio, Battery, Brightness, Center, Column, Cpu, End, Image, Media, Memory,
    Notifications, Parent, Pointer, Rectangle, Row, ScrollArea, Service, SpaceBetween, Widget,
};

use crate::art::{artwork, media_labels};
use crate::clock::Clock;
use crate::settings::Settings;
use crate::theme::{Scheme, mix};
use crate::ui::{Tab, Ui};

use super::widgets::{
    APPS, BELL, BRIGHTNESS, CLOSE, MUTED, NEXT, PAUSE, PLAY, PREVIOUS, VOLUME, WALLPAPER,
    boxed, card, chip, icon, icon_button, label, progress, round_button, slider, text,
};

pub const WIDTH: f32 = 700.0;
pub const HEIGHT: f32 = 404.0;
pub const RADIUS: f32 = 28.0;

const PADDING: f32 = 20.0;
const TAB_HEIGHT: f32 = 36.0;
const TAB_GAP: f32 = 6.0;
const BODY: f32 = HEIGHT - PADDING * 2.0 - TAB_HEIGHT - 16.0;

pub fn view(scheme: &Scheme, tab: Tab, alpha: f32) -> Rectangle {
    let tabs = [
        (Tab::Overview, "Overview"),
        (Tab::Media, "Media"),
        (Tab::Wallpapers, "Wallpapers"),
        (Tab::Alerts, "Alerts"),
        (Tab::Apps, "Apps"),
    ];

    let header: Vec<Box<dyn Widget>> = tabs
        .iter()
        .map(|&(target, name)| {
            let key = format!("tab:{name}");

            boxed(chip(&key, name, 104.0, tab == target, scheme, move || {
                Ui::select(target)
            }))
        })
        .collect();

    let body = match tab {
        Tab::Overview => overview(scheme),
        Tab::Media => media(scheme),
        Tab::Wallpapers => wallpapers(scheme),
        Tab::Alerts => alerts(scheme),
        Tab::Apps => apps(scheme),
    };

    // the body fades whenever the tab changes, after the header is built
    let alpha = alpha * crate::motion::fade("tab", tab.index());

    Rectangle::new()
        .width(Parent)
        .height(Parent)
        .padding(PADDING)
        .opacity(alpha)
        .child(
            Column::new(vec![
                boxed(Row::new(header).gap(TAB_GAP).align(Center)),
                body,
            ])
            .width(Parent)
            .gap(16.0),
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

    card(200.0, scheme.primary_container).child(
        Column::new(vec![
            boxed(label(clock.greeting(), 14.0, color, amane::Weight::Medium, 1)),
            boxed(text(
                clock.time(),
                54.0,
                color,
                amane::Weight::SemiBold,
            )),
            boxed(label(clock.date(), 12.5, color, amane::Weight::Regular, 1)),
        ])
        .gap(6.0)
        .width(Parent)
        .height(Parent)
        .justify(End),
    )
}

fn media_card(scheme: &Scheme) -> Rectangle {
    let media = Media::read();

    let (title, artist) = media_labels(&media);

    let glyph = if media.playing() { PAUSE } else { PLAY };

    let controls = Row::new(vec![
        boxed(round_button(
            "card-previous",
            PREVIOUS,
            38.0,
            scheme.secondary_container,
            scheme.on_secondary_container,
            Media::previous,
        )),
        boxed(round_button(
            "card-play",
            glyph,
            44.0,
            scheme.primary,
            scheme.on_primary,
            Media::play_pause,
        )),
        boxed(round_button(
            "card-next",
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
    .width(Parent);

    card(240.0, scheme.surface_container_high).child(
        Column::new(vec![
            boxed(
                Row::new(vec![boxed(artwork(media.art_url(), 128.0, 24.0, scheme))])
                    .width(Parent)
                    .justify(Center),
            ),
            boxed(label(
                title,
                15.0,
                scheme.on_surface,
                amane::Weight::SemiBold,
                1,
            )),
            boxed(label(
                artist,
                12.0,
                scheme.on_surface_variant,
                amane::Weight::Regular,
                1,
            )),
            boxed(controls),
        ])
        .gap(9.0)
        .width(Parent)
        .height(Parent)
        .justify(SpaceBetween),
    )
}

fn controls_card(scheme: &Scheme) -> Rectangle {
    let audio = Audio::read();
    let brightness = Brightness::read();
    let battery = Battery::read();

    let volume_glyph = if audio.muted() { MUTED } else { VOLUME };

    let mut rows: Vec<Box<dyn Widget>> = vec![boxed(
        Row::new(vec![
            boxed(icon_button(
                "mute",
                volume_glyph,
                26.0,
                scheme,
                Audio::toggle_mute,
            )),
            boxed(slider(
                "volume",
                f32::from(audio.volume()) / 100.0,
                132.0,
                scheme,
                |value| Audio::set_volume((value * 100.0).round() as u8),
            )),
        ])
        .gap(6.0)
        .align(Center),
    )];

    if brightness.present() {
        rows.push(boxed(
            Row::new(vec![
                boxed(
                    Rectangle::new()
                        .width(26.0)
                        .height(26.0)
                        .align_child(Center, Center)
                        .child(icon(BRIGHTNESS, 18.0, scheme.on_surface_variant)),
                ),
                boxed(slider(
                    "brightness",
                    f32::from(brightness.percent()) / 100.0,
                    132.0,
                    scheme,
                    |value| Brightness::set((value * 100.0).round() as u8),
                )),
            ])
            .gap(6.0)
            .align(Center),
        ));
    }

    rows.push(stat("CPU", Cpu::read().percent(), scheme));
    rows.push(stat("RAM", Memory::read().percent(), scheme));

    if battery.present() {
        rows.push(stat("BAT", battery.percent(), scheme));
    }

    card(196.0, scheme.surface_container_high).child(
        Column::new(rows)
            .gap(14.0)
            .width(Parent)
            .height(Parent)
            .justify(Center),
    )
}

fn stat(name: &str, percent: u8, scheme: &Scheme) -> Box<dyn Widget> {
    boxed(
        Column::new(vec![
            boxed(
                Row::new(vec![
                    boxed(text(
                        name,
                        11.0,
                        scheme.on_surface_variant,
                        amane::Weight::Medium,
                    )),
                    boxed(text(
                        format!("{percent}%"),
                        11.0,
                        scheme.on_surface,
                        amane::Weight::SemiBold,
                    )),
                ])
                .justify(SpaceBetween)
                .width(Parent),
            ),
            boxed(meter(name, percent, 164.0, scheme)),
        ])
        .gap(5.0)
        .width(Parent),
    )
}

// a thin progress bar that grows toward its value instead of snapping
fn meter(key: &str, percent: u8, width: f32, scheme: &Scheme) -> Rectangle {
    let target = f32::from(percent.min(100)) / 100.0;

    let fraction = crate::motion::number(
        &format!("meter:{key}"),
        target,
        std::time::Duration::from_millis(320),
    );

    Rectangle::new()
        .width(width)
        .height(5.0)
        .fill(scheme.surface_container_highest)
        .radius(2.5)
        .clip()
        .child(
            Rectangle::new()
                .width((width * fraction).max(5.0))
                .height(Parent)
                .fill(scheme.tertiary)
                .radius(2.5),
        )
}

fn media(scheme: &Scheme) -> Box<dyn Widget> {
    let media = Media::read();

    let glyph = if media.playing() { PAUSE } else { PLAY };

    let length = media.length().as_secs();
    let position = media.position().as_secs();

    let fraction = if length == 0 {
        0.0
    } else {
        (position as f32 / length as f32).clamp(0.0, 1.0)
    };

    let (title, artist) = media_labels(&media);

    let times = Row::new(vec![
        boxed(text(
            format_time(position),
            12.0,
            scheme.on_surface_variant,
            amane::Weight::Medium,
        )),
        boxed(text(
            format_time(length),
            12.0,
            scheme.on_surface_variant,
            amane::Weight::Medium,
        )),
    ])
    .justify(SpaceBetween)
    .width(Parent);

    let controls = Row::new(vec![
        boxed(round_button(
            "full-previous",
            PREVIOUS,
            52.0,
            scheme.secondary_container,
            scheme.on_secondary_container,
            Media::previous,
        )),
        boxed(round_button(
            "full-play",
            glyph,
            64.0,
            scheme.primary,
            scheme.on_primary,
            Media::play_pause,
        )),
        boxed(round_button(
            "full-next",
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
            amane::Weight::SemiBold,
            1,
        )),
        boxed(label(
            artist,
            15.0,
            scheme.on_surface_variant,
            amane::Weight::Regular,
            1,
        )),
        boxed(progress("seek", fraction, 340.0, scheme)),
        boxed(times),
        boxed(controls),
    ])
    .gap(14.0)
    .width(340.0)
    .height(Parent)
    .justify(Center);

    boxed(
        card(660.0, scheme.surface_container_high).child(
            Row::new(vec![
                boxed(artwork(media.art_url(), 260.0, 28.0, scheme)),
                boxed(info),
            ])
            .gap(28.0)
            .align(Center),
        ),
    )
}

fn wallpapers(scheme: &Scheme) -> Box<dyn Widget> {
    let settings = Settings::read();

    let modes = Row::new(vec![
        boxed(chip("mode:dark", "Dark", 88.0, settings.dark(), scheme, || {
            Settings::set_dark(true)
        })),
        boxed(chip("mode:light", "Light", 88.0, !settings.dark(), scheme, || {
            Settings::set_dark(false)
        })),
    ])
    .gap(TAB_GAP);

    let current = settings.wallpaper();

    let rows: Vec<Box<dyn Widget>> = settings
        .wallpapers()
        .chunks(3)
        .map(|chunk| {
            let thumbs: Vec<Box<dyn Widget>> = chunk
                .iter()
                .map(|path| boxed(thumb(path, path == current, scheme)))
                .collect();

            boxed(Row::new(thumbs).gap(12.0))
        })
        .collect();

    let grid: Box<dyn Widget> = if rows.is_empty() {
        boxed(empty(
            WALLPAPER,
            "No wallpapers yet",
            "Put images in ~/Pictures/Wallpapers",
            scheme,
        ))
    } else {
        boxed(Column::new(rows).gap(12.0))
    };

    boxed(
        Column::new(vec![
            boxed(modes),
            boxed(
                ScrollArea::new("wallpapers", Column::new(vec![grid]).width(660.0))
                    .width(660.0)
                    .height(BODY - TAB_HEIGHT - 16.0),
            ),
        ])
        .gap(16.0),
    )
}

fn thumb(path: &str, selected: bool, scheme: &Scheme) -> Rectangle {
    let owned = String::from(path);

    let thumb = Rectangle::new()
        .width(208.0)
        .height(117.0)
        .radius(18.0)
        .clip()
        .fill(Image::cover(path).thumbnail(416, 234))
        .cursor(Pointer)
        .on_click(move |_| {
            Settings::set_wallpaper(&owned);
        });

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
        boxed(text(
            format!("{} notifications", list.len()),
            13.0,
            scheme.on_surface_variant,
            amane::Weight::Medium,
        )),
        boxed(chip(
            "alerts-clear",
            "Clear all",
            104.0,
            false,
            scheme,
            Notifications::clear,
        )),
    ])
    .justify(SpaceBetween)
    .align(Center)
    .width(Parent);

    let cards: Vec<Box<dyn Widget>> = list
        .iter()
        .rev()
        .map(|notification| boxed(alert_card(scheme, notification)))
        .collect();

    let body: Box<dyn Widget> = if cards.is_empty() {
        boxed(empty(
            BELL,
            "You're all caught up",
            "New notifications show up here",
            scheme,
        ))
    } else {
        boxed(Column::new(cards).gap(8.0).width(660.0))
    };

    boxed(
        Column::new(vec![
            boxed(header),
            boxed(
                ScrollArea::new("alerts", Column::new(vec![body]).width(660.0))
                    .width(660.0)
                    .height(BODY - TAB_HEIGHT - 12.0),
            ),
        ])
        .gap(12.0),
    )
}

fn alert_card(scheme: &Scheme, notification: &amane::Notification) -> Rectangle {
    let id = notification.id();

    let bell = Rectangle::new()
        .width(40.0)
        .height(40.0)
        .fill(scheme.primary_container)
        .radius(20.0)
        .align_child(Center, Center)
        .child(icon(BELL, 20.0, scheme.on_primary_container));

    let info = Column::new(vec![
        boxed(label(
            notification.summary(),
            13.5,
            scheme.on_surface,
            amane::Weight::SemiBold,
            1,
        )),
        boxed(label(
            notification.body(),
            11.5,
            scheme.on_surface_variant,
            amane::Weight::Regular,
            2,
        )),
    ])
    .gap(1.0)
    .width(Parent);

    Rectangle::new()
        .width(648.0)
        .height(74.0)
        .fill(scheme.surface_container_high)
        .radius(18.0)
        .padding(12.0)
        .child(
            Row::new(vec![
                boxed(bell),
                boxed(info),
                boxed(icon_button("alert-close", CLOSE, 32.0, scheme, move || {
                    Notifications::dismiss(id)
                })),
            ])
            .gap(12.0)
            .align(Center)
            .width(Parent),
        )
}

fn apps(scheme: &Scheme) -> Box<dyn Widget> {
    let apps = Apps::read();
    let list = apps.list();

    let rows: Vec<Box<dyn Widget>> = list
        .iter()
        .map(|app| boxed(app_row(scheme, app)))
        .collect();

    let body: Box<dyn Widget> = if rows.is_empty() {
        boxed(empty(
            APPS,
            "Looking for applications",
            "The launcher fills in after the first scan",
            scheme,
        ))
    } else {
        boxed(Column::new(rows).gap(6.0).width(660.0))
    };

    boxed(
        ScrollArea::new("apps", Column::new(vec![body]).width(660.0))
            .width(660.0)
            .height(BODY),
    )
}

fn app_row(scheme: &Scheme, app: &amane::DesktopApp) -> Rectangle {
    let key = format!("app:{}", app.name());
    let state = crate::motion::hover(&key);

    let fill = mix(
        scheme.surface_container_high,
        scheme.on_surface,
        0.08 * state,
    );

    let icon_box: Box<dyn Widget> = match app.icon_path() {
        Some(path) if Image::loaded(path) => boxed(
            Rectangle::new()
                .width(36.0)
                .height(36.0)
                .radius(10.0)
                .clip()
                .fill(Image::cover(path).thumbnail(72, 72)),
        ),
        _ => boxed(
            Rectangle::new()
                .width(36.0)
                .height(36.0)
                .radius(10.0)
                .fill(scheme.secondary_container)
                .align_child(Center, Center)
                .child(icon(APPS, 20.0, scheme.on_secondary_container)),
        ),
    };

    let description = app.description().unwrap_or("Application");

    let info = Column::new(vec![
        boxed(label(
            app.name(),
            13.5,
            scheme.on_surface,
            amane::Weight::Medium,
            1,
        )),
        boxed(label(
            description,
            11.0,
            scheme.on_surface_variant,
            amane::Weight::Regular,
            1,
        )),
    ])
    .gap(1.0)
    .width(Parent);

    let owned = key.clone();
    let exec = String::from(app.exec());

    Rectangle::new()
        .width(648.0)
        .height(56.0)
        .fill(fill)
        .radius(16.0)
        .padding(10.0)
        .cursor(Pointer)
        .on_hover(move |inside| crate::motion::set_hover(owned.clone(), inside))
        .on_click(move |_| amane::spawn(&exec))
        .child(
            Row::new(vec![icon_box, boxed(info)])
                .gap(12.0)
                .align(Center)
                .width(Parent),
        )
}

// a centered icon and two lines, for lists that have nothing to show yet
fn empty(glyph: &str, title: &str, body: &str, scheme: &Scheme) -> Rectangle {
    let circle = Rectangle::new()
        .width(64.0)
        .height(64.0)
        .fill(scheme.surface_container_highest)
        .radius(32.0)
        .align_child(Center, Center)
        .child(icon(glyph, 30.0, scheme.on_surface_variant));

    Rectangle::new()
        .width(660.0)
        .height(BODY - 40.0)
        .align_child(Center, Center)
        .child(
            Column::new(vec![
                boxed(Row::new(vec![boxed(circle)]).width(Parent).justify(Center)),
                boxed(
                    Row::new(vec![boxed(text(
                        title,
                        15.0,
                        scheme.on_surface,
                        amane::Weight::Medium,
                    ))])
                    .width(Parent)
                    .justify(Center),
                ),
                boxed(
                    Row::new(vec![boxed(text(
                        body,
                        12.0,
                        scheme.on_surface_variant,
                        amane::Weight::Regular,
                    ))])
                    .width(Parent)
                    .justify(Center),
                ),
            ])
            .gap(10.0)
            .width(Parent)
            .align(Center),
        )
}

fn format_time(seconds: u64) -> String {
    format!("{}:{:02}", seconds / 60, seconds % 60)
}
