use amane::{
    Audio, Battery, Brightness, Center, Column, Cpu, End, Image, Media, Memory,
    Notifications, Parent, Pointer, Rectangle, Row, ScrollArea, Service, SpaceBetween, Stack,
    Start,
    Widget,
};

use crate::art::{artwork, media_labels};
use crate::clock::Clock;
use crate::m3::{self, scale, scale_label, Type};
use crate::overlay::{Overlay, Panel};
use crate::settings::Settings;
use crate::theme::{Scheme, fade};
use crate::ui::{Tab, Ui};
use crate::weather::{Hour, Weather};

use super::widgets::{
    BELL, BRIGHTNESS, CLOSE, DROP, MUTED, NEXT, PAUSE, PLAY, PREVIOUS, VOLUME, WALLPAPER,
    AIR, boxed, card, chip, icon, icon_button, label, progress, round_button, slider, text,
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
        (Tab::Home, "Home"),
        (Tab::Media, "Media"),
        (Tab::Weather, "Clima"),
        (Tab::Wallpapers, "Papéis"),
        (Tab::Alerts, "Alertas"),
    ];

    let mut header: Vec<Box<dyn Widget>> = tabs
        .iter()
        .map(|&(target, name)| {
            let key = format!("tab:{name}");

            boxed(chip(&key, name, 100.0, tab == target, scheme, move || {
                Ui::select(target)
            }))
        })
        .collect();

    // the config button sits at the left of the chips, the filler at the right
    let spacer = Rectangle::new().width(Parent).height(1.0);

    let body = match tab {
        Tab::Home => overview(scheme),
        Tab::Media => media(scheme),
        Tab::Weather => weather_tab(scheme),
        Tab::Wallpapers => wallpapers(scheme),
        Tab::Alerts => alerts(scheme),
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
                boxed(
                    Row::new({
                        let mut header = header;

                        header.insert(
                            0,
                            boxed(m3::icon_button(
                                "dash:config",
                                m3::TUNE,
                                34.0,
                                scheme,
                                || {
                                    Overlay::open(Panel::Config);
                                },
                            )),
                        );

                        header.push(boxed(spacer));

                        header
                    })
                    .gap(TAB_GAP)
                    .align(Center)
                    .width(Parent),
                ),
                body,
            ])
            .width(Parent)
            .gap(16.0),
        )
}

fn overview(scheme: &Scheme) -> Box<dyn Widget> {
    Box::new(
        Column::new(vec![
            Box::new(
                Row::new(vec![
                    Box::new(hero_card(scheme)),
                    Box::new(weather_hero(scheme)),
                    Box::new(media_mini(scheme)),
                ])
                .gap(12.0)
                .width(Parent)
                .height(170.0),
            ),
            Box::new(
                Row::new(vec![
                    Box::new(controls_card(scheme)),
                    Box::new(meters_card(scheme)),
                ])
                .gap(12.0)
                .width(Parent)
                .height(126.0),
            ),
        ])
        .gap(12.0)
        .width(Parent)
        .height(Parent),
    )
}

/*
 * the clock of the home tab: the greeting above, the big display below,
 * all on the primary container
 */
fn hero_card(scheme: &Scheme) -> Rectangle {
    let clock = Clock::read();
    let color = scheme.on_primary_container;

    card(216.0, scheme.primary_container).child(
        Column::new(vec![
            Box::new(scale(Type::LabelLarge, clock.greeting(), color)),
            Box::new(scale(Type::DisplayMedium, clock.time(true), color)),
            Box::new(scale_label(
                Type::BodySmall,
                clock.date(),
                color,
                1,
            )),
        ])
        .gap(4.0)
        .width(Parent)
        .height(Parent)
        .align(Start)
        .justify(Center),
    )
}

/*
 * a compact weather read: the cookie badge on the left, the numbers on
 * the right — the whole thing on the secondary container
 */
fn weather_hero(scheme: &Scheme) -> Rectangle {
    let (condition, temp, humidity, wind, glyph) = {
        let weather = Weather::read();

        (
            String::from(weather.condition()),
            String::from(weather.temperature()),
            String::from(weather.humidity()),
            String::from(weather.wind()),
            weather.glyph(),
        )
    };

    let color = scheme.on_secondary_container;

    let name = if condition.is_empty() {
        String::from("Sem dados")
    } else {
        condition
    };

    let empty = String::from("—");
    let temp = if temp.is_empty() { empty.clone() } else { temp };

    let badge = Stack::new(vec![
        Box::new(m3::cookie(7.0, 62.0, scheme.primary_container)) as Box<dyn amane::Widget>,
        Box::new(icon(glyph, 28.0, scheme.on_primary_container)) as Box<dyn amane::Widget>,
    ])
    .height(62.0)
    .width(62.0);

    let weather_rows = Column::new(vec![
        Box::new(scale_label(Type::TitleSmall, name, color, 2)),
        Box::new(scale(Type::DisplaySmall, temp, color)),
        Box::new(
            Row::new(vec![
                Box::new(
                    Row::new(vec![
                        Box::new(icon(DROP, 13.0, color)),
                        Box::new(scale(Type::LabelSmall, humidity, color)),
                    ])
                    .gap(4.0)
                    .align(Center),
                ),
                Box::new(
                    Row::new(vec![
                        Box::new(icon(AIR, 13.0, color)),
                        Box::new(scale(Type::LabelSmall, wind, color)),
                    ])
                    .gap(4.0)
                    .align(Center),
                ),
            ])
            .gap(12.0)
            .align(Center),
        ),
    ])
    .gap(2.0)
    .width(126.0)
    .align(Start);

    card(234.0, scheme.secondary_container).child(
        Row::new(vec![
            Box::new(badge),
            Box::new(weather_rows),
        ])
        .gap(14.0)
        .width(Parent)
        .height(Parent)
        .align(Center),
    )
}

// the media mini on the home row: cover or cookie, names, and a play/pause
fn media_mini(scheme: &Scheme) -> Rectangle {
    let media = Media::read();

    let (title, artist) = media_labels(&media);

    let glyph = if media.playing() { PAUSE } else { PLAY };

    let artwork = crate::art::artwork(media.art_url(), 44.0, 12.0, scheme);

    drop(media);

    let controls = m3::round_button(
        "home-play",
        glyph,
        34.0,
        scheme.primary,
        scheme.on_primary,
        Media::play_pause,
    );

    card(150.0, scheme.surface_container_high).child(Column::new(vec![
        Box::new(Row::new(vec![Box::new(artwork)]).width(Parent).justify(Center)),
        Box::new(scale_label(Type::TitleSmall, &title, scheme.on_surface, 1)),
        Box::new(scale_label(
            Type::LabelSmall,
            &artist,
            scheme.on_surface_variant,
            1,
        )),
        Box::new(Row::new(vec![Box::new(controls)]).width(Parent).justify(Center)),
    ])
    .gap(2.0)
    .width(Parent)
    .height(Parent)
    .align(Center)
    .justify(Center))
}

/*
 * the machine utilities: volume and brightness as real m3 sliders,
 * each with its glyph inside
 */
fn controls_card(scheme: &Scheme) -> Rectangle {
    let audio = Audio::read();
    let brightness = Brightness::read();

    let volume_glyph = if audio.muted() || audio.volume() == 0 {
        MUTED
    } else {
        VOLUME
    };

    let volume = f32::from(audio.volume()) / 100.0;

    drop(audio);

    let body = if brightness.present() {
        let brightness_percent = brightness.percent();

        let brightness_slider = m3::slider(
            "home:brightness",
            BRIGHTNESS,
            f32::from(brightness_percent) / 100.0,
            264.0,
            scheme,
            move |fraction| Brightness::set((fraction * 100.0).round() as u8),
        );

        drop(brightness);

        Column::new(vec![
            Box::new(
                m3::slider(
                    "home:volume-r",
                    volume_glyph,
                    volume,
                    264.0,
                    scheme,
                    move |fraction| Audio::set_volume((fraction * 100.0).round() as u8),
                ),
            ),
            Box::new(brightness_slider),
        ])
        .gap(10.0)
        .width(Parent)
        .height(Parent)
        .align(Center)
        .justify(Center)
    } else {
        drop(brightness);

        Column::new(vec![Box::new(
            m3::slider(
                "home:volume-r",
                volume_glyph,
                volume,
                264.0,
                scheme,
                move |fraction| Audio::set_volume((fraction * 100.0).round() as u8),
            ),
        )])
        .width(Parent)
        .height(Parent)
        .align(Center)
        .justify(Center)
    };

    card(380.0, scheme.surface_container_high).child(body)
}

/*
 * the machine numbers as expressive wavy progress meters
 */
fn meters_card(scheme: &Scheme) -> Rectangle {
    card(296.0, scheme.surface_container_high).child(
        Column::new(vec![
            boxed(meter_row("CPU", Cpu::read().percent(), scheme)),
            boxed(meter_row("RAM", Memory::read().percent(), scheme)),
            if Battery::read().present() {
                boxed(meter_row("BAT", Battery::read().percent(), scheme))
            } else {
                boxed(scale_label(Type::LabelSmall, "sem bateria", scheme.on_surface_variant, 1))
            },
        ])
        .gap(12.0)
        .width(Parent)
        .height(Parent)
        .align(Center)
        .justify(Center),
    )
}

fn meter_row(name: &str, percent: u8, scheme: &Scheme) -> Column {
    Column::new(vec![
        Box::new(
            Row::new(vec![
                Box::new(scale_label(
                    Type::LabelSmall,
                    name,
                    scheme.on_surface_variant,
                    1,
                )),
                Box::new(scale_label(Type::LabelSmall, format!("{percent}%"), scheme.on_surface, 1)),
            ])
            .width(Parent)
            .justify(amane::SpaceBetween)
            .align(Center),
        ),
        Box::new(
            m3::wavy(
                f32::from(percent.min(100)) / 100.0,
                264.0,
                false,
                scheme.tertiary,
                scheme.surface_container_highest,
            ),
        ),
    ])
    .gap(4.0)
    .width(Parent)
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

/*
 * the weather tab: a big current card plus the hourly strip that the
 * weather service refreshes in the background
 */
fn weather_tab(scheme: &Scheme) -> Box<dyn Widget> {
    let (condition, temp, humidity, wind, glyph) = {
        let weather = Weather::read();

        (
            String::from(weather.condition()),
            String::from(weather.temperature()),
            String::from(weather.humidity()),
            String::from(weather.wind()),
            weather.glyph(),
        )
    };

    let color = scheme.on_secondary_container;

    let condition = if condition.is_empty() {
        String::from("Sem dados")
    } else {
        condition
    };

    let temp = if temp.is_empty() {
        String::from("—")
    } else {
        temp
    };

    let humidity = if humidity.is_empty() {
        String::from("—")
    } else {
        humidity
    };

    let wind = if wind.is_empty() {
        String::from("—")
    } else {
        wind
    };

    // the cookie badge sits beside the reading, like the m3 expressive rows
    let badge = Rectangle::new()
        .width(76.0)
        .height(76.0)
        .align_child(Center, Center)
        .child(
            Stack::new(vec![
                Box::new(super::widgets::cookie(7.0, 76.0, scheme.primary_container)) as Box<dyn Widget>,
                Box::new(icon(glyph, 34.0, scheme.on_primary_container)) as Box<dyn Widget>,
            ]),
        );

    let detail = |glyph: &'static str, value: &str| {
        Row::new(vec![
            Box::new(icon(glyph, 14.0, color)) as Box<dyn Widget>,
            Box::new(text(
                value,
                13.0,
                color,
                amane::Weight::Medium,
            )) as Box<dyn Widget>,
        ])
        .gap(5.0)
        .align(Center)
    };

    let reading = Column::new(vec![
        Box::new(scale(Type::LabelMedium, &condition, fade(color, 200))),
        Box::new(scale(Type::DisplaySmall, &temp, color)),
        Box::new(
            Row::new(vec![
                Box::new(detail(DROP, &humidity)),
                Box::new(detail(AIR, &wind)),
            ])
            .width(Parent)
            .gap(14.0)
            .align(Center),
        ),
    ])
    .gap(4.0)
    .width(Parent);

    let current = card(660.0, scheme.secondary_container).child(
        Row::new(vec![
            Box::new(badge),
            Box::new(reading),
        ])
        .gap(24.0)
        .width(Parent)
        .height(Parent)
        .align(Center),
    );

    // the hourly strip, one cookie-ish chip per hour from the forecast
    let hours: Vec<Box<dyn Widget>> = Weather::read()
        .hourly()
        .iter()
        .map(|hour| boxed(hour_chip(hour, scheme)))
        .collect();

    let strip = card(660.0, scheme.surface_container_high).child(if hours.is_empty() {
        Stack::new(vec![Box::new(
            Rectangle::new()
                .width(Parent)
                .height(Parent)
                .align_child(Center, Center)
                .child(text(
                    "Previsão por hora chega junto do próximo ciclo",
                    12.0,
                    scheme.on_surface_variant,
                    amane::Weight::Regular,
                )),
        ) as Box<dyn Widget>])
    } else {
        Stack::new(vec![
            Box::new(Row::new(hours).gap(8.0).width(Parent).align(Center)) as Box<dyn Widget>,
        ])
    });

    boxed(
        Column::new(vec![Box::new(current), Box::new(strip)])
            .gap(12.0)
            .width(660.0),
    )
}

// one hour of the strip: the hour, a small condition icon and the temp
fn hour_chip(hour: &Hour, scheme: &Scheme) -> Rectangle {
    let color = scheme.on_surface;

    Rectangle::new()
        .width(76.0)
        .height(104.0)
        .fill(scheme.surface_container_highest)
        .radius(20.0)
        .clip()
        .child(
            Column::new(vec![
                Box::new(
                    Row::new(vec![Box::new(text(
                        &hour.label,
                        11.0,
                        scheme.on_surface_variant,
                        amane::Weight::Medium,
                    ))])
                    .width(Parent)
                    .justify(Center),
                ),
                Box::new(icon(hour.glyph, 20.0, color)) as Box<dyn Widget>,
                Box::new(
                    Row::new(vec![Box::new(text(
                        &hour.temperature,
                        12.5,
                        color,
                        amane::Weight::SemiBold,
                    ))])
                    .width(Parent)
                    .justify(Center),
                ),
            ])
            .gap(4.0)
            .width(Parent)
            .height(Parent)
            .justify(Center),
        )
}

/*
 * the visual settings tab: dark or light mode and the matugen scheme,
 * regenerating every color the user's config knows about
 */
// the centered icon and two lines, for lists that have nothing to show yet
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
