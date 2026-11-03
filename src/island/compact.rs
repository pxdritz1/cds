use amane::{
    Button, Battery, Color, Center, Monitor, Padding, Parent, Rectangle, Row, Service,
    SpaceBetween, Widget, Workspace, Workspaces,
};

use crate::clock::Clock;
use crate::motion;
use crate::theme::{Scheme, mix};
use crate::tray::Tray;

use super::widgets::{BATTERY, EXPAND_MORE, boxed, icon, text};

const DOT: f32 = 8.0;
const FOCUSED: f32 = 22.0;
const GAP: f32 = 7.0;
const SIDE: f32 = 18.0;
const CLOCK: f32 = 48.0;
const SPREAD: f32 = 30.0;

pub fn view(monitor: &Monitor, scheme: &Scheme, alpha: f32) -> (Box<dyn Widget>, f32) {
    let workspaces = Workspaces::read();
    let clock = Clock::read();
    let battery = Battery::read();

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

    /*
     * The compact island keeps its essential controls visible. Wi-Fi and
     * audio open the control center; the chevron opens the window tray.
     */
    let mut status: Vec<Box<dyn Widget>> = vec![boxed(
        tray_button("tray-chevron", scheme, Tray::toggle),
    )];
    let mut status_width = 30.0;

    if battery.present() {
        let percent = battery.percent();

        let color = if percent <= 15 {
            scheme.error
        } else if percent <= 30 {
            scheme.tertiary
        } else {
            scheme.on_surface_variant
        };

        status.insert(0, boxed(icon(BATTERY, 18.0, color)));
        status.insert(1, boxed(text(
            format!("{percent}%"),
            11.5,
            color,
            amane::Weight::Medium,
        )));

        status_width += 8.0 + 18.0 + 4.0 + 28.0;
    }

    let time = Rectangle::new()
        .width(CLOCK)
        .height(Parent)
        .align_child(Center, Center)
        .child(text(
            clock.time(true),
            14.0,
            scheme.on_surface,
            amane::Weight::SemiBold,
        ));

    let row = Row::new(vec![
        boxed(Row::new(dots).gap(GAP).align(Center).width(dots_width)),
        boxed(time),
        boxed(
            Row::new(status)
                .gap(8.0)
                .align(Center)
                .width(status_width),
        ),
    ])
    .width(Parent)
    .height(Parent)
    .justify(SpaceBetween)
    .align(Center);

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

    let width = SIDE * 2.0 + dots_width + CLOCK + status_width + SPREAD;

    (boxed(content), width.max(160.0))
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
fn tray_button(
    key: &'static str,
    scheme: &Scheme,
    on_click: impl Fn() + 'static,
) -> Rectangle {
    let (state, open) = {
        let tray = Tray::read();

        (motion::hover(key), tray.open())
    };

    let fill = mix(
        Color::TRANSPARENT,
        scheme.on_surface_variant,
        0.12 * state,
    );

    let owned = String::from(key);

    let degrees = motion::number(key, if open { 180.0 } else { 0.0 }, motion::SELECT_DURATION);

    Rectangle::new()
        .width(30.0)
        .height(30.0)
        .fill(fill)
        .radius(15.0)
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
