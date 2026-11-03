/*
 * the dash: a material 3 surface with a navigation rail on the left and
 * tonal cards on the right. every card has an exact size and the sizes
 * add up to the content area, so nothing overflows or drifts.
 */
use amane::{
    Audio, Battery, Brightness, Button, Center, Column, Cpu, Image, Media, Memory, Notifications, Parent,
    Pointer, Rectangle, Row, ScrollArea, Service, SpaceBetween, Start, Widget,
};

use crate::art::artwork;
use crate::clock::Clock;
use crate::glyphs;
use crate::m3::{
    self, Item, Kind, Type, badge_shape, boxed, button, icon, icon_button, leading_icon, list_item,
    rail_item, round_button, scale, scale_label, slider, wavy,
};
use crate::overlay::{Overlay, Panel};
use crate::settings::Settings;
use crate::theme::{Scheme, fade};
use crate::ui::{Tab, Ui};
use crate::weather::{Hour, Weather};

pub const WIDTH: f32 = 740.0;
pub const HEIGHT: f32 = 420.0;
pub const RADIUS: f32 = 32.0;

const PADDING: f32 = 16.0;
const RAIL: f32 = 72.0;
const GAP: f32 = 12.0;

// the area the tabs draw in
const CONTENT_WIDTH: f32 = WIDTH - PADDING * 2.0 - RAIL - PADDING;
const CONTENT_HEIGHT: f32 = HEIGHT - PADDING * 2.0;

const CARD_RADIUS: f32 = 24.0;

pub fn view(scheme: &Scheme, tab: Tab, alpha: f32) -> Rectangle {
    let body = match tab {
        Tab::Home => home(scheme),
        Tab::Media => media_tab(scheme),
        Tab::Weather => weather_tab(scheme),
        Tab::Wallpapers => wallpapers(scheme),
        Tab::Alerts => alerts(scheme),
    };

    // the body fades whenever the tab changes
    let alpha = alpha * crate::motion::fade("tab", tab.index());

    let content = Rectangle::new()
        .width(CONTENT_WIDTH)
        .height(CONTENT_HEIGHT)
        .opacity(alpha)
        .child(amane::Stack::new(vec![body]).width(Parent).height(Parent));

    Rectangle::new()
        .width(Parent)
        .height(Parent)
        .padding(PADDING)
        .child(
            Row::new(vec![boxed(rail(scheme, tab)), boxed(content)])
                .gap(PADDING)
                .width(Parent)
                .height(Parent),
        )
}

fn rail(scheme: &Scheme, tab: Tab) -> Column {
    let items = [
        (Tab::Home, m3::HOME, "Home", "home"),
        (Tab::Media, m3::MUSIC, "Mídia", "media"),
        (Tab::Weather, m3::CLOUDY, "Clima", "weather"),
        (Tab::Wallpapers, m3::WALLPAPER, "Papéis", "wallpapers"),
        (Tab::Alerts, m3::BELL, "Alertas", "alerts"),
    ];

    let mut children: Vec<Box<dyn Widget>> = vec![boxed(
        Row::new(vec![boxed(round_button(
            "dash:config",
            m3::TUNE,
            48.0,
            scheme.primary_container,
            scheme.on_primary_container,
            crate::settingsview::toggle,
        ))])
        .width(Parent)
        .justify(Center),
    )];

    for (target, glyph, name, key) in items {
        children.push(boxed(rail_item(
            &format!("dash:rail:{key}"),
            glyph,
            name,
            tab == target,
            scheme,
            move || Ui::select(target),
        )));
    }

    Column::new(children)
        .gap(6.0)
        .width(RAIL)
        .height(Parent)
        .align(Center)
}

fn card(width: f32, height: f32, fill: amane::Color) -> Rectangle {
    Rectangle::new()
        .width(width)
        .height(height)
        .fill(fill)
        .radius(CARD_RADIUS)
        .clip()
        .padding(PADDING)
}

// ---- home -----------------------------------------------------------------

const ROW_A: f32 = 172.0;
const ROW_B: f32 = CONTENT_HEIGHT - ROW_A - GAP;

const HERO: f32 = 196.0;
const WEATHER: f32 = 212.0;
const PLAYER: f32 = CONTENT_WIDTH - HERO - WEATHER - GAP * 2.0;

const CONTROLS: f32 = 308.0;
const METERS: f32 = CONTENT_WIDTH - CONTROLS - GAP;

fn home(scheme: &Scheme) -> Box<dyn Widget> {
    boxed(
        Column::new(vec![
            boxed(
                Row::new(vec![
                    boxed(hero(scheme)),
                    boxed(weather_card(scheme)),
                    boxed(player(scheme)),
                ])
                .gap(GAP)
                .width(Parent)
                .height(ROW_A),
            ),
            boxed(
                Row::new(vec![boxed(controls(scheme)), boxed(meters(scheme))])
                    .gap(GAP)
                    .width(Parent)
                    .height(ROW_B),
            ),
        ])
        .gap(GAP)
        .width(Parent)
        .height(Parent),
    )
}

fn hero(scheme: &Scheme) -> Rectangle {
    let clock = Clock::read();
    let color = scheme.on_primary_container;

    card(HERO, ROW_A, scheme.primary_container).child(
        Column::new(vec![
            boxed(scale(Type::LabelLarge, clock.greeting(), fade(color, 220))),
            boxed(scale(Type::DisplayMedium, clock.time(true), color)),
            boxed(scale_label(Type::BodySmall, clock.date(), color, 1)),
        ])
        .gap(2.0)
        .width(Parent)
        .height(Parent)
        .align(Start)
        .justify(Center),
    )
}

fn weather_card(scheme: &Scheme) -> Rectangle {
    let (condition, temp, humidity, wind, glyph) = weather_reading();

    let color = scheme.on_tertiary_container;

    let head = Row::new(vec![
        boxed(badge_shape(
            glyph,
            48.0,
            7.0,
            scheme.tertiary,
            scheme.on_tertiary,
        )),
        boxed(scale_label(Type::TitleSmall, condition, color, 2)),
    ])
    .gap(10.0)
    .align(Center)
    .width(Parent);

    card(WEATHER, ROW_A, scheme.tertiary_container)
        .cursor(Pointer)
        .on_click(|button| {
            if button == Button::Left {
                Ui::select(Tab::Weather);
            }
        })
        .child(
        Column::new(vec![
            boxed(head),
            boxed(scale(Type::DisplaySmall, temp, color)),
            boxed(detail_row(&humidity, &wind, color)),
        ])
        .gap(6.0)
        .width(Parent)
        .height(Parent)
        .justify(Center),
    )
}

fn detail_row(humidity: &str, wind: &str, color: amane::Color) -> Row {
    Row::new(vec![
        boxed(
            Row::new(vec![
                boxed(icon(m3::DROP, 14.0, color)),
                boxed(scale(Type::LabelMedium, humidity, color)),
            ])
            .gap(4.0)
            .align(Center),
        ),
        boxed(
            Row::new(vec![
                boxed(icon(m3::AIR, 14.0, color)),
                boxed(scale(Type::LabelMedium, wind, color)),
            ])
            .gap(4.0)
            .align(Center),
        ),
    ])
    .gap(14.0)
    .align(Center)
}

fn weather_reading() -> (String, String, String, String, &'static str) {
    let weather = Weather::read();

    let fallback = |value: &str, empty: &str| {
        if value.is_empty() {
            String::from(empty)
        } else {
            String::from(value)
        }
    };

    (
        fallback(weather.condition(), "Sem dados"),
        fallback(weather.temperature(), "—"),
        fallback(weather.humidity(), "—"),
        fallback(weather.wind(), "—"),
        weather.glyph(),
    )
}

fn labels(media: &Media) -> (String, String) {
    if media.active().is_none() {
        return (String::from("Nada tocando"), String::from("Sem player"));
    }

    crate::art::media_labels(media)
}

fn player(scheme: &Scheme) -> Rectangle {
    let media = Media::read();

    let (title, artist) = labels(&media);
    let playing = media.playing();
    let art = artwork(media.art_url(), 52.0, 16.0, scheme);

    drop(media);

    let controls = Row::new(vec![
        boxed(icon_button(
            "home:previous",
            m3::PREVIOUS,
            34.0,
            scheme,
            Media::previous,
        )),
        boxed(round_button(
            "home:play",
            if playing { m3::PAUSE } else { m3::PLAY },
            40.0,
            scheme.primary,
            scheme.on_primary,
            Media::play_pause,
        )),
        boxed(icon_button(
            "home:next",
            m3::NEXT,
            34.0,
            scheme,
            Media::next,
        )),
    ])
    .gap(6.0)
    .align(Center);

    card(PLAYER, ROW_A, scheme.surface_container_high).child(
        Column::new(vec![
            boxed(art),
            boxed(
                Column::new(vec![
                    boxed(scale_label(Type::TitleSmall, title, scheme.on_surface, 1)),
                    boxed(scale_label(
                        Type::LabelSmall,
                        artist,
                        scheme.on_surface_variant,
                        1,
                    )),
                ])
                .width(Parent)
                .align(Center),
            ),
            boxed(controls),
        ])
        .gap(4.0)
        .width(Parent)
        .height(Parent)
        .align(Center)
        .justify(Center),
    )
}

fn controls(scheme: &Scheme) -> Rectangle {
    let inner = CONTROLS - PADDING * 2.0;

    let (volume, muted, mic, mic_muted) = {
        let audio = Audio::read();

        (
            audio.volume(),
            audio.muted(),
            audio.microphone_volume(),
            audio.microphone_muted(),
        )
    };

    let mut rows: Vec<Box<dyn Widget>> = vec![boxed(slider(
        "home:volume",
        glyphs::volume(volume, muted),
        f32::from(volume) / 100.0,
        inner,
        scheme,
        |fraction| Audio::set_volume((fraction * 100.0).round() as u8),
    ))];

    let brightness = Brightness::read();

    if brightness.present() {
        let percent = brightness.percent();

        rows.push(boxed(slider(
            "home:brightness",
            glyphs::brightness(percent),
            f32::from(percent) / 100.0,
            inner,
            scheme,
            |fraction| Brightness::set((fraction * 100.0).round() as u8),
        )));
    }

    drop(brightness);

    rows.push(boxed(slider(
        "home:mic",
        glyphs::microphone(mic_muted),
        if mic_muted {
            0.0
        } else {
            f32::from(mic) / 100.0
        },
        inner,
        scheme,
        |fraction| Audio::set_microphone_volume((fraction * 100.0).round() as u8),
    )));

    card(CONTROLS, ROW_B, scheme.surface_container_high).child(
        Column::new(rows)
            .gap(8.0)
            .width(Parent)
            .height(Parent)
            .justify(Center),
    )
}

fn meters(scheme: &Scheme) -> Rectangle {
    let inner = METERS - PADDING * 2.0;

    let mut rows: Vec<Box<dyn Widget>> = vec![
        boxed(meter("CPU", Cpu::read().percent(), inner, scheme)),
        boxed(meter("RAM", Memory::read().percent(), inner, scheme)),
    ];

    let battery = Battery::read();

    if battery.present() {
        rows.push(boxed(meter("Bateria", battery.percent(), inner, scheme)));
    }

    card(METERS, ROW_B, scheme.surface_container_high).child(
        Column::new(rows)
            .gap(10.0)
            .width(Parent)
            .height(Parent)
            .justify(Center),
    )
}

fn meter(name: &str, percent: u8, width: f32, scheme: &Scheme) -> Column {
    Column::new(vec![
        boxed(
            Row::new(vec![
                boxed(scale(Type::LabelMedium, name, scheme.on_surface_variant)),
                boxed(scale(
                    Type::LabelMedium,
                    format!("{percent}%"),
                    scheme.on_surface,
                )),
            ])
            .width(Parent)
            .justify(SpaceBetween)
            .align(Center),
        ),
        boxed(wavy(
            f32::from(percent.min(100)) / 100.0,
            width,
            false,
            scheme.tertiary,
            scheme.surface_container_highest,
        )),
    ])
    .width(Parent)
}

// ---- media ----------------------------------------------------------------

fn media_tab(scheme: &Scheme) -> Box<dyn Widget> {
    let media = Media::read();

    let length = media.length().as_secs();
    let position = media.position().as_secs();

    let fraction = if length == 0 {
        0.0
    } else {
        (position as f32 / length as f32).clamp(0.0, 1.0)
    };

    let (title, artist) = labels(&media);
    let playing = media.playing();
    let art = artwork(media.art_url(), 240.0, 28.0, scheme);

    drop(media);

    let bar = CONTENT_WIDTH - PADDING * 2.0 - 240.0 - 28.0;

    let times = Row::new(vec![
        boxed(scale(
            Type::LabelMedium,
            format_time(position),
            scheme.on_surface_variant,
        )),
        boxed(scale(
            Type::LabelMedium,
            format_time(length),
            scheme.on_surface_variant,
        )),
    ])
    .justify(SpaceBetween)
    .width(Parent);

    let controls = Row::new(vec![
        boxed(round_button(
            "media:previous",
            m3::PREVIOUS,
            52.0,
            scheme.secondary_container,
            scheme.on_secondary_container,
            Media::previous,
        )),
        boxed(round_button(
            "media:play",
            if playing { m3::PAUSE } else { m3::PLAY },
            68.0,
            scheme.primary,
            scheme.on_primary,
            Media::play_pause,
        )),
        boxed(round_button(
            "media:next",
            m3::NEXT,
            52.0,
            scheme.secondary_container,
            scheme.on_secondary_container,
            Media::next,
        )),
    ])
    .gap(14.0)
    .align(Center);

    let info = Column::new(vec![
        boxed(scale_label(
            Type::HeadlineSmall,
            title,
            scheme.on_surface,
            1,
        )),
        boxed(scale_label(
            Type::BodyLarge,
            artist,
            scheme.on_surface_variant,
            1,
        )),
        boxed(super::widgets::progress("dash:seek", fraction, bar, scheme)),
        boxed(times),
        boxed(controls),
    ])
    .gap(12.0)
    .width(bar)
    .height(Parent)
    .justify(Center);

    boxed(
        card(CONTENT_WIDTH, CONTENT_HEIGHT, scheme.surface_container_high).child(
            Row::new(vec![boxed(art), boxed(info)])
                .gap(28.0)
                .align(Center)
                .height(Parent),
        ),
    )
}

// ---- weather --------------------------------------------------------------

const WEATHER_CURRENT: f32 = 164.0;

fn weather_tab(scheme: &Scheme) -> Box<dyn Widget> {
    let (condition, temp, humidity, wind, glyph) = weather_reading();

    let color = scheme.on_tertiary_container;

    let current = card(CONTENT_WIDTH, WEATHER_CURRENT, scheme.tertiary_container).child(
        Row::new(vec![
            boxed(badge_shape(
                glyph,
                96.0,
                8.0,
                scheme.tertiary,
                scheme.on_tertiary,
            )),
            boxed(
                Column::new(vec![
                    boxed(scale(Type::TitleMedium, condition, fade(color, 220))),
                    boxed(scale(Type::DisplayMedium, temp, color)),
                    boxed(detail_row(&humidity, &wind, color)),
                ])
                .gap(4.0)
                .width(Parent),
            ),
        ])
        .gap(24.0)
        .align(Center)
        .width(Parent)
        .height(Parent),
    );

    let hours: Vec<Box<dyn Widget>> = Weather::read()
        .hourly()
        .iter()
        .take(7)
        .map(|hour| boxed(hour_chip(hour, scheme)))
        .collect();

    let strip_height = CONTENT_HEIGHT - WEATHER_CURRENT - GAP;

    let strip_body: Box<dyn Widget> = if hours.is_empty() {
        boxed(
            Rectangle::new()
                .width(Parent)
                .height(Parent)
                .align_child(Center, Center)
                .child(scale(
                    Type::LabelLarge,
                    "A previsão por hora chega no próximo ciclo",
                    scheme.on_surface_variant,
                )),
        )
    } else {
        boxed(
            Row::new(hours)
                .gap(8.0)
                .width(Parent)
                .height(Parent)
                .align(Center),
        )
    };

    boxed(
        Column::new(vec![
            boxed(current),
            boxed(
                card(CONTENT_WIDTH, strip_height, scheme.surface_container_high).child(
                    amane::Stack::new(vec![strip_body])
                        .width(Parent)
                        .height(Parent),
                ),
            ),
        ])
        .gap(GAP)
        .width(Parent)
        .height(Parent),
    )
}

fn hour_chip(hour: &Hour, scheme: &Scheme) -> Rectangle {
    Rectangle::new()
        .width(76.0)
        .height(Parent)
        .fill(scheme.surface_container_highest)
        .radius(CARD_RADIUS)
        .clip()
        .child(
            Column::new(vec![
                boxed(scale(
                    Type::LabelMedium,
                    hour.label.clone(),
                    scheme.on_surface_variant,
                )),
                boxed(icon(hour.glyph, 26.0, scheme.on_surface)),
                boxed(scale(
                    Type::TitleSmall,
                    hour.temperature.clone(),
                    scheme.on_surface,
                )),
            ])
            .gap(10.0)
            .width(Parent)
            .height(Parent)
            .align(Center)
            .justify(Center),
        )
}

// ---- wallpapers -----------------------------------------------------------

const THUMB_WIDTH: f32 = 196.0;
const THUMB_HEIGHT: f32 = 112.0;

fn wallpapers(scheme: &Scheme) -> Box<dyn Widget> {
    let settings = Settings::read();

    let modes = Row::new(vec![
        boxed(m3::chip(
            "mode:dark",
            "Escuro",
            96.0,
            settings.dark(),
            scheme,
            || Settings::set_dark_live(true),
        )),
        boxed(m3::chip(
            "mode:light",
            "Claro",
            96.0,
            !settings.dark(),
            scheme,
            || Settings::set_dark_live(false),
        )),
    ])
    .gap(8.0);

    let current = String::from(settings.wallpaper());

    let rows: Vec<Box<dyn Widget>> = settings
        .wallpapers()
        .chunks(3)
        .map(|chunk| {
            let thumbs: Vec<Box<dyn Widget>> = chunk
                .iter()
                .map(|path| boxed(thumb(path, *path == current, scheme)))
                .collect();

            boxed(Row::new(thumbs).gap(GAP))
        })
        .collect();

    drop(settings);

    let list_height = CONTENT_HEIGHT - 32.0 - GAP;

    let grid: Box<dyn Widget> = if rows.is_empty() {
        boxed(empty(
            m3::WALLPAPER,
            "Nenhum papel de parede",
            "Coloque imagens em ~/Pictures/Wallpapers",
            list_height,
            scheme,
        ))
    } else {
        boxed(
            ScrollArea::new(
                "dash-wallpapers",
                Column::new(rows).gap(GAP).width(CONTENT_WIDTH),
            )
            .width(CONTENT_WIDTH)
            .height(list_height),
        )
    };

    boxed(
        Column::new(vec![boxed(modes), grid])
            .gap(GAP)
            .width(Parent)
            .height(Parent),
    )
}

fn thumb(path: &str, selected: bool, scheme: &Scheme) -> Rectangle {
    let owned = String::from(path);

    let thumb = Rectangle::new()
        .width(THUMB_WIDTH)
        .height(THUMB_HEIGHT)
        .radius(20.0)
        .clip()
        .fill(Image::cover(path).thumbnail(392, 224))
        .cursor(Pointer)
        .on_click(move |_| Settings::set_wallpaper_live(&owned));

    if selected {
        thumb.border(3.0, scheme.primary)
    } else {
        thumb
    }
}

// ---- alerts ---------------------------------------------------------------

fn alerts(scheme: &Scheme) -> Box<dyn Widget> {
    let list: Vec<amane::Notification> = Notifications::read().list().to_vec();

    let header = Row::new(vec![
        boxed(scale(
            Type::TitleMedium,
            format!("{} notificações", list.len()),
            scheme.on_surface,
        )),
        boxed(button(
            "alerts-clear",
            "Limpar tudo",
            Some(m3::CLEAR_ALL),
            Kind::Tonal,
            scheme,
            || {
                Notifications::clear();
            },
        )),
    ])
    .justify(SpaceBetween)
    .align(Center)
    .width(Parent);

    let list_height = CONTENT_HEIGHT - 40.0 - GAP;

    let body: Box<dyn Widget> = if list.is_empty() {
        boxed(empty(
            m3::BELL,
            "Tudo em dia",
            "As notificações novas aparecem aqui",
            list_height,
            scheme,
        ))
    } else {
        let rows: Vec<Box<dyn Widget>> = list
            .iter()
            .rev()
            .map(|notification| boxed(alert_card(notification, scheme)))
            .collect();

        boxed(
            ScrollArea::new(
                "dash-alerts",
                Column::new(rows).gap(8.0).width(CONTENT_WIDTH),
            )
            .width(CONTENT_WIDTH)
            .height(list_height),
        )
    };

    boxed(
        Column::new(vec![boxed(header), body])
            .gap(GAP)
            .width(Parent)
            .height(Parent),
    )
}

fn alert_card(notification: &amane::Notification, scheme: &Scheme) -> Rectangle {
    let id = notification.id();

    let summary = if notification.summary().is_empty() {
        notification.app_name()
    } else {
        notification.summary()
    };

    list_item(
        Item {
            key: &format!("dash:alert:{id}"),
            headline: summary,
            supporting: Some(notification.body()),
            selected: false,
            height: 72.0,
        },
        Some(boxed(leading_icon(
            m3::BELL,
            scheme.primary_container,
            scheme.on_primary_container,
        ))),
        Some(boxed(icon_button(
            &format!("dash:dismiss:{id}"),
            m3::CLOSE,
            36.0,
            scheme,
            move || Notifications::dismiss(id),
        ))),
        CONTENT_WIDTH - 8.0,
        scheme,
        |_| {},
    )
}

// ---- shared ---------------------------------------------------------------

fn empty(glyph: &str, title: &str, body: &str, height: f32, scheme: &Scheme) -> Rectangle {
    m3::empty_state(glyph, title, body, CONTENT_WIDTH, height, scheme)
}

fn format_time(seconds: u64) -> String {
    format!("{}:{:02}", seconds / 60, seconds % 60)
}
