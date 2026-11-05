/*
 * the three islands of the bar. each one is its own surface: workspaces
 * and tray on the left, the clock in the middle (it grows into the dash)
 * and the system status on the right (it opens the control center).
 */
use std::f32::consts::TAU;
use std::time::Duration;

use amane::{
    Arc, Audio, Battery, Bluetooth, Button, Canvas, Cap, Center, Color, Image, Line, Media, Monitor, Network, Parent, Pointer, Rectangle,
    Row, Service, Shape, Stack, Widget, Workspace, Workspaces,
};

use crate::clock::Clock;
use crate::focused::FocusedWindow;
use crate::settings::Settings;
use crate::glyphs;
use crate::m3::{self, Type};
use crate::motion;
use crate::overlay::{Overlay, Panel};
use crate::theme::{Scheme, fade, mix};
use crate::tray::Tray;
use crate::weather::Weather;

use super::widgets::{EXPAND_MORE, MUSIC, boxed, icon, text};

const DOT: f32 = 8.0;
const FOCUSED: f32 = 28.0;
const FOCUSED_HEIGHT: f32 = 10.0;
const DOT_GAP: f32 = 7.0;
const SIDE: f32 = 14.0;
const ICON: f32 = 18.0;
const ITEM_GAP: f32 = 10.0;
const CLOCK_WIDTH: f32 = 260.0;
const CLOCK_CENTER_WIDTH: f32 = 166.0;

fn analog_clock(time: &str, scheme: &Scheme) -> Canvas {
    let (hour, minute) = time.split_once(':').unwrap_or(("0", "0"));
    let hour = hour.parse::<f32>().unwrap_or(0.0) % 12.0;
    let minute = minute.parse::<f32>().unwrap_or(0.0);
    let minute_angle = minute / 60.0 * TAU - TAU / 4.0;
    let hour_angle = (hour + minute / 60.0) / 12.0 * TAU - TAU / 4.0;
    let center = 20.0;
    let radius = 17.0;

    let mut shapes: Vec<Box<dyn amane::Shape>> = vec![
        Box::new(Arc::new().stroke(1.5, scheme.outline_variant)),
    ];

    for tick in 0..12 {
        let angle = tick as f32 / 12.0 * TAU - TAU / 4.0;
        let inner = radius - 3.0;
        shapes.push(Box::new(
            Line::new()
                .from(center + angle.cos() * inner, center + angle.sin() * inner)
                .to(center + angle.cos() * radius, center + angle.sin() * radius)
                .stroke(1.0, scheme.on_surface_variant)
                .cap(Cap::Round),
        ));
    }

    shapes.push(Box::new(
        Line::new()
            .from(center, center)
            .to(center + hour_angle.cos() * 9.0, center + hour_angle.sin() * 9.0)
            .stroke(2.8, scheme.primary)
            .cap(Cap::Round),
    ));
    shapes.push(Box::new(
        Line::new()
            .from(center, center)
            .to(center + minute_angle.cos() * 13.0, center + minute_angle.sin() * 13.0)
            .stroke(1.8, scheme.on_surface)
            .cap(Cap::Round),
    ));

    Canvas::new().width(40.0).height(40.0).shapes(shapes)
}

pub fn clock(scheme: &Scheme, alpha: f32) -> (Box<dyn Widget>, f32) {
    let clock = Clock::read();

    let analog = Settings::read().flag("clock_analog");
    let time = m3::scale(Type::TitleSmall, clock.time(true), scheme.on_surface);
    let date = m3::scale(
        Type::LabelMedium,
        clock.short_date(),
        scheme.on_surface_variant,
    );

    let search = Rectangle::new()
        .width(26.0)
        .height(36.0)
        .cursor(Pointer)
        .on_click(|button| {
            if button == Button::Left {
                crate::spotlight::Spotlight::toggle();
            }
        })
        .align_child(Center, Center)
        .child(icon(crate::m3::SEARCH, ICON, scheme.on_surface_variant));

    let media_playing = Media::read().playing();
    let weather = Weather::read();
    let weather_icon: Box<dyn Widget> = if media_playing {
        boxed(
            Rectangle::new()
                .width(26.0)
                .height(36.0)
                .cursor(Pointer)
                .on_click(|button| match button {
                    Button::Left => crate::ui::Ui::toggle_media_sheet(),
                    Button::Right => crate::ui::Ui::open(crate::ui::Tab::Media),
                    _ => {}
                })
                .align_child(Center, Center)
                .child(
                    Stack::new(vec![
                        boxed(m3::cookie(12.0, 26.0, scheme.primary_container)),
                        boxed(
                            Rectangle::new()
                                .width(26.0)
                                .height(26.0)
                                .align_child(Center, Center)
                                .child(icon(MUSIC, 13.0, scheme.on_primary_container)),
                        ),
                    ])
                    .width(26.0)
                    .height(26.0),
                ),
        )
    } else {
        boxed(
            Rectangle::new()
                .width(26.0)
                .height(36.0)
                .cursor(Pointer)
                .on_click(|button| {
                    if button == Button::Left {
                        crate::ui::Ui::open(crate::ui::Tab::Weather);
                    }
                })
                .align_child(Center, Center)
                .child(icon(weather.glyph(), ICON, scheme.on_surface_variant)),
        )
    };

    let clock_content = if analog {
        Row::new(vec![
            boxed(analog_clock(clock.time(true), scheme)),
            boxed(date),
        ])
        .gap(10.0)
        .align(Center)
    } else {
        Row::new(vec![
            boxed(time),
            boxed(icon("·", 10.0, fade(scheme.on_surface_variant, 140))),
            boxed(date),
        ])
        .gap(6.0)
        .align(Center)
    };

    let time_date = Rectangle::new()
        .width(CLOCK_CENTER_WIDTH)
        .height(Parent)
        .cursor(Pointer)
        .align_child(Center, Center)
        .child(clock_content);

    let row = Row::new(vec![
        boxed(search),
        boxed(time_date),
        boxed(Stack::new(vec![weather_icon]).width(26.0).height(36.0)),
    ])
    .gap(3.0)
    .align(Center)
    .width(Parent)
    .height(Parent)
    .justify(Center);

    (
        boxed(
            Rectangle::new()
                .width(Parent)
                .height(Parent)
                .opacity(alpha)
                .padding(amane::Padding {
                    top: 2.0,
                    right: 3.0,
                    bottom: 2.0,
                    left: 3.0,
                })
                .child(row),
        ),
        CLOCK_WIDTH,
    )
}

pub fn workspaces(monitor: &Monitor, scheme: &Scheme) -> (Box<dyn Widget>, f32) {
    let workspaces = Workspaces::read();

    let mut dots: Vec<Box<dyn Widget>> = Vec::new();
    let mut dots_width = 0.0;

    for workspace in workspaces.list() {
        if workspace.output() != Some(monitor.name.as_str()) {
            continue;
        }

        if !dots.is_empty() {
            dots_width += DOT_GAP;
        }

        let width = dot_width(workspace);
        let height = dot_height(workspace);
        dots_width += width;
        dots.push(boxed(dot(workspace, width, height, scheme)));
    }

    drop(workspaces);

    let (focused_title, focused_app, focused_icon) = {
        let focused = FocusedWindow::read();
        (
            String::from(focused.title()),
            String::from(focused.app_id()),
            focused.icon_path().map(String::from),
        )
    };

    let mut children: Vec<Box<dyn Widget>> = Vec::new();
    let mut width = SIDE * 2.0 - 6.0;

    if !dots.is_empty() {
        children.push(boxed(Row::new(dots).gap(DOT_GAP).align(Center)));
        width += dots_width + ITEM_GAP;
    }

    if !focused_title.is_empty() {
        let icon_widget: Box<dyn Widget> = match focused_icon.as_deref() {
            Some(path) if Image::loaded(path) => boxed(
                Rectangle::new()
                    .width(24.0)
                    .height(24.0)
                    .radius(7.0)
                    .clip()
                    .fill(Image::contain(path).thumbnail(48, 48)),
            ),
            _ => boxed(m3::icon(
                glyphs::application(&focused_title, &focused_app),
                20.0,
                scheme.on_surface_variant,
            )),
        };

        let title: String = focused_title.chars().take(24).collect();
        let title = if focused_title.chars().count() > 24 {
            format!("{title}…")
        } else {
            title
        };

        let title_widget = Rectangle::new()
            .width(170.0)
            .height(30.0)
            .clip()
            .align_child(Center, Center)
            .child(m3::scale(Type::LabelSmall, title, scheme.on_surface_variant));

        children.push(boxed(
            Row::new(vec![icon_widget, boxed(title_widget)])
                .gap(8.0)
                .align(Center),
        ));
        width += ITEM_GAP + 202.0;
    }

    let row = Row::new(children)
        .gap(ITEM_GAP)
        .align(Center)
        .width(Parent)
        .height(Parent)
        .justify(Center);

    (boxed(row), width)
}

pub fn status(scheme: &Scheme) -> (Box<dyn Widget>, f32) {
    let network = Network::read();
    let bluetooth = Bluetooth::read();
    let audio = Audio::read();
    let battery = Battery::read();

    let dim = fade(scheme.on_surface_variant, 130);
    let color = scheme.on_surface_variant;

    let mut items: Vec<Box<dyn Widget>> = Vec::new();
    let mut width = SIDE * 2.0 - 4.0;

    // Keep the expandable tray chevron immediately to the left of Wi-Fi.
    items.push(boxed(tray_button(scheme)));
    width += 28.0;

    items.push(boxed(icon(
        glyphs::wifi(&network),
        ICON,
        if network.connected() { color } else { dim },
    )));
    width += ITEM_GAP + ICON;

    if bluetooth.available() {
        let connected = bluetooth.devices().iter().any(|device| device.connected());

        items.push(boxed(icon(
            glyphs::bluetooth(true, bluetooth.powered(), connected, false),
            ICON,
            if bluetooth.powered() { color } else { dim },
        )));
        width += ITEM_GAP + ICON;
    }

    let silent = audio.muted() || audio.volume() == 0;

    items.push(boxed(icon(
        glyphs::volume(audio.volume(), audio.muted()),
        ICON,
        if silent { dim } else { color },
    )));
    width += ITEM_GAP + ICON;

    if audio.microphone_muted() {
        items.push(boxed(icon(glyphs::MIC_OFF, ICON, scheme.error)));
        width += ITEM_GAP + ICON;
    }

    if battery.present() {
        let percent = battery.percent();
        let charging = battery.charging();

        let tint = if charging {
            scheme.primary
        } else if percent <= 15 {
            scheme.error
        } else if percent <= 30 {
            scheme.tertiary
        } else {
            color
        };

        items.push(boxed(
            Row::new(vec![
                boxed(icon(glyphs::battery(percent, charging), ICON, tint)),
                boxed(text(
                    format!("{percent}%"),
                    12.0,
                    tint,
                    amane::Weight::Medium,
                )),
            ])
            .gap(4.0)
            .align(Center),
        ));

        width += ITEM_GAP + ICON + 4.0 + if percent >= 100 { 32.0 } else { 26.0 };
    }

    let row = Row::new(items)
        .gap(ITEM_GAP)
        .align(Center)
        .width(Parent)
        .height(Parent)
        .justify(Center);

    (boxed(row), width)
}

// the right island lights up while the control center is open
pub fn status_fill(scheme: &Scheme) -> Color {
    let open = Overlay::read().panel() == Panel::Control;

    let progress = motion::number(
        "status-open",
        if open { 1.0 } else { 0.0 },
        motion::SELECT_DURATION,
    );

    let state = motion::hover("status-island");

    let base = mix(
        scheme.surface_container_high,
        scheme.primary_container,
        progress,
    );

    mix(base, scheme.on_surface, 0.08 * state)
}

pub fn base_fill(scheme: &Scheme) -> Color {
    scheme.surface_container_high
}

fn tray_button(scheme: &Scheme) -> Rectangle {
    let open = Tray::read().open();

    let state = motion::hover("tray-chevron");

    let fill = mix(Color::TRANSPARENT, scheme.on_surface_variant, 0.14 * state);

    let degrees = motion::number(
        "tray-chevron",
        if open { 180.0 } else { 0.0 },
        motion::SELECT_DURATION,
    );

    Rectangle::new()
        .width(28.0)
        .height(28.0)
        .fill(fill)
        .radius(14.0)
        .cursor(Pointer)
        .on_hover(|inside| motion::set_hover("tray-chevron", inside))
        .on_click(|button| {
            if button == Button::Left {
                Tray::toggle();
            }
        })
        .align_child(Center, Center)
        .child(
            Rectangle::new()
                .width(22.0)
                .height(22.0)
                .rotate(degrees)
                .align_child(Center, Center)
                .child(icon(EXPAND_MORE, 20.0, scheme.on_surface_variant)),
        )
}

// the focused workspace widens into a pill while the others stay dots
fn dot_width(workspace: &Workspace) -> f32 {
    let target = if workspace.focused() { FOCUSED } else { DOT };

    motion::number(
        &format!("workspace:{}", workspace.id()),
        target,
        motion::WORKSPACE_DURATION,
    )
}

fn dot_height(workspace: &Workspace) -> f32 {
    motion::number(
        &format!("workspace-height:{}", workspace.id()),
        if workspace.focused() { FOCUSED_HEIGHT } else { DOT },
        motion::WORKSPACE_DURATION,
    )
}

fn dot(workspace: &Workspace, width: f32, height: f32, scheme: &Scheme) -> Rectangle {
    let target = if workspace.focused() {
        scheme.primary
    } else if workspace.urgent() {
        scheme.error
    } else {
        scheme.outline
    };

    let fill = motion::color(
        &format!("workspace-color:{}", workspace.id()),
        target,
        Duration::from_millis(280),
    );

    let id = workspace.id();

    Rectangle::new()
        .width(width)
        .height(height)
        .fill(fill)
        .radius(height / 2.0)
        .cursor(Pointer)
        .on_click(move |_| Workspaces::focus(id))
}
