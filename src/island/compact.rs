/*
 * the compact bar is a row of material 3 expressive islands: workspaces,
 * the clock and the system status. each one is a tonal pill that keeps
 * its own spacing, so the bar reads as modules instead of one rigid line.
 */
use amane::{
    Audio, Battery, Bluetooth, Button, Center, Color, Monitor, Network, Padding, Parent, Rectangle,
    Row, Service, Widget, Workspace, Workspaces,
};

use crate::clock::Clock;
use crate::m3::{self, Type};
use crate::motion;
use crate::theme::{Scheme, fade, mix};
use crate::tray::Tray;
use crate::ui::{Tab, Ui};

use super::widgets::{BATTERY, EXPAND_MORE, MUTED, VOLUME, boxed, icon, text};

const DOT: f32 = 7.0;
const FOCUSED: f32 = 22.0;
const GAP: f32 = 7.0;
const SIDE: f32 = 10.0;
const ISLAND_GAP: f32 = 8.0;
const CLOCK_WIDTH: f32 = 156.0;
const PILL: f32 = 30.0;

pub fn view(monitor: &Monitor, scheme: &Scheme, alpha: f32) -> (Box<dyn Widget>, f32) {
    let workspaces = Workspaces::read();
    let clock = Clock::read();
    let battery = Battery::read();
    let network = Network::read();
    let bluetooth = Bluetooth::read();
    let audio = Audio::read();

    // --- workspaces island -------------------------------------------------
    let mut dots: Vec<Box<dyn Widget>> = Vec::new();
    let mut dots_width = 0.0;

    for workspace in workspaces.list() {
        if workspace.output() != Some(monitor.name.as_str()) {
            continue;
        }

        if !dots.is_empty() {
            dots_width += GAP;
        }

        let width = dot_width(workspace);
        dots_width += width;
        dots.push(boxed(dot(workspace, width, scheme)));
    }

    let workspaces_width = if dots.is_empty() {
        0.0
    } else {
        dots_width + 20.0
    };

    let workspaces_island: Box<dyn Widget> = if dots.is_empty() {
        boxed(Rectangle::new().width(0.0).height(PILL))
    } else {
        boxed(pill(
            workspaces_width,
            scheme,
            Row::new(dots)
                .gap(GAP)
                .align(Center)
                .width(Parent)
                .height(Parent),
        ))
    };

    // --- clock island ------------------------------------------------------
    let time = m3::scale(Type::TitleSmall, clock.time(true), scheme.on_surface).size(14.0);
    let date = m3::scale(
        Type::LabelSmall,
        clock.short_date(),
        scheme.on_surface_variant,
    );

    let clock_body = Row::new(vec![
        boxed(time),
        boxed(icon("·", 10.0, fade(scheme.on_surface_variant, 140))),
        boxed(date),
    ])
    .gap(6.0)
    .align(Center)
    .width(Parent)
    .height(Parent);

    let clock_island = clock_pill(CLOCK_WIDTH, scheme, clock_body);

    // --- status island -----------------------------------------------------
    let mut status: Vec<Box<dyn Widget>> = Vec::new();
    let mut status_parts = 0usize;
    let mut status_width = 0.0;

    let mut add = |widget: Box<dyn Widget>, width: f32| {
        status.push(widget);
        status_width += width;
        status_parts += 1;
    };

    let wifi_glyph = if network.connected() {
        m3::WIFI
    } else {
        m3::WIFI_OFF
    };

    let wifi_color = if network.connected() {
        scheme.on_surface_variant
    } else {
        fade(scheme.on_surface_variant, 120)
    };

    add(boxed(icon(wifi_glyph, 17.0, wifi_color)), 17.0);

    if bluetooth.available() {
        let glyph = if bluetooth.powered() {
            m3::BLUETOOTH_CONNECTED
        } else {
            m3::BLUETOOTH_OFF
        };

        let color = if bluetooth.powered() {
            scheme.on_surface_variant
        } else {
            fade(scheme.on_surface_variant, 120)
        };

        add(boxed(icon(glyph, 17.0, color)), 17.0);
    }

    let volume_glyph = if audio.muted() || audio.volume() == 0 {
        MUTED
    } else {
        VOLUME
    };

    add(
        boxed(icon(volume_glyph, 17.0, scheme.on_surface_variant)),
        17.0,
    );

    if battery.present() {
        let percent = battery.percent();

        let color = if percent <= 15 {
            scheme.error
        } else if percent <= 30 {
            scheme.tertiary
        } else {
            scheme.on_surface_variant
        };

        let glyph = if battery.charging() {
            m3::BATTERY_CHARGING
        } else {
            BATTERY
        };

        add(boxed(icon(glyph, 17.0, color)), 17.0);
        add(
            boxed(text(
                format!("{percent}%"),
                11.5,
                color,
                amane::Weight::Medium,
            )),
            28.0,
        );
    }

    // the affordance that says this island expands into the quick controls
    let open = Ui::read().panel();
    let degrees = motion::number(
        "status-chevron",
        if open { 180.0 } else { 0.0 },
        motion::SELECT_DURATION,
    );

    add(
        boxed(
            Rectangle::new()
                .width(16.0)
                .height(16.0)
                .fill(Color::TRANSPARENT)
                .rotate(degrees)
                .align_child(Center, Center)
                .child(icon(EXPAND_MORE, 16.0, scheme.on_surface_variant)),
        ),
        16.0,
    );

    // the row gaps plus the pill's own horizontal padding
    if status_parts > 1 {
        status_width += (status_parts as f32 - 1.0) * 10.0;
    }

    let status_pill_width = status_width + 22.0;
    let status_island = status_pill(status_pill_width, scheme, status);

    // --- tray chevron ------------------------------------------------------
    let tray = boxed(tray_button("tray-chevron", scheme, Tray::toggle));

    // --- assembly ----------------------------------------------------------
    let row = Row::new(vec![
        workspaces_island,
        boxed(clock_island),
        boxed(status_island),
        tray,
    ])
    .gap(ISLAND_GAP)
    .width(Parent)
    .height(Parent)
    .align(Center)
    .justify(amane::Start);

    let content = Rectangle::new()
        .width(Parent)
        .height(Parent)
        .opacity(alpha)
        .padding(Padding {
            top: 0.0,
            right: SIDE,
            bottom: 0.0,
            left: SIDE,
        })
        .child(row);

    let width = SIDE * 2.0
        + workspaces_width
        + ISLAND_GAP * 3.0
        + CLOCK_WIDTH
        + status_pill_width
        + PILL
        + 2.0;

    let max = (monitor.width as f32 - 32.0).max(240.0);

    (boxed(content), width.min(max).max(200.0))
}

// a tonal material 3 pill that holds one group of the bar
fn pill(width: f32, scheme: &Scheme, child: impl Widget + 'static) -> Rectangle {
    Rectangle::new()
        .width(width)
        .height(PILL)
        .fill(pill_fill(scheme, 0.0))
        .radius(PILL / 2.0)
        .align_child(Center, Center)
        .child(child)
}

// the tonal base shared by the bar islands
fn pill_base(scheme: &Scheme) -> Color {
    mix(
        scheme.surface_container_highest,
        scheme.surface_container_high,
        0.4,
    )
}

// the base plus an optional state layer
fn pill_fill(scheme: &Scheme, state: f32) -> Color {
    mix(pill_base(scheme), scheme.on_surface, 0.04 + 0.06 * state)
}

// the clock opens the home tab, so the bar can reach the dash without a keybind
fn clock_pill(width: f32, scheme: &Scheme, child: impl Widget + 'static) -> Rectangle {
    let state = motion::hover("clock-island");

    Rectangle::new()
        .width(width)
        .height(PILL)
        .fill(pill_fill(scheme, state))
        .radius(PILL / 2.0)
        .cursor(amane::Pointer)
        .on_hover(|inside| motion::set_hover("clock-island", inside))
        .on_click(|button| {
            if button == Button::Left {
                Ui::open(Tab::Home);
            }
        })
        .align_child(Center, Center)
        .child(child)
}

// the status island doubles as the quick controls button
fn status_pill(width: f32, scheme: &Scheme, children: Vec<Box<dyn Widget>>) -> Rectangle {
    let state = motion::hover("status-island");

    let open = Ui::read().panel();
    let progress = motion::number(
        "status-open",
        if open { 1.0 } else { 0.0 },
        motion::SELECT_DURATION,
    );

    let base = pill_base(scheme);

    let fill = mix(base, scheme.primary_container, 0.65 * progress);
    let fill = mix(fill, scheme.on_surface, 0.04 + 0.06 * state);

    let row = Row::new(children)
        .gap(10.0)
        .align(Center)
        .width(Parent)
        .height(Parent);

    let owned = String::from("status-island");

    Rectangle::new()
        .width(width)
        .height(PILL)
        .fill(fill)
        .radius(PILL / 2.0)
        .padding(Padding {
            top: 0.0,
            right: 10.0,
            bottom: 0.0,
            left: 12.0,
        })
        .cursor(amane::Pointer)
        .on_hover(move |inside| motion::set_hover(owned.clone(), inside))
        .on_click(move |button| {
            if button == Button::Left {
                Ui::toggle_panel();
            }
        })
        .align_child(Center, Center)
        .child(row)
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

/*
 * the chevron that opens the tray: a round button with a hover state
 * layer, spinning half a turn while the tray is open
 */
fn tray_button(key: &'static str, scheme: &Scheme, on_click: impl Fn() + 'static) -> Rectangle {
    let (state, open) = {
        let tray = Tray::read();

        (motion::hover(key), tray.open())
    };

    let fill = mix(Color::TRANSPARENT, scheme.on_surface_variant, 0.12 * state);

    let owned = String::from(key);

    let degrees = motion::number(key, if open { 180.0 } else { 0.0 }, motion::SELECT_DURATION);

    Rectangle::new()
        .width(PILL)
        .height(PILL)
        .fill(fill)
        .radius(PILL / 2.0)
        .cursor(amane::Pointer)
        .on_hover(move |inside| motion::set_hover(owned.clone(), inside))
        .on_click(move |button| {
            if button == Button::Left {
                on_click()
            }
        })
        .align_child(Center, Center)
        .child(
            Rectangle::new()
                .width(24.0)
                .height(24.0)
                .fill(Color::TRANSPARENT)
                .rotate(degrees)
                .align_child(Center, Center)
                .child(icon(EXPAND_MORE, 20.0, scheme.on_surface_variant)),
        )
}

fn dot(workspace: &Workspace, width: f32, scheme: &Scheme) -> Rectangle {
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
        motion::SELECT_DURATION,
    );

    let id = workspace.id();

    Rectangle::new()
        .width(width)
        .height(DOT)
        .fill(fill)
        .radius(DOT / 2.0)
        .cursor(amane::Pointer)
        .on_click(move |_| Workspaces::focus(id))
}
