/*
 * the apps panel: the m3 app drawer. every desktop program is a real
 * tile; a click spawns the program and the panel steps down.
 */
use amane::{
    Apps, Button, Center, Column, DesktopApp, Image, Key, LayerWindow, Monitor, Padding, Parent,
    Pointer, Rectangle, Row, ScrollArea, Service, Widget,
};

use crate::m3::{self, Type, badge_shape, boxed, scale, scale_label, state_layer};
use crate::motion;
use crate::overlay::{self, Overlay, Panel};
use crate::theme::{Scheme, Theme};

const WIDTH: f32 = 760.0;
const HEIGHT: f32 = 520.0;
const PADDING: f32 = 20.0;

const COLUMNS: usize = 4;
const TILE_WIDTH: f32 = 172.0;
const TILE_HEIGHT: f32 = 112.0;

pub fn view(monitor: &Monitor) -> LayerWindow {
    let theme = Theme::read();
    let scheme = theme.scheme();

    let open = Overlay::read().panel() == Panel::Apps;

    let list = Apps::read().list().to_vec();

    let mut rows: Vec<Box<dyn Widget>> = Vec::new();

    for (row, group) in list.chunks(COLUMNS).enumerate() {
        let items: Vec<Box<dyn Widget>> = group
            .iter()
            .enumerate()
            .map(|(column, app)| boxed(app_tile(app, &scheme, row * COLUMNS + column)))
            .collect();

        rows.push(boxed(Row::new(items).gap(8.0)));
    }

    let grid: Box<dyn Widget> = if rows.is_empty() {
        boxed(m3::empty_state(
            m3::APPS,
            "Nenhum app instalado",
            "Os programas aparecem depois da primeira varredura",
            Parent,
            220.0,
            scheme,
        ))
    } else {
        boxed(
            ScrollArea::new("apps-grid", Column::new(rows).gap(8.0).width(Parent))
                .width(Parent)
                .height(Parent),
        )
    };

    let header = Row::new(vec![
        boxed(scale(Type::HeadlineSmall, "Aplicativos", scheme.on_surface)),
        boxed(Rectangle::new().width(Parent).height(1.0)),
        boxed(m3::icon_button(
            "apps:spotlight",
            m3::SEARCH,
            40.0,
            scheme,
            || {
                Overlay::open(Panel::Spotlight);
            },
        )),
    ])
    .gap(10.0)
    .align(Center)
    .width(Parent);

    let content = Rectangle::new()
        .width(Parent)
        .height(Parent)
        .padding(Padding {
            top: PADDING,
            right: PADDING,
            bottom: PADDING,
            left: PADDING,
        })
        .child(
            Column::new(vec![Box::new(header), grid])
                .width(Parent)
                .height(Parent)
                .gap(14.0),
        );

    overlay::layer(
        "apps",
        "cds-apps",
        &monitor.name,
        open,
        scheme,
        overlay::Frame {
            width: WIDTH,
            height: HEIGHT,
            radius: m3::shape::XL,
            place: overlay::Place::Under,
        },
        content,
        |key| {
            if key == Key::Escape {
                Overlay::close();
            }
        },
    )
}

fn app_tile(app: &DesktopApp, scheme: &Scheme, index: usize) -> Rectangle {
    let name = String::from(app.name());

    let key = format!("apps:{index}");

    let state = motion::hover(&key);

    let owned = String::from(key);

    let icon_box: Box<dyn Widget> = match app.icon_path() {
        Some(path) if Image::loaded(path) => {
            let pixels = 88u32;

            Box::new(
                Rectangle::new()
                    .width(44.0)
                    .height(44.0)
                    .radius(14.0)
                    .clip()
                    .fill(Image::cover(path).thumbnail(pixels, pixels)),
            )
        }
        _ => Box::new(badge_shape(
            m3::APPS,
            44.0,
            8.0,
            scheme.primary_container,
            scheme.primary,
        )),
    };

    let body = Column::new(vec![
        icon_box,
        Box::new(scale_label(Type::BodySmall, name, scheme.on_surface, 1)),
    ])
    .gap(8.0)
    .width(Parent)
    .height(Parent)
    .align(Center)
    .justify(Center);

    let app = app.clone();

    Rectangle::new()
        .width(TILE_WIDTH)
        .height(TILE_HEIGHT)
        .fill(state_layer(
            scheme.surface_container_low,
            scheme.on_surface,
            0.08 * state,
        ))
        .radius(m3::shape::MD)
        .cursor(Pointer)
        .on_hover(move |inside| motion::set_hover(owned.clone(), inside))
        .on_click(move |button| {
            if button == Button::Left {
                app.launch();

                Overlay::close();
            }
        })
        .child(body)
}
