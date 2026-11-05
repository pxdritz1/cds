/*
 * the apps panel: the m3 app drawer. every desktop program is a real
 * tile; a click spawns the program and the panel steps down.
 */
use amane::{
    Apps, Button, Service, Center, Column, DesktopApp, Image, Key, LayerWindow, Monitor, Padding, Parent,
    Pointer, Rectangle, Row, ScrollArea, Widget,
};

use crate::m3::{self, Type, badge_shape, boxed, scale, scale_label, state_layer};
use crate::motion;
use crate::overlay::{self, Overlay, Panel};
use crate::settings::Settings;
use crate::theme::{Scheme, Theme};

const PADDING: f32 = 20.0;

const COLUMNS: usize = 6;
const TILE_WIDTH: f32 = 128.0;
const TILE_HEIGHT: f32 = 120.0;

#[derive(Default)]
pub struct AppGrid {
    selected: usize,
}

impl Service for AppGrid {
    fn new() -> Self {
        Self::default()
    }

    fn listen() {}
}

impl AppGrid {
    pub fn move_by(delta: isize, count: usize) {
        if count == 0 {
            return;
        }

        let mut state = Self::write();
        let last = count.saturating_sub(1) as isize;
        let current = state.selected.min(count - 1) as isize;
        state.selected = (current + delta).clamp(0, last) as usize;
    }

    pub fn move_vertical(delta: isize, count: usize) {
        Self::move_by(delta * COLUMNS as isize, count);
    }

    pub fn launch_selected() {
        let selected = Self::read().selected;
        let app = Apps::read().list().get(selected).cloned();

        if let Some(app) = app {
            app.launch();
            Overlay::close();
        }
    }
}

pub fn view(monitor: &Monitor) -> LayerWindow {
    let theme = Theme::read();
    let scheme = theme.scheme();

    let open = Overlay::read().panel() == Panel::Apps;

    let list = Apps::read().list().to_vec();
    let selected = if list.is_empty() {
        0
    } else {
        AppGrid::read().selected.min(list.len() - 1)
    };

    let mut rows: Vec<Box<dyn Widget>> = Vec::new();

    for (row, group) in list.chunks(COLUMNS).enumerate() {
        let items: Vec<Box<dyn Widget>> = group
            .iter()
            .enumerate()
            .map(|(column, app)| {
                let index = row * COLUMNS + column;
                boxed(app_tile(app, &scheme, index, index == selected))
            })
            .collect();

        rows.push(boxed(Row::new(items).gap(12.0).width(Parent).justify(Center)));
    }

    let grid: Box<dyn Widget> = if rows.is_empty() {
        boxed(m3::empty_state(
            m3::APPS,
            crate::lang::text("Nenhum app instalado", "No apps installed"),
            crate::lang::text("Os programas aparecem depois da primeira varredura", "Apps will appear after the first scan"),
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
        boxed(scale(Type::HeadlineSmall, crate::lang::text("Aplicativos", "Applications"), scheme.on_surface)),
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
            width: 0.0,
            height: 0.0,
            radius: m3::shape::XL,
            place: overlay::Place::Middle,
            full_screen: true,
        },
        content,
        move |key| match key {
            Key::Left => AppGrid::move_by(-1, list.len()),
            Key::Right | Key::Tab => AppGrid::move_by(1, list.len()),
            Key::Up => AppGrid::move_vertical(-1, list.len()),
            Key::Down => AppGrid::move_vertical(1, list.len()),
            Key::Enter => AppGrid::launch_selected(),
            _ => {}
        },
    )
}

fn app_tile(app: &DesktopApp, scheme: &Scheme, index: usize, selected: bool) -> Rectangle {
    let name = String::from(app.name());

    let key = format!("apps:{index}");

    let state = motion::hover(&key);
    let selected_fill = if selected {
        scheme.secondary_container
    } else {
        scheme.surface_container_low
    };

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
            crate::glyphs::application(app.name(), app.exec()),
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

    let exec = String::from(app.exec());
    let app = app.clone();

    Rectangle::new()
        .width(TILE_WIDTH)
        .height(TILE_HEIGHT)
        .fill(state_layer(
            selected_fill,
            scheme.on_surface,
            0.08 * state,
        ))
        .radius(m3::shape::MD)
        .cursor(Pointer)
        .on_hover(move |inside| motion::set_hover(owned.clone(), inside))
        .on_click(move |button| match button {
            Button::Left => {
                let count = Apps::read().list().len();
                let current = AppGrid::read().selected;
                AppGrid::move_by(index as isize - current as isize, count);
                app.launch();
                Overlay::close();
            }
            Button::Right => Settings::toggle_pinned(&exec),
            _ => {}
        })
        .child(body)
}
