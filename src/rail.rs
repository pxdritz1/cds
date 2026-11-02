/*
 * the workspaces rail: the thin vertical strip on the left edge of the
 * screen, one item per workspace of this monitor, like dankmaterial's
 * side bar. active is highlighted, urgent is red, empty is faint.
 */
use amane::{
    Button, Center, Column, Full, Layer, LayerWindow, Margin, Monitor, Parent, Pointer,
    Rectangle, Start, Widget, Workspaces,
};

use amane::Service;
use crate::m3::{self, boxed, scale, Type};
use crate::motion;
use crate::theme::{Scheme, Theme};

const WIDTH: f32 = 46.0;
const HEIGHT: f32 = 300.0;
const ITEM: f32 = 52.0;

pub fn view(monitor: &Monitor) -> LayerWindow {
    let theme = Theme::read();
    let scheme = theme.scheme();

    let items: Vec<Box<dyn Widget>> = Workspaces::read()
        .list()
        .iter()
        .filter(|workspace| {
            workspace.output().is_some_and(|output| output == monitor.name)
        })
        .cloned()
        .map(|workspace| boxed(tab_item(&workspace, scheme)))
        .collect();

    let body = Column::new(items)
        .gap(4.0)
        .width(Parent)
        .height(Parent)
        .align(Center)
        .justify(Center);

    LayerWindow::new()
        .width(WIDTH)
        .height(Full)
        .anchor_horizontal(amane::Horizontal::Left)
        .layer(Layer::Top)
        .namespace("cds-rail")
        .margin(Margin {
            top: 8,
            right: 0,
            bottom: 8,
            left: 8,
        })
        .child(
            Rectangle::new()
                .width(Parent)
                .height(Parent)
                .align_child(Start, Center)
                .child(
                    Rectangle::new()
                        .width(WIDTH)
                        .height(HEIGHT)
                        .fill(scheme.surface_container_high)
                        .radius(26.0)
                        .clip()
                        .child(body),
                ),
        )
}

// one item of the rail: the workspace index, highlighted when current
fn tab_item(workspace: &amane::Workspace, scheme: &Scheme) -> Rectangle {
    let id = workspace.id();

    let index = workspace.index();

    let urgent = workspace.urgent();

    let focused = workspace.focused();

    let active = workspace.active();

    let has_windows = workspace.windows() > 0;

    let key = format!("rail:{id}");

    let state = motion::hover(&key);

    let owned = String::from(key);

    let (fill, color) = if urgent {
        (scheme.error_container, scheme.on_error_container)
    } else if focused {
        (scheme.primary, scheme.on_primary)
    } else if active {
        (scheme.secondary_container, scheme.on_secondary_container)
    } else if has_windows {
        (scheme.surface_container_highest, scheme.on_surface_variant)
    } else {
        // an empty desk reads at half strength
        (
            crate::theme::fade(scheme.surface_container_highest, 90),
            crate::theme::fade(scheme.on_surface_variant, 160),
        )
    };

    let fill = crate::m3::state_layer(fill, color, 0.10 * state);

    Rectangle::new()
        .width(ITEM)
        .height(ITEM)
        .fill(fill)
        .cursor(Pointer)
        .align_child(Center, Center)
        .on_hover(move |inside| motion::set_hover(owned.clone(), inside))
        .on_click(move |button| {
            if button == amane::Button::Left {
                Workspaces::focus(id);
            }
        })
        .child(scale(Type::LabelLarge, index.to_string(), color))
}
