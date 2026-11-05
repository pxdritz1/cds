use amane::{
    Apps, Button, Center, DesktopApp, Horizontal, Image, Layer, LayerWindow, Margin, Monitor,
    Parent, Pointer, Rectangle, Row, Service, Shadow, Vertical, Widget, Zone,
};

use crate::m3;
use crate::motion;
use crate::overlay::Overlay;
use crate::settings::Settings;
use crate::theme::{Scheme, Theme, fade};

const HEIGHT: f32 = 128.0;
const SLOT: f32 = 112.0;
const ACTION_ICON: f32 = 42.0;

pub fn view(_monitor: &Monitor) -> LayerWindow {
    // Hover state is published through Ui::refresh, just like other shell
    // surfaces, so the dock must subscribe even though it has no panel state.
    crate::ui::Ui::watch();

    let settings = Settings::read();
    let mode = settings.shell_mode();
    let position = String::from(settings.dock_position());
    let alignment = String::from(settings.dock_alignment());
    let dock_enabled = settings.flag("show_dock");
    drop(settings);

    let visible = dock_enabled && (mode == 2 || mode == 3);

    if !visible {
        return LayerWindow::new()
            .width(1.0)
            .height(1.0)
            .visible(false)
            .namespace("cds-dock");
    }

    let theme = Theme::read();
    let scheme = theme.scheme();
    let apps = Apps::read();
    let settings = Settings::read();
    let pinned = settings.pinned_apps().to_vec();
    let icon_size = settings.dock_icon_size() as f32;
    let magnification = settings.flag("dock_magnification");
    let items: Vec<DesktopApp> = pinned.iter()
        .filter_map(|exec| apps.list().iter().find(|app| app.exec() == exec).cloned())
        .collect();
    drop(settings);
    drop(apps);

    let dock_width = (items.len() as f32 * SLOT + SLOT + 16.0).clamp(128.0, 1800.0);

    let mut children: Vec<Box<dyn Widget>> = items
        .iter()
        .map(|app| Box::new(app_item(app, scheme, icon_size, magnification)) as Box<dyn Widget>)
        .collect();

    children.push(Box::new(action_item(
        "dock:trash",
        m3::DELETE,
        scheme,
        || {
            amane::spawn("xdg-open trash:///");
        },
    )));

    let bar = Rectangle::new()
        .width(dock_width)
        .height(HEIGHT - 12.0)
        .fill(scheme.surface_container_high)
        .radius(30.0)
        .clip()
        .border(1.0, fade(scheme.outline_variant, 90))
        .shadow(
            Shadow::drop(scheme.shadow)
                .opacity(0.34)
                .blur(28.0)
                .offset(0.0, 8.0),
        )
        .padding(8.0)
        .child(
            Row::new(children)
                .gap(4.0)
                .align(Center)
                .justify(Center)
                .width(Parent)
                .height(Parent),
        );

    let horizontal = match alignment.as_str() {
        "left" => Horizontal::Left,
        "right" => Horizontal::Right,
        _ => Horizontal::Middle,
    };

    let margin = if position == "top" {
        Margin {
            top: 12,
            ..Margin::default()
        }
    } else {
        Margin {
            bottom: 14,
            ..Margin::default()
        }
    };

    LayerWindow::new()
        .width(dock_width)
        .height(HEIGHT)
        .anchor_vertical(if position == "top" {
            Vertical::Top
        } else {
            Vertical::Bottom
        })
        .anchor_horizontal(horizontal)
        .layer(Layer::Top)
        .space(Zone::Reserve)
        .margin(margin)
        .namespace("cds-dock")
        .visible(true)
        .child(
            Rectangle::new()
                .width(Parent)
                .height(Parent)
                .align_child(Center, Center)
                .child(bar),
        )
}

fn app_item(app: &DesktopApp, scheme: &Scheme, icon_base: f32, magnification: bool) -> Rectangle {
    let exec = String::from(app.exec());
    let key = format!("dock:{}", exec);
    let hover = motion::hover(&key);
    let scale = motion::number(
        &format!("{key}:scale"),
        if magnification && hover > 0.5 { 1.32 } else { 1.0 },
        motion::style_duration(180, 130),
    );
    let size = icon_base * scale;
    let owned = key.clone();
    let launch = exec.clone();
    let icon: Box<dyn Widget> = match app.icon_path() {
        Some(path) if Image::loaded(path) => Box::new(
            Rectangle::new()
                .width(size)
                .height(size)
                .radius(13.0)
                .clip()
                .fill(Image::contain(path).thumbnail(96, 96)),
        ),
        _ => Box::new(
            Rectangle::new()
                .width(size)
                .height(size)
                .radius(13.0)
                .fill(scheme.primary_container)
                .align_child(Center, Center)
                .child(m3::icon(
                    crate::glyphs::application(app.name(), app.exec()),
                    size * 0.58,
                    scheme.on_primary_container,
                )),
        ),
    };

    Rectangle::new()
        .width(SLOT)
        .height(HEIGHT - 28.0)
        .cursor(Pointer)
        .on_hover(move |inside| motion::set_hover(owned.clone(), inside))
        .on_click(move |button| match button {
            Button::Left => {
                amane::spawn(&launch);
                Overlay::close();
            }
            Button::Right => Settings::toggle_pinned(&launch),
            _ => {}
        })
        .align_child(Center, Center)
        .child(amane::Stack::new(vec![icon]))
}

fn action_item(
    key: &'static str,
    glyph: &str,
    scheme: &Scheme,
    action: impl Fn() + 'static,
) -> Rectangle {
    let hover = motion::hover(key);
    let scale = motion::number(
        &format!("{key}:scale"),
        if hover > 0.5 { 1.18 } else { 1.0 },
        motion::style_duration(160, 120),
    );
    let owned = String::from(key);

    Rectangle::new()
        .width(SLOT)
        .height(HEIGHT - 28.0)
        .cursor(Pointer)
        .on_hover(move |inside| motion::set_hover(owned.clone(), inside))
        .on_click(move |button| {
            if button == Button::Left {
                action();
            }
        })
        .align_child(Center, Center)
        .child(
            Rectangle::new()
                .width(ACTION_ICON * scale)
                .height(ACTION_ICON * scale)
                .radius(14.0)
                .fill(scheme.surface_container_highest)
                .align_child(Center, Center)
                .child(m3::icon(glyph, icon_base * 0.58 * scale, scheme.on_surface)),
        )
}
