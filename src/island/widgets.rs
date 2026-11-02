use amane::{Color, Parent, Rectangle, Widget, Service};
use crate::theme::{Scheme, Theme};

pub const WIFI: &str = "\u{F0624}";
pub const WIFI_OFF: &str = "\u{F0629}";
pub const BELL: &str = "\u{F0135}";
pub const BRIGHTNESS: &str = "\u{F0237}";
pub const CLOSE: &str = "\u{F0139}";
pub const MUTED: &str = "\u{F06A8}";
pub const NEXT: &str = "\u{F04AD}";
pub const PAUSE: &str = "\u{F04AE}";
pub const PLAY: &str = "\u{F04B0}";
pub const PREVIOUS: &str = "\u{F04B1}";
pub const VOLUME: &str = "\u{F06A9}";

pub fn boxed(widget: impl Widget + 'static) -> Box<dyn Widget> {
    Box::new(widget)
}

pub fn icon(glyph: &str, size: f32, color: Color) -> amane::Text {
    let theme = Theme::read();
    let typography = theme.typography();
    amane::Text::new(glyph)
        .font(&typography.icon)
        .size(size)
        .color(color)
        .tight()
}

pub fn label(
    content: impl Into<String>,
    size: f32,
    color: Color,
    weight: amane::Weight,
    width: f32,
) -> Rectangle {
    let theme = Theme::read();
    let typography = theme.typography();
    Rectangle::new()
        .width(width)
        .height(Parent)
        .align_child(amane::Center, amane::Center)
        .child(
            amane::Text::new(content)
                .font(&typography.body)
                .size(size)
                .color(color)
                .weight(weight)
                .wrap()
                .max_lines(2),
        )
}

pub fn card(width: f32, fill: Color) -> Rectangle {
    Rectangle::new()
        .width(width)
        .height(Parent)
        .fill(fill)
        .radius(24.0)
        .padding(16.0)
}

pub fn chip(
    text: &str,
    width: f32,
    selected: bool,
    scheme: &Scheme,
    on_click: impl Fn() + 'static,
) -> Rectangle {
    let (fill, color) = if selected {
        (scheme.secondary_container, scheme.on_secondary_container)
    } else {
        (scheme.surface_container_highest, scheme.on_surface)
    };

    let theme = Theme::read();
    let typography = theme.typography();
    Rectangle::new()
        .width(width)
        .height(32.0)
        .fill(fill)
        .radius(8.0)
        .cursor(amane::Pointer)
        .on_click(move |_| on_click())
        .align_child(amane::Center, amane::Center)
        .child(
            amane::Text::new(text)
                .font(&typography.body)
                .size(14.0)
                .weight(amane::Weight::Medium)
                .color(color),
        )
}

pub fn slider(
    percent: f32,
    width: f32,
    scheme: &Scheme,
    on_change: impl Fn(f32) + 'static,
) -> Rectangle {
    Rectangle::new()
        .width(width)
        .height(4.0)
        .fill(scheme.surface_container_highest)
        .radius(2.0)
        .on_move(move |point| on_change(point.x / width))
        .child(
            Rectangle::new()
                .width(width * percent)
                .height(Parent)
                .fill(scheme.primary)
                .radius(2.0),
        )
}

pub fn round_button(
    glyph: &str,
    size: f32,
    fill: Color,
    color: Color,
    on_click: impl Fn() + 'static,
) -> Rectangle {
    Rectangle::new()
        .width(size)
        .height(size)
        .fill(fill)
        .radius(size / 2.0)
        .cursor(amane::Pointer)
        .on_click(move |_| on_click())
        .align_child(amane::Center, amane::Center)
        .child(icon(glyph, size * 0.45, color))
}