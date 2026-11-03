use std::f32::consts::TAU;
use std::time::{Duration, SystemTime, UNIX_EPOCH};

use amane::{
    Arc, Button, Canvas, Cap, Center, Color, Column, Padding, Parent, Path, Pointer, Rectangle,
    Row, Service, Shadow, Shape, Size, Stack, Start, Text, Widget,
};

use crate::motion;
use crate::theme::{Scheme, Theme, fade, mix};

pub const BELL: &str = "\u{e7f4}";
pub const BELL_OFF: &str = "\u{e7f6}";
pub const BRIGHTNESS: &str = "\u{e3ab}";
pub const CLOSE: &str = "\u{e5cd}";
pub const MUTED: &str = "\u{e04f}";
pub const VOLUME: &str = "\u{e050}";
pub const VOLUME_LOW: &str = "\u{e04d}";
pub const NEXT: &str = "\u{e044}";
pub const PREVIOUS: &str = "\u{e045}";
pub const PLAY: &str = "\u{e037}";
pub const PAUSE: &str = "\u{e034}";
pub const BATTERY: &str = "\u{e1a4}";
pub const BATTERY_CHARGING: &str = "\u{e1a3}";
pub const MUSIC: &str = "\u{e405}";
pub const WALLPAPER: &str = "\u{e1bc}";
pub const APPS: &str = "\u{e5c3}";
pub const GRID: &str = "\u{e9b0}";
pub const SEARCH: &str = "\u{e8b6}";
pub const PASTE: &str = "\u{e14f}";
pub const COPY: &str = "\u{e14d}";
pub const DELETE: &str = "\u{e872}";
pub const POWER: &str = "\u{e8ac}";
pub const LOCK: &str = "\u{e897}";
pub const LOGOUT: &str = "\u{e9ba}";
pub const RESTART: &str = "\u{f053}";
pub const FORWARD: &str = "\u{e5c8}";
pub const BACK: &str = "\u{e5c4}";
pub const CHECK: &str = "\u{e5ca}";
pub const CHEVRON_RIGHT: &str = "\u{e5cc}";
pub const CHEVRON_LEFT: &str = "\u{e5cb}";
pub const EXPAND_MORE: &str = "\u{e5cf}";
pub const TUNE: &str = "\u{e429}";
pub const OPEN_NEW: &str = "\u{e89e}";
pub const SETTINGS: &str = "\u{e8b8}";
pub const HOME: &str = "\u{e88a}";
pub const CALENDAR: &str = "\u{ebcc}";
pub const DARK_MODE: &str = "\u{e51c}";
pub const LIGHT_MODE: &str = "\u{e518}";
pub const WIFI: &str = "\u{e63e}";
pub const WIFI_OFF: &str = "\u{e648}";
pub const BLUETOOTH: &str = "\u{e1a7}";
pub const BLUETOOTH_CONNECTED: &str = "\u{e1a8}";
pub const BLUETOOTH_OFF: &str = "\u{e1a9}";
pub const DND: &str = "\u{e644}";
pub const MIC: &str = "\u{e029}";
pub const MIC_OFF: &str = "\u{e02b}";
pub const SCREENSHOT: &str = "\u{e3af}";
pub const PERSON: &str = "\u{e7fd}";
pub const INFO: &str = "\u{e88e}";
pub const PALETTE: &str = "\u{e40a}";
pub const WIDGETS: &str = "\u{e1bd}";
pub const CALCULATE: &str = "\u{ea5f}";
pub const TERMINAL: &str = "\u{eb8e}";
pub const LANGUAGE: &str = "\u{e894}";
pub const ENTER: &str = "\u{e31b}";
pub const CLEAR_ALL: &str = "\u{e0b8}";
pub const BOLT: &str = "\u{ea0b}";
pub const MEMORY: &str = "\u{e322}";
pub const SPEED: &str = "\u{e9e4}";
pub const FOLDER: &str = "\u{e2c7}";
pub const REFRESH: &str = "\u{e5d5}";
pub const KEYBOARD: &str = "\u{e312}";
pub const WINDOWS: &str = "\u{e8ab}";
pub const SUNNY: &str = "\u{e81a}";
pub const FOGGY: &str = "\u{e818}";
pub const RAINY: &str = "\u{f176}";
pub const STORM: &str = "\u{ebdb}";
pub const CLOUDY: &str = "\u{f15c}";
pub const AIR: &str = "\u{efd8}";
pub const DROP: &str = "\u{e798}";
pub const SNOWING: &str = "\u{e80f}";

pub mod shape {
    // Material 3 Expressive shape tokens
    pub const NONE: f32 = 0.0;
    pub const EXTRA_SMALL: f32 = 4.0;
    pub const SMALL: f32 = 8.0;
    pub const MEDIUM: f32 = 12.0;
    pub const LARGE: f32 = 16.0;
    pub const LARGE_PLUS: f32 = 20.0;
    pub const EXTRA_LARGE: f32 = 28.0;
    pub const EXTRA_LARGE_PLUS: f32 = 32.0;
    pub const DOUBLE_EXTRA_LARGE: f32 = 48.0;

    // Component-specific shape constants
    pub const CORNER_SMALL: f32 = 8.0;
    pub const CORNER_MEDIUM: f32 = 12.0;
    pub const CORNER_LARGE: f32 = 16.0;
    pub const CORNER_EXTRA_LARGE: f32 = 28.0;
    pub const CORNER_FULL: f32 = 9999.0;

    // Island and bar specific shapes
    pub const ISLAND_COMPACT: f32 = 20.0;
    pub const ISLAND_EXPANDED: f32 = 16.0;
    pub const ISLAND_ACTIVE: f32 = 24.0;

    // Legacy aliases for backward compatibility
    pub const XS: f32 = EXTRA_SMALL;
    pub const SM: f32 = SMALL;
    pub const MD: f32 = MEDIUM;
    pub const LG: f32 = LARGE;
    pub const LG_PLUS: f32 = LARGE_PLUS;
    pub const XL: f32 = EXTRA_LARGE;
    pub const XL_PLUS: f32 = EXTRA_LARGE_PLUS;
    pub const XXL: f32 = DOUBLE_EXTRA_LARGE;
}

#[derive(Clone, Copy)]
pub enum Type {
    // Display typography (Material 3 Expressive)
    DisplayLarge,
    DisplayMedium,
    DisplaySmall,

    // Headline typography
    HeadlineLarge,
    HeadlineMedium,
    HeadlineSmall,

    // Title typography
    TitleLarge,
    TitleMedium,
    TitleSmall,

    // Body typography
    BodyLarge,
    BodyMedium,
    BodySmall,

    // Label typography (for buttons, chips, etc.)
    LabelLarge,
    LabelMedium,
    LabelSmall,
}

impl Type {
    fn spec(self) -> (f32, amane::Weight) {
        use amane::Weight::{Medium, Regular};

        match self {
            // Display typography (from Material 3 spec)
            Type::DisplayLarge => (57.0, Regular), // 57sp, Regular weight
            Type::DisplayMedium => (45.0, Regular), // 45sp, Regular weight
            Type::DisplaySmall => (36.0, Regular), // 36sp, Regular weight

            // Headline typography
            Type::HeadlineLarge => (32.0, Regular), // 32sp, Regular weight
            Type::HeadlineMedium => (28.0, Regular), // 28sp, Regular weight
            Type::HeadlineSmall => (24.0, Regular), // 24sp, Regular weight

            // Title typography
            Type::TitleLarge => (22.0, Regular), // 22sp, Regular weight
            Type::TitleMedium => (16.0, Medium), // 16sp, Medium weight
            Type::TitleSmall => (14.0, Medium),  // 14sp, Medium weight

            // Body typography
            Type::BodyLarge => (16.0, Regular), // 16sp, Regular weight
            Type::BodyMedium => (14.0, Regular), // 14sp, Regular weight
            Type::BodySmall => (12.0, Regular), // 12sp, Regular weight

            // Label typography (for buttons, chips, etc.)
            Type::LabelLarge => (14.0, Medium), // 14sp, Medium weight
            Type::LabelMedium => (12.0, Medium), // 12sp, Medium weight
            Type::LabelSmall => (11.0, Medium), // 11sp, Medium weight
        }
    }

    // Helper methods for Material 3 Expressive typographic scales
    fn line_height(self) -> f32 {
        match self {
            Type::DisplayLarge => 64.0,
            Type::DisplayMedium => 52.0,
            Type::DisplaySmall => 44.0,
            Type::HeadlineLarge => 40.0,
            Type::HeadlineMedium => 36.0,
            Type::HeadlineSmall => 32.0,
            Type::TitleLarge => 28.0,
            Type::TitleMedium => 24.0,
            Type::TitleSmall => 20.0,
            Type::BodyLarge => 24.0,
            Type::BodyMedium => 20.0,
            Type::BodySmall => 16.0,
            Type::LabelLarge => 20.0,
            Type::LabelMedium => 16.0,
            Type::LabelSmall => 16.0,
        }
    }

    fn tracking(self) -> f32 {
        match self {
            Type::DisplayLarge => -0.15,
            Type::DisplayMedium => -0.15,
            Type::DisplaySmall => -0.15,
            Type::HeadlineLarge => -0.15,
            Type::HeadlineMedium => -0.1,
            Type::HeadlineSmall => -0.1,
            Type::TitleLarge => -0.1,
            Type::TitleMedium => 0.0,
            Type::TitleSmall => 0.0,
            Type::BodyLarge => 0.0,
            Type::BodyMedium => 0.0,
            Type::BodySmall => 0.0,
            Type::LabelLarge => 0.0,
            Type::LabelMedium => 0.0,
            Type::LabelSmall => 0.0,
        }
    }
}

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

pub fn text(content: impl Into<String>, size: f32, color: Color, weight: amane::Weight) -> Text {
    let theme = Theme::read();
    let typography = theme.typography();

    Text::new(content)
        .font(&typography.body)
        .size(size)
        .color(color)
        .weight(weight)
}

pub fn label(
    content: impl Into<String>,
    size: f32,
    color: Color,
    weight: amane::Weight,
    lines: usize,
) -> Text {
    text(content, size, color, weight).wrap().max_lines(lines)
}

pub fn scale(kind: Type, content: impl Into<String>, color: Color) -> Text {
    let (size, weight) = kind.spec();

    text(content, size, color, weight)
}

pub fn scale_label(kind: Type, content: impl Into<String>, color: Color, lines: usize) -> Text {
    let (size, weight) = kind.spec();

    label(content, size, color, weight, lines)
}

pub fn measured(widget: &impl Widget) -> f32 {
    match widget.width() {
        Size::Fixed(pixels) => pixels,
        Size::Parent => 0.0,
    }
}

pub fn state_layer(base: Color, over: Color, amount: f32) -> Color {
    if amount <= 0.0 {
        return base;
    }

    if base.alpha() == 0 {
        return fade(over, (amount.clamp(0.0, 1.0) * 255.0) as u8);
    }

    mix(base, over, amount)
}

pub fn surface(
    width: impl Into<Size>,
    height: impl Into<Size>,
    fill: Color,
    radius: f32,
) -> Rectangle {
    Rectangle::new()
        .width(width)
        .height(height)
        .fill(fill)
        .radius(radius)
}

pub fn spacer(width: f32, height: f32) -> Rectangle {
    Rectangle::new().width(width).height(height)
}

pub fn divider(scheme: &Scheme) -> Rectangle {
    Rectangle::new()
        .width(Parent)
        .height(1.0)
        .fill(fade(scheme.outline_variant, 120))
}

pub fn centered(widget: impl Widget + 'static) -> Row {
    Row::new(vec![boxed(widget)]).width(Parent).justify(Center)
}

pub fn scallop(lobes: f32, depth: f32, size: f32, fill: Color) -> Canvas {
    const STEPS: usize = 120;

    let radius = size / 2.0;
    let depth = radius * depth;

    let mut path = Path::new();

    for step in 0..=STEPS {
        let angle = TAU * step as f32 / STEPS as f32;
        let reach = radius - depth + depth * (lobes * angle).cos();
        let (sin, cos) = angle.sin_cos();

        let x = radius + reach * cos;
        let y = radius + reach * sin;

        path = if step == 0 {
            path.move_to(x, y)
        } else {
            path.line_to(x, y)
        };
    }

    Canvas::new()
        .width(size)
        .height(size)
        .shapes(vec![Box::new(path.fill(fill))])
}

pub fn cookie(lobes: f32, size: f32, fill: Color) -> Canvas {
    scallop(lobes, 0.09, size, fill)
}

pub fn badge_shape(glyph: &str, size: f32, lobes: f32, fill: Color, color: Color) -> Stack {
    Stack::new(vec![
        boxed(scallop(lobes, 0.08, size, fill)),
        boxed(
            Rectangle::new()
                .width(size)
                .height(size)
                .align_child(Center, Center)
                .child(icon(glyph, size * 0.46, color)),
        ),
    ])
    .width(size)
    .height(size)
}

pub fn avatar(letter: &str, size: f32, scheme: &Scheme) -> Stack {
    Stack::new(vec![
        boxed(scallop(9.0, 0.06, size, scheme.primary_container)),
        boxed(
            Rectangle::new()
                .width(size)
                .height(size)
                .align_child(Center, Center)
                .child(text(
                    letter,
                    size * 0.42,
                    scheme.on_primary_container,
                    amane::Weight::SemiBold,
                )),
        ),
    ])
    .width(size)
    .height(size)
}

pub fn badge(count: usize, scheme: &Scheme) -> Rectangle {
    let content = if count > 9 {
        String::from("9+")
    } else {
        count.to_string()
    };
    let width = if count > 9 { 20.0 } else { 16.0 };

    Rectangle::new()
        .width(width)
        .height(16.0)
        .fill(scheme.error)
        .radius(8.0)
        .align_child(Center, Center)
        .child(text(
            content,
            10.0,
            scheme.on_error,
            amane::Weight::SemiBold,
        ))
}

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

    let fill = motion::color(&format!("chip-fill:{key}"), target, motion::SELECT_DURATION);
    let fill = state_layer(fill, scheme.on_surface, 0.08 * state);

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

    let mut chip = Rectangle::new()
        .width(width)
        .height(32.0)
        .fill(fill)
        .radius(shape::MEDIUM)
        .cursor(Pointer)
        .on_hover(move |inside| motion::set_hover(owned.clone(), inside))
        .on_click(move |_| on_click())
        .align_child(Center, Center)
        .child(scale(Type::LabelLarge, content, color));

    if !selected {
        chip = chip.border(1.0, fade(scheme.outline, 160));
    }

    chip
}

pub fn round_button(
    key: &str,
    glyph: &str,
    size: f32,
    fill: Color,
    color: Color,
    on_click: impl Fn() + 'static,
) -> Rectangle {
    let state = motion::hover(key);
    let fill = state_layer(fill, color, 0.10 * state);
    let owned = String::from(key);

    Rectangle::new()
        .width(size)
        .height(size)
        .fill(fill)
        .radius(size / 2.0)
        .cursor(Pointer)
        .on_hover(move |inside| motion::set_hover(owned.clone(), inside))
        .on_click(move |button| {
            if button == Button::Left {
                on_click()
            }
        })
        .align_child(Center, Center)
        .child(icon(glyph, size * 0.5, color))
}

pub fn icon_button(
    key: &str,
    glyph: &str,
    size: f32,
    scheme: &Scheme,
    on_click: impl Fn() + 'static,
) -> Rectangle {
    let state = motion::hover(key);
    let fill = state_layer(Color::TRANSPARENT, scheme.on_surface_variant, 0.12 * state);
    let owned = String::from(key);

    Rectangle::new()
        .width(size)
        .height(size)
        .fill(fill)
        .radius(size / 2.0)
        .cursor(Pointer)
        .on_hover(move |inside| motion::set_hover(owned.clone(), inside))
        .on_click(move |button| {
            if button == Button::Left {
                on_click()
            }
        })
        .align_child(Center, Center)
        .child(icon(glyph, size * 0.56, scheme.on_surface_variant))
}

pub fn toggle_icon_button(
    key: &str,
    glyph: &str,
    size: f32,
    active: bool,
    scheme: &Scheme,
    on_click: impl Fn() + 'static,
) -> Rectangle {
    let target = if active {
        scheme.primary
    } else {
        scheme.surface_container_highest
    };

    let color = if active {
        scheme.on_primary
    } else {
        scheme.on_surface_variant
    };

    let fill = motion::color(&format!("toggle:{key}"), target, motion::SELECT_DURATION);

    round_button(key, glyph, size, fill, color, on_click)
}

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum Kind {
    Elevated,
    Filled,
    Tonal,
    Outlined,
    Text,
}

pub fn button(
    key: &str,
    content: &str,
    glyph: Option<&str>,
    kind: Kind,
    scheme: &Scheme,
    on_click: impl Fn() + 'static,
) -> Rectangle {
    let (fill, color) = match kind {
        Kind::Elevated => (
            mix(scheme.surface_container_low, scheme.surface_tint, 0.05),
            scheme.on_surface,
        ),
        Kind::Filled => (scheme.primary, scheme.on_primary),
        Kind::Tonal => (scheme.secondary_container, scheme.on_secondary_container),
        Kind::Outlined | Kind::Text => (Color::TRANSPARENT, scheme.primary),
    };

    let state = motion::hover(key);
    let fill = state_layer(fill, color, 0.10 * state);

    let label = scale(Type::LabelLarge, content, color);
    let label_width = measured(&label);

    let mut children: Vec<Box<dyn Widget>> = Vec::new();
    let mut width = label_width + 48.0;

    if let Some(glyph) = glyph {
        children.push(boxed(icon(glyph, 18.0, color)));
        width = label_width + 18.0 + 8.0 + 40.0;
    }

    children.push(boxed(label));

    let owned = String::from(key);

    let mut button = Rectangle::new()
        .width(width)
        .height(40.0)
        .fill(fill)
        .radius(20.0)
        .cursor(Pointer)
        .on_hover(move |inside| motion::set_hover(owned.clone(), inside))
        .on_click(move |press| {
            if press == Button::Left {
                on_click()
            }
        })
        .align_child(Center, Center)
        .child(Row::new(children).gap(8.0).align(Center));

    if kind == Kind::Outlined {
        button = button.border(1.0, scheme.outline);
    } else if kind == Kind::Elevated {
        button = button.shadow(
            Shadow::drop(scheme.shadow)
                .opacity(0.16)
                .blur(2.0)
                .offset(0.0, 1.0),
        );
    }

    button
}

pub fn switch(
    key: &str,
    on: bool,
    scheme: &Scheme,
    on_toggle: impl Fn(bool) + 'static,
) -> Rectangle {
    let state = motion::hover(key);

    let progress = motion::number(
        &format!("switch:{key}"),
        if on { 1.0 } else { 0.0 },
        motion::SELECT_DURATION,
    );

    let track = mix(scheme.surface_container_highest, scheme.primary, progress);
    let base = 16.0 + 8.0 * progress;
    let size = base + 3.0 * state;
    let left = 8.0 + 16.0 * progress - (size - base) / 2.0;
    let thumb_color = mix(scheme.outline, scheme.on_primary, progress);
    let outline = (255.0 * (1.0 - progress)) as u8;
    let owned = String::from(key);

    let thumb = Rectangle::new()
        .width(size)
        .height(size)
        .fill(thumb_color)
        .radius(size / 2.0)
        .align_child(Center, Center)
        .child(icon(
            CHECK,
            14.0,
            fade(scheme.primary, (255.0 * progress) as u8),
        ))
        .translate(left, 0.0);

    Rectangle::new()
        .width(52.0)
        .height(32.0)
        .fill(track)
        .radius(16.0)
        .border(2.0, fade(scheme.outline, outline))
        .cursor(Pointer)
        .on_hover(move |inside| motion::set_hover(owned.clone(), inside))
        .on_click(move |button| {
            if button == Button::Left {
                on_toggle(!on)
            }
        })
        .align_child(Start, Center)
        .child(thumb)
}

fn corners(rectangle: Rectangle, left: f32, right: f32) -> Rectangle {
    rectangle
        .radius_top_left(left)
        .radius_bottom_left(left)
        .radius_top_right(right)
        .radius_bottom_right(right)
}

pub fn slider(
    key: &str,
    glyph: &str,
    fraction: f32,
    width: f32,
    scheme: &Scheme,
    on_change: impl Fn(f32) + 'static,
) -> Stack {
    const HEIGHT: f32 = 44.0;
    const GAP: f32 = 6.0;

    let state = motion::hover(key);

    let value = motion::number(
        &format!("slider:{key}"),
        fraction.clamp(0.0, 1.0),
        Duration::from_millis(90),
    );

    let handle = 4.0 - 2.0 * state;
    let center = width * value;
    let active = (center - handle / 2.0 - GAP).max(0.0);
    let inactive_x = (center + handle / 2.0 + GAP).min(width);
    let inactive = (width - inactive_x).max(0.0);

    let glyph_color = if active > 38.0 {
        scheme.on_primary
    } else {
        scheme.on_secondary_container
    };

    let owned = String::from(key);
    let seek = move |point: amane::Point| on_change((point.x / width).clamp(0.0, 1.0));

    let track = |size: f32, fill: Color, left: f32, right: f32| {
        corners(
            Rectangle::new()
                .width(size.max(0.0))
                .height(HEIGHT)
                .fill(fill),
            left,
            right,
        )
    };

    Stack::new(vec![
        boxed(track(inactive, scheme.secondary_container, 6.0, 22.0).translate(inactive_x, 0.0)),
        boxed(track(active, scheme.primary, 22.0, 6.0)),
        boxed(
            Rectangle::new()
                .width(handle)
                .height(HEIGHT)
                .fill(scheme.primary)
                .radius(handle / 2.0)
                .translate(center - handle / 2.0, 0.0),
        ),
        boxed(
            Rectangle::new()
                .width(HEIGHT)
                .height(HEIGHT)
                .align_child(Center, Center)
                .child(icon(glyph, 20.0, glyph_color)),
        ),
        boxed(
            Rectangle::new()
                .width(width)
                .height(HEIGHT)
                .cursor(Pointer)
                .on_hover(move |inside| motion::set_hover(owned.clone(), inside))
                .on_drag(seek),
        ),
    ])
    .width(width)
    .height(HEIGHT)
}

pub fn wavy(fraction: f32, width: f32, animated: bool, color: Color, track: Color) -> Canvas {
    const HEIGHT: f32 = 16.0;
    const AMPLITUDE: f32 = 2.6;
    const LENGTH: f32 = 22.0;
    const STROKE: f32 = 4.0;

    let fraction = fraction.clamp(0.0, 1.0);
    let active = width * fraction;
    let middle = HEIGHT / 2.0;

    let phase = if animated {
        amane::request_frame();

        let millis = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map_or(0, |elapsed| elapsed.as_millis() % 1600);

        TAU * millis as f32 / 1600.0
    } else {
        0.0
    };

    let mut shapes: Vec<Box<dyn Shape>> = Vec::new();

    let rest = active + STROKE + 2.0;

    if rest < width {
        shapes.push(Box::new(
            amane::Line::new()
                .from(rest, middle)
                .to(width - STROKE / 2.0, middle)
                .stroke(STROKE, track)
                .cap(Cap::Round),
        ));
    }

    if active > 1.0 {
        let wave = |x: f32| middle + AMPLITUDE * (TAU * x / LENGTH - phase).sin();

        let mut path = Path::new().move_to(STROKE / 2.0, wave(STROKE / 2.0));
        let mut x = STROKE / 2.0;

        while x < active {
            x = (x + 2.0).min(active);
            path = path.line_to(x, wave(x));
        }

        shapes.push(Box::new(path.stroke(STROKE, color).cap(Cap::Round)));
    }

    Canvas::new().width(width).height(HEIGHT).shapes(shapes)
}

pub fn ring(
    key: &str,
    fraction: f32,
    size: f32,
    thickness: f32,
    color: Color,
    track: Color,
) -> Canvas {
    let value = motion::number(
        &format!("ring:{key}"),
        fraction.clamp(0.0, 1.0),
        Duration::from_millis(360),
    );

    let mut shapes: Vec<Box<dyn Shape>> = vec![Box::new(Arc::new().stroke(thickness, track))];

    if value > 0.004 {
        shapes.push(Box::new(
            Arc::new()
                .start(0.0)
                .sweep(value * 359.0)
                .stroke(thickness, color)
                .cap(Cap::Round),
        ));
    }

    Canvas::new().width(size).height(size).shapes(shapes)
}

pub fn rail_item(
    key: &str,
    glyph: &str,
    name: &str,
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

    let fill = motion::color(&format!("rail:{key}"), target, motion::SELECT_DURATION);
    let fill = state_layer(fill, scheme.on_surface, 0.08 * state);

    let tint = motion::color(
        &format!("rail-tint:{key}"),
        if selected {
            scheme.on_secondary_container
        } else {
            scheme.on_surface_variant
        },
        motion::SELECT_DURATION,
    );

    let label_color = if selected {
        scheme.on_surface
    } else {
        scheme.on_surface_variant
    };

    let owned = String::from(key);

    let indicator = Rectangle::new()
        .width(56.0)
        .height(32.0)
        .fill(fill)
        .radius(16.0)
        .align_child(Center, Center)
        .child(icon(glyph, 24.0, tint));

    Rectangle::new()
        .width(72.0)
        .height(60.0)
        .cursor(Pointer)
        .on_hover(move |inside| motion::set_hover(owned.clone(), inside))
        .on_click(move |button| {
            if button == Button::Left {
                on_click()
            }
        })
        .child(
            Column::new(vec![
                boxed(indicator),
                boxed(scale(Type::LabelMedium, name, label_color)),
            ])
            .gap(4.0)
            .width(Parent)
            .align(Center),
        )
}

pub struct Tile<'a> {
    pub key: &'a str,
    pub glyph: &'a str,
    pub title: &'a str,
    pub subtitle: &'a str,
    pub active: bool,
}

pub fn tile(
    spec: Tile,
    width: impl Into<Size>,
    scheme: &Scheme,
    on_click: impl Fn() + 'static,
    on_detail: Option<Box<dyn Fn()>>,
) -> Rectangle {
    let Tile {
        key,
        glyph,
        title,
        subtitle,
        active,
    } = spec;

    let state = motion::hover(key);

    let progress = motion::number(
        &format!("tile:{key}"),
        if active { 1.0 } else { 0.0 },
        motion::SELECT_DURATION,
    );

    let fill = mix(scheme.surface_container_highest, scheme.primary, progress);
    let fill = state_layer(
        fill,
        mix(scheme.on_surface, scheme.on_primary, progress),
        0.08 * state,
    );
    let color = mix(scheme.on_surface, scheme.on_primary, progress);
    let soft = mix(scheme.on_surface_variant, scheme.on_primary, progress);
    let radius = 32.0 - 12.0 * progress;
    let owned = String::from(key);

    let mut children: Vec<Box<dyn Widget>> = vec![
        boxed(icon(glyph, 24.0, color)),
        boxed(
            Column::new(vec![
                boxed(scale_label(Type::LabelLarge, title, color, 1)),
                boxed(scale_label(Type::LabelSmall, subtitle, soft, 1)),
            ])
            .width(Parent),
        ),
    ];

    if let Some(detail) = on_detail {
        children.push(boxed(
            Rectangle::new()
                .width(28.0)
                .height(28.0)
                .radius(14.0)
                .cursor(Pointer)
                .align_child(Center, Center)
                .on_click(move |button| {
                    if button == Button::Left {
                        detail()
                    }
                })
                .child(icon(CHEVRON_RIGHT, 20.0, soft)),
        ));
    }

    Rectangle::new()
        .width(width)
        .height(64.0)
        .fill(fill)
        .radius(radius)
        .cursor(Pointer)
        .padding(Padding {
            top: 0.0,
            right: 10.0,
            bottom: 0.0,
            left: 18.0,
        })
        .on_hover(move |inside| motion::set_hover(owned.clone(), inside))
        .on_click(move |button| {
            if button == Button::Left {
                on_click()
            }
        })
        .child(
            Row::new(children)
                .gap(12.0)
                .align(Center)
                .width(Parent)
                .height(Parent),
        )
}

pub struct Item<'a> {
    pub key: &'a str,
    pub headline: &'a str,
    pub supporting: Option<&'a str>,
    pub selected: bool,
    pub height: f32,
}

pub fn list_item(
    item: Item,
    leading: Option<Box<dyn Widget>>,
    trailing: Option<Box<dyn Widget>>,
    width: impl Into<Size>,
    scheme: &Scheme,
    on_click: impl Fn(Button) + 'static,
) -> Rectangle {
    let Item {
        key,
        headline,
        supporting,
        selected,
        height,
    } = item;

    let state = motion::hover(key);

    let base = motion::color(
        &format!("item:{key}"),
        if selected {
            scheme.secondary_container
        } else {
            Color::TRANSPARENT
        },
        motion::SELECT_DURATION,
    );

    let fill = state_layer(base, scheme.on_surface, 0.08 * state);

    let color = if selected {
        scheme.on_secondary_container
    } else {
        scheme.on_surface
    };

    let soft = if selected {
        fade(scheme.on_secondary_container, 200)
    } else {
        scheme.on_surface_variant
    };

    let mut lines: Vec<Box<dyn Widget>> =
        vec![boxed(scale_label(Type::TitleSmall, headline, color, 1))];

    if let Some(supporting) = supporting {
        lines.push(boxed(scale_label(Type::BodySmall, supporting, soft, 1)));
    }

    let mut children: Vec<Box<dyn Widget>> = Vec::new();

    if let Some(leading) = leading {
        children.push(leading);
    }

    children.push(boxed(Column::new(lines).width(Parent)));

    if let Some(trailing) = trailing {
        children.push(trailing);
    }

    let owned = String::from(key);

    Rectangle::new()
        .width(width)
        .height(height)
        .fill(fill)
        .radius(shape::LG)
        .cursor(Pointer)
        .padding(Padding {
            top: 0.0,
            right: 12.0,
            bottom: 0.0,
            left: 12.0,
        })
        .on_hover(move |inside| motion::set_hover(owned.clone(), inside))
        .on_click(on_click)
        .child(
            Row::new(children)
                .gap(14.0)
                .align(Center)
                .width(Parent)
                .height(Parent),
        )
}

pub fn leading_icon(glyph: &str, container: Color, color: Color) -> Rectangle {
    Rectangle::new()
        .width(40.0)
        .height(40.0)
        .fill(container)
        .radius(20.0)
        .align_child(Center, Center)
        .child(icon(glyph, 20.0, color))
}

pub fn section_title(content: &str, scheme: &Scheme) -> Text {
    scale(Type::LabelLarge, content, scheme.primary)
}

pub fn empty_state(
    glyph: &str,
    title: &str,
    body: &str,
    width: impl Into<Size>,
    height: f32,
    scheme: &Scheme,
) -> Rectangle {
    Rectangle::new()
        .width(width)
        .height(height)
        .align_child(Center, Center)
        .child(
            Column::new(vec![
                boxed(centered(badge_shape(
                    glyph,
                    72.0,
                    8.0,
                    scheme.surface_container_highest,
                    scheme.on_surface_variant,
                ))),
                boxed(centered(scale(Type::TitleMedium, title, scheme.on_surface))),
                boxed(centered(scale(
                    Type::BodyMedium,
                    body,
                    scheme.on_surface_variant,
                ))),
            ])
            .gap(8.0)
            .width(Parent)
            .align(Center),
        )
}

pub fn clip_image(path: &str, size: f32, radius: f32, fallback: Color) -> Rectangle {
    use amane::Image;

    let base = Rectangle::new()
        .width(size)
        .height(size)
        .radius(radius)
        .clip();

    if Image::loaded(path) {
        let pixels = (size * 2.0).round() as u32;

        base.fill(Image::cover(path).thumbnail(pixels, pixels))
    } else {
        base.fill(fallback)
    }
}
