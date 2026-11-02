use std::time::Duration;

use amane::{Color, Parent, Rectangle, Service, Stack, Text, Widget};

use crate::motion;
use crate::theme::{mix, Scheme, Theme};

// material symbols rounded codepoints
pub const BELL: &str = "\u{e7f4}";
pub const BRIGHTNESS: &str = "\u{e3ab}";
pub const CLOSE: &str = "\u{e5cd}";
pub const MUTED: &str = "\u{e04f}";
pub const VOLUME: &str = "\u{e050}";
pub const NEXT: &str = "\u{e044}";
pub const PREVIOUS: &str = "\u{e045}";
pub const PLAY: &str = "\u{e037}";
pub const PAUSE: &str = "\u{e034}";
pub const BATTERY: &str = "\u{e1a4}";
pub const MUSIC: &str = "\u{e405}";
pub const WALLPAPER: &str = "\u{e1bc}";
pub const APPS: &str = "\u{e5c3}";
pub const SEARCH: &str = "\u{e8b6}";
pub const PASTE: &str = "\u{e14f}";
pub const COPY: &str = "\u{e14d}";
pub const DELETE: &str = "\u{e872}";
pub const POWER: &str = "\u{e8ac}";
pub const LOCK: &str = "\u{e88d}";
pub const LOGOUT: &str = "\u{e9ba}";
pub const RESTART: &str = "\u{f053}";
pub const FORWARD: &str = "\u{e5c8}";
pub const CHECK: &str = "\u{e5ca}";

// tray and menus
pub const EXPAND_MORE: &str = "\u{e5cf}";
pub const TUNE: &str = "\u{e429}";
pub const PALETTE: &str = "\u{e40a}";

// weather
pub const SUNNY: &str = "\u{e81a}";
pub const FOGGY: &str = "\u{e818}";
pub const RAINY: &str = "\u{f176}";
pub const STORM: &str = "\u{ebdb}";
pub const CLOUDY: &str = "\u{f15c}";
pub const AIR: &str = "\u{efd8}";
pub const DROP: &str = "\u{e798}";
pub const SNOWING: &str = "\u{e80f}";

const PROGRESS: Duration = Duration::from_millis(180);

pub fn boxed(widget: impl Widget + 'static) -> Box<dyn Widget> {
    Box::new(widget)
}

pub fn icon(glyph: &str, size: f32, color: Color) -> Text {
    let theme = Theme::read();
    let typography = theme.typography();

    Text::new(glyph)
        .font(&typography.icon)
        .size(size)
        .color(color)
        .tight()
}

pub fn text(
    content: impl Into<String>,
    size: f32,
    color: Color,
    weight: amane::Weight,
) -> Text {
    let theme = Theme::read();
    let typography = theme.typography();

    Text::new(content)
        .font(&typography.body)
        .size(size)
        .color(color)
        .weight(weight)
}

// wrapped text, filling the width its container gives it
pub fn label(
    content: impl Into<String>,
    size: f32,
    color: Color,
    weight: amane::Weight,
    lines: usize,
) -> Text {
    text(content, size, color, weight).wrap().max_lines(lines)
}

pub fn card(width: f32, fill: Color) -> Rectangle {
    Rectangle::new()
        .width(width)
        .height(Parent)
        .fill(fill)
        .radius(24.0)
        .padding(16.0)
}

// a material filter chip, animating its container and label between states
pub fn chip(
    key: &str,
    content: &str,
    width: f32,
    selected: bool,
    scheme: &Scheme,
    on_click: impl Fn() + 'static,
) -> Rectangle {
    let state = motion::hover(key);

    let target = if selected {
        scheme.secondary_container
    } else {
        Color::TRANSPARENT
    };

    let fill = motion::color(
        &format!("chip-fill:{key}"),
        target,
        motion::SELECT_DURATION,
    );

    let fill = mix(fill, scheme.on_surface, 0.08 * state);
    let color = motion::color(
        &format!("chip-text:{key}"),
        if selected {
            scheme.on_secondary_container
        } else {
            scheme.on_surface_variant
        },
        motion::SELECT_DURATION,
    );

    let owned = String::from(key);

    Rectangle::new()
        .width(width)
        .height(36.0)
        .fill(fill)
        .radius(18.0)
        .cursor(amane::Pointer)
        .on_hover(move |inside| motion::set_hover(owned.clone(), inside))
        .on_click(move |_| on_click())
        .align_child(amane::Center, amane::Center)
        .child(text(content, 14.0, color, amane::Weight::Medium))
}

// a material slider: a thin track with a round handle, dragged from anywhere
pub fn slider(
    key: &str,
    percent: f32,
    width: f32,
    scheme: &Scheme,
    on_change: impl Fn(f32) + 'static,
) -> Stack {
    let state = motion::hover(key);
    let percent = motion::number(&format!("slider:{key}"), percent.clamp(0.0, 1.0), PROGRESS);

    let track = mix(scheme.surface_container_highest, scheme.primary, 0.25 * state);
    let thumb = 16.0 + 4.0 * state;
    let center = (width * percent).clamp(thumb / 2.0, width - thumb / 2.0);

    let owned = String::from(key);
    let seek = move |point: amane::Point| on_change((point.x / width).clamp(0.0, 1.0));

    Stack::new(vec![
        boxed(
            Rectangle::new()
                .width(width)
                .height(4.0)
                .fill(track)
                .radius(2.0)
                .translate(0.0, 6.0),
        ),
        boxed(
            Rectangle::new()
                .width(width * percent)
                .height(4.0)
                .fill(scheme.primary)
                .radius(2.0)
                .translate(0.0, 6.0),
        ),
        boxed(
            Rectangle::new()
                .width(thumb)
                .height(thumb)
                .fill(scheme.primary)
                .radius(thumb / 2.0)
                .translate(center - thumb / 2.0, (16.0 - thumb) / 2.0),
        ),
        boxed(
            Rectangle::new()
                .width(width)
                .height(16.0)
                .fill(Color::TRANSPARENT)
                .cursor(amane::Pointer)
                .on_hover(move |inside| motion::set_hover(owned.clone(), inside))
                .on_drag(seek),
        ),
    ])
    .width(width)
    .height(16.0)
}

// a read-only progress bar, for playback position and other values
pub fn progress(key: &str, percent: f32, width: f32, scheme: &Scheme) -> Stack {
    let percent = motion::number(&format!("progress:{key}"), percent.clamp(0.0, 1.0), PROGRESS);

    Stack::new(vec![
        boxed(
            Rectangle::new()
                .width(width)
                .height(4.0)
                .fill(scheme.surface_container_highest)
                .radius(2.0)
                .translate(0.0, 6.0),
        ),
        boxed(
            Rectangle::new()
                .width(width * percent)
                .height(4.0)
                .fill(scheme.primary)
                .radius(2.0)
                .translate(0.0, 6.0),
        ),
    ])
    .width(width)
    .height(16.0)
}

// a round material button with a state layer instead of a hard hover color
pub fn round_button(
    key: &str,
    glyph: &str,
    size: f32,
    fill: Color,
    color: Color,
    on_click: impl Fn() + 'static,
) -> Rectangle {
    let state = motion::hover(key);
    let fill = mix(fill, color, 0.10 * state);
    let owned = String::from(key);

    Rectangle::new()
        .width(size)
        .height(size)
        .fill(fill)
        .radius(size / 2.0)
        .cursor(amane::Pointer)
        .on_hover(move |inside| motion::set_hover(owned.clone(), inside))
        .on_click(move |_| on_click())
        .align_child(amane::Center, amane::Center)
        .child(icon(glyph, size * 0.46, color))
}

// a button that is only an icon until the pointer arrives
pub fn icon_button(
    key: &str,
    glyph: &str,
    size: f32,
    scheme: &Scheme,
    on_click: impl Fn() + 'static,
) -> Rectangle {
    let state = motion::hover(key);
    let fill = mix(Color::TRANSPARENT, scheme.on_surface_variant, 0.12 * state);
    let owned = String::from(key);

    Rectangle::new()
        .width(size)
        .height(size)
        .fill(fill)
        .radius(size / 2.0)
        .cursor(amane::Pointer)
        .on_hover(move |inside| motion::set_hover(owned.clone(), inside))
        .on_click(move |_| on_click())
        .align_child(amane::Center, amane::Center)
        .child(icon(glyph, size * 0.56, scheme.on_surface_variant))
}
