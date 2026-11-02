use amane::{
    Battery, Center, Monitor, Network, Padding, Parent, Rectangle, Row, Service, SpaceBetween,
    Text, Weight, Widget, Workspace, Workspaces,
};

use crate::clock::Clock;
use crate::theme::Scheme;

use super::widgets::{WIFI, WIFI_OFF, boxed, icon};

const DOT: f32 = 8.0;
const FOCUSED: f32 = 22.0;
const GAP: f32 = 7.0;
const SIDE: f32 = 18.0;
const CLOCK: f32 = 46.0;
const SPREAD: f32 = 28.0;

pub fn view(monitor: &Monitor, scheme: &Scheme) -> (Box<dyn Widget>, f32) {
    let workspaces = Workspaces::read();
    let clock = Clock::read();
    let battery = Battery::read();
    let network = Network::read();

    let mut dots: Vec<Box<dyn Widget>> = Vec::new();
    let mut dots_width = 0.0;

    for workspace in workspaces.list() {
        if workspace.output() != Some(monitor.name.as_str()) {
            continue;
        }

        if !dots.is_empty() {
            dots_width += GAP;
        }

        let width = if workspace.focused() { FOCUSED } else { DOT };

        dots_width += width;

        dots.push(boxed(dot(workspace, width, scheme)));
    }

    let wifi = if network.connected() { WIFI } else { WIFI_OFF };

    let mut status: Vec<Box<dyn Widget>> = vec![boxed(icon(wifi, 15.0, scheme.on_surface_variant))];
    let mut status_width = 16.0;

    if battery.present() {
        status.push(boxed(
            Text::new(format!("{}%", battery.percent()))
                .size(12.0)
                .weight(Weight::Medium)
                .color(scheme.on_surface_variant),
        ));

        status_width += 8.0 + 30.0;
    }

    let time = Rectangle::new()
        .width(CLOCK)
        .height(Parent)
        .align_child(Center, Center)
        .child(
            Text::new(clock.time())
                .size(14.0)
                .weight(Weight::SemiBold)
                .color(scheme.on_surface),
        );

    let row = Row::new(vec![
        boxed(Row::new(dots).gap(GAP).align(Center).width(dots_width)),
        boxed(time),
        boxed(Row::new(status).gap(8.0).align(Center).width(status_width)),
    ])
    .width(Parent)
    .height(Parent)
    .justify(SpaceBetween)
    .align(Center);

    let content = Rectangle::new()
        .width(Parent)
        .height(Parent)
        .padding(Padding {
            top: 0.0,
            right: SIDE,
            bottom: 0.0,
            left: SIDE,
        })
        .child(row);

    let width = SIDE * 2.0 + dots_width + CLOCK + status_width + SPREAD;

    (boxed(content), width.max(150.0))
}

fn dot(workspace: &Workspace, width: f32, scheme: &Scheme) -> Rectangle {
    let fill = if workspace.focused() {
        scheme.primary
    } else if workspace.urgent() {
        scheme.error
    } else {
        scheme.outline
    };

    let id = workspace.id();

    Rectangle::new()
        .width(width)
        .height(DOT)
        .fill(fill)
        .radius(DOT / 2.0)
        .cursor(amane::Pointer)
        .on_click(move |_| Workspaces::focus(id))
}
