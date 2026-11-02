use amane::{
    Audio, Battery, Brightness, Center, Column, Cpu, End, Image, Media, Memory,
    Notifications, Parent, Pointer, Rectangle, Row, ScrollArea, Service, SpaceBetween, Stack,
    Widget,
};

use crate::art::{artwork, media_labels};
use crate::clock::Clock;
use crate::settings::Settings;
use crate::theme::Scheme;
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
        (Tab::Overview, "Overview"),
        (Tab::Media, "Media"),
        (Tab::Wallpapers, "Wallpapers"),
        (Tab::Alerts, "Alertas"),
        (Tab::Weather, "Clima"),
        (Tab::Settings, "Ajustes"),
    ];

    let header: Vec<Box<dyn Widget>> = tabs
        .iter()
        .map(|&(target, name)| {
            let key = format!("tab:{name}");

            boxed(chip(&key, name, 100.0, tab == target, scheme, move || {
                Ui::select(target)
            }))
        })
        .collect();

    let body = match tab {
        Tab::Overview => overview(scheme),
        Tab::Media => media(scheme),
        Tab::Wallpapers => wallpapers(scheme),
        Tab::Alerts => alerts(scheme),
        Tab::Weather => weather_tab(scheme),
        Tab::Settings => settings_tab(scheme),
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
        Column::new(vec![
            boxed(
                Row::new(vec![
                    boxed(time_card(scheme)),
                    boxed(weather_card(scheme)),
                    boxed(media_mini(scheme)),
                ])
                .gap(12.0),
            ),
            boxed(controls_card(scheme)),
        ])
        .gap(12.0),
    )
}

// the weather as wttr.in answered it last, with the condition translated
fn media_mini(scheme: &Scheme) -> Rectangle {
    let media = Media::read();

    let (title, artist) = media_labels(&media);

    let glyph = if media.playing() { PAUSE } else { PLAY };

    let artwork = crate::art::artwork(media.art_url(), 96.0, 14.0, scheme);

    card(196.0, scheme.surface_container_high).child(
        Row::new(vec![
            boxed(artwork),
            boxed(
                Column::new(vec![
                    boxed(label(
                        title,
                        13.0,
                        scheme.on_surface,
                        amane::Weight::SemiBold,
                        1,
                    )),
                    boxed(label(
                        artist,
                        11.0,
                        scheme.on_surface_variant,
                        amane::Weight::Regular,
                        1,
                    )),
                    boxed(round_button(
                        "overview-play",
                        glyph,
                        36.0,
                        scheme.primary,
                        scheme.on_primary,
                        Media::play_pause,
                    )),
                ])
                .gap(5.0)
                .width(Parent),
            ),
        ])
        .gap(10.0)
        .width(Parent)
        .height(Parent)
        .align(Center),
    )
}

// the weather as open-meteo answered it last, with the condition translated
fn weather_card(scheme: &Scheme) -> Rectangle {
    let weather = Weather::read();

    let glyph = weather.glyph();
    let name = weather.condition().to_string();

    let temperature = String::from(weather.temperature());
    let humidity = String::from(weather.humidity());
    let wind = String::from(weather.wind());

    drop(weather);

    let color = scheme.on_secondary_container;

    let temperature = if temperature.is_empty() {
        String::from("—")
    } else {
        temperature
    };

    let name = if name.is_empty() {
        String::from("Sem dados")
    } else {
        name
    };

    let detail = |glyph: &'static str, value: &str| {
        Row::new(vec![
            boxed(icon(glyph, 12.0, color)),
            boxed(text(value, 11.0, color, amane::Weight::Medium)),
        ])
        .gap(4.0)
        .align(Center)
    };

    card(196.0, scheme.secondary_container).child(
        Column::new(vec![
            boxed(
                Row::new(vec![boxed(icon(glyph, 34.0, color))])
                    .width(Parent)
                    .justify(Center),
            ),
            boxed(
                Row::new(vec![boxed(text(
                    temperature,
                    30.0,
                    color,
                    amane::Weight::SemiBold,
                ))])
                .width(Parent)
                .justify(Center),
            ),
            boxed(
                Row::new(vec![boxed(text(
                    name,
                    12.5,
                    color,
                    amane::Weight::Medium,
                ))])
                .width(Parent)
                .justify(Center),
            ),
            boxed(
                Row::new(vec![
                    boxed(detail(DROP, &humidity)),
                    boxed(detail(AIR, &wind)),
                ])
                .width(Parent)
                .gap(10.0)
                .justify(Center),
            ),
        ])
        .gap(5.0)
        .width(Parent)
        .height(Parent)
        .justify(Center),
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

fn controls_card(scheme: &Scheme) -> Rectangle {
    let audio = Audio::read();
    let brightness = Brightness::read();
    let battery = Battery::read();

    let volume_glyph = if audio.muted() { MUTED } else { VOLUME };

    let mut sliders: Vec<Box<dyn Widget>> = vec![boxed(
        Row::new(vec![
            boxed(icon_button(
                "mute",
                volume_glyph,
                24.0,
                scheme,
                Audio::toggle_mute,
            )),
            boxed(slider(
                "volume",
                f32::from(audio.volume()) / 100.0,
                118.0,
                scheme,
                |value| Audio::set_volume((value * 100.0).round() as u8),
            )),
        ])
        .gap(4.0)
        .align(Center),
    )];

    if brightness.present() {
        sliders.push(boxed(
            Row::new(vec![
                boxed(
                    Rectangle::new()
                        .width(24.0)
                        .height(24.0)
                        .align_child(Center, Center)
                        .child(icon(BRIGHTNESS, 16.0, scheme.on_surface_variant)),
                ),
                boxed(slider(
                    "brightness",
                    f32::from(brightness.percent()) / 100.0,
                    118.0,
                    scheme,
                    |value| Brightness::set((value * 100.0).round() as u8),
                )),
            ])
            .gap(4.0)
            .align(Center),
        ));
    }

    /*
     * the hardware numbers spread along the card instead of stacking
     * below the sliders, so the overview keeps to its rows
     */
    let mut stats: Vec<Box<dyn Widget>> = vec![
        stat("CPU", Cpu::read().percent(), 140.0, scheme),
        stat("RAM", Memory::read().percent(), 140.0, scheme),
    ];

    if battery.present() {
        stats.push(stat("BAT", battery.percent(), 140.0, scheme));
    }

    card(660.0, scheme.surface_container_high).child(
        Row::new(vec![
            boxed(
                Column::new(sliders)
                    .gap(10.0)
                    .align(Center),
            ),
            boxed(Row::new(stats).gap(14.0).align(Center)),
        ])
        .gap(14.0)
        .width(Parent)
        .height(Parent)
        .justify(SpaceBetween)
        .align(Center),
    )
}

fn stat(name: &str, percent: u8, width: f32, scheme: &Scheme) -> Box<dyn Widget> {
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
            boxed(meter(name, percent, width, scheme)),
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

    // the cookie shape behind the condition icon, the M3 expressive badge
    let badge = Rectangle::new()
        .width(92.0)
        .height(92.0)
        .align_child(Center, Center)
        .child(
            Stack::new(vec![
                Box::new(super::widgets::cookie(7.0, 92.0, scheme.primary_container)) as Box<dyn Widget>,
                Box::new(icon(glyph, 44.0, scheme.on_primary_container)) as Box<dyn Widget>,
            ]),
        );

    let detail = |glyph: &'static str, value: &str| {
        Row::new(vec![
            Box::new(icon(glyph, 16.0, color)) as Box<dyn Widget>,
            Box::new(text(value, 14.0, color, amane::Weight::Medium)) as Box<dyn Widget>,
        ])
        .gap(5.0)
        .align(Center)
    };

    let current = card(660.0, scheme.secondary_container).child(
        Column::new(vec![
            Box::new(
                Row::new(vec![Box::new(badge)])
                    .width(Parent)
                    .justify(Center),
            ),
            Box::new(
                Row::new(vec![Box::new(text(
                    &temp,
                    44.0,
                    color,
                    amane::Weight::SemiBold,
                ))])
                .width(Parent)
                .justify(Center),
            ),
            Box::new(
                Row::new(vec![Box::new(text(
                    &condition,
                    18.0,
                    color,
                    amane::Weight::Medium,
                ))])
                .width(Parent)
                .justify(Center),
            ),
            Box::new(
                Row::new(vec![
                    Box::new(detail(DROP, &humidity)),
                    Box::new(detail(AIR, &wind)),
                ])
                .width(Parent)
                .gap(20.0)
                .justify(Center),
            ),
        ])
        .gap(10.0)
        .width(Parent)
        .height(Parent)
        .justify(Center),
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
fn settings_tab(scheme: &Scheme) -> Box<dyn Widget> {
    let settings = Settings::read();

    let dark = settings.dark();

    let current = settings.scheme_name().to_string();

    drop(settings);

    const SCHEMES: [(&str, &str); 8] = [
        ("scheme-tonal-spot", "Tonal"),
        ("scheme-content", "Content"),
        ("scheme-expressive", "Expressivo"),
        ("scheme-neutral", "Neutro"),
        ("scheme-vibrant", "Vibrante"),
        ("scheme-rainbow", "Rainbow"),
        ("scheme-fruit-salad", "Frutas"),
        ("scheme-fidelity", "Fidelidade"),
    ];

    let mode_row = Row::new(vec![
        boxed(chip(
            "mode-dark",
            "Escuro",
            96.0,
            dark,
            scheme,
            || Settings::set_dark(true),
        )),
        boxed(chip(
            "mode-light",
            "Claro",
            96.0,
            !dark,
            scheme,
            || Settings::set_dark(false),
        )),
        boxed(label(
            "Troca o matugen de tudo: shell, alacritty, niri, spicetify…",
            11.0,
            scheme.on_surface_variant,
            amane::Weight::Regular,
            2,
        )),
    ])
    .gap(8.0)
    .align(Center)
    .width(Parent);

    let rows: Vec<Box<dyn Widget>> = SCHEMES
        .chunks(4)
        .map(|chunk| {
            let chips: Vec<Box<dyn Widget>> = chunk
                .iter()
                .map(|&(value, name)| {
                    boxed(chip(
                        &format!("scheme:{value}"),
                        name,
                        142.0,
                        current == value,
                        scheme,
                        move || {
                            Settings::set_scheme(value);
                        },
                    ))
                })
                .collect();

            boxed(Row::new(chips).gap(6.0))
        })
        .collect();

    let scheme_card = card(660.0, scheme.surface_container_high).child(
        Column::new(vec![
            boxed(label(
                "Esquema de cores",
                14.0,
                scheme.on_surface,
                amane::Weight::Medium,
                1,
            )),
            boxed(
                Column::new(rows)
                    .gap(6.0)
                    .width(Parent),
            ),
            boxed(text(
                &format!("Atual: {current}"),
                11.5,
                scheme.on_surface_variant,
                amane::Weight::Medium,
            )),
        ])
        .gap(12.0)
        .width(Parent)
        .height(Parent),
    );

    let mode_card = card(660.0, scheme.surface_container_high).child(mode_row);

    boxed(
        ScrollArea::new(
            "settings-tab",
            Column::new(vec![boxed(mode_card), boxed(scheme_card)])
                .gap(12.0)
                .width(660.0),
        )
        .width(660.0)
        .height(BODY),
    )
}

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
