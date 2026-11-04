use amane::{
    Apps, Center, Column, DesktopApp, Image, Key, LayerWindow, Monitor, Padding, Parent, Pointer,
    Rectangle, Row, ScrollArea, Service, SpaceBetween, TextInput, Widget,
};

use crate::island::widgets::{APPS, FORWARD, SEARCH, boxed, icon, label, text};
use crate::m3;
use crate::overlay::{self, Overlay, Panel, row_fill};
use crate::theme::{Scheme, Theme, fade, mix};

const WIDTH: f32 = 640.0;
const HEIGHT: f32 = 460.0;
const PADDING: f32 = 16.0;
const RESULTS: f32 = 316.0;

const INPUT: &str = "cds-launcher-input";

#[derive(Default)]
pub struct Launcher {
    query: String,
    selected: usize,
}

impl Service for Launcher {
    fn new() -> Self {
        Self::default()
    }

    // the query and the selection only change through input and ipc
    fn listen() {}
}

impl Launcher {
    pub fn query(&self) -> &str {
        &self.query
    }

    pub fn selected(&self) -> usize {
        self.selected
    }

    pub fn set_query(query: String) {
        let mut launcher = Self::write();

        launcher.query = query;
        launcher.selected = 0;
    }

    pub fn move_by(delta: isize, count: usize) {
        if count == 0 {
            return;
        }

        let mut launcher = Self::write();

        let last = count.saturating_sub(1) as isize;
        let current = (launcher.selected.min(last as usize)) as isize;

        launcher.selected = (current + delta).clamp(0, last) as usize;
    }

    pub fn reset() {
        TextInput::set_text(INPUT, "");

        let mut launcher = Self::write();

        launcher.query.clear();
        launcher.selected = 0;
    }

    pub fn toggle() {
        let showing = Overlay::read().panel() == Panel::Launcher;

        if showing {
            Overlay::close();

            return;
        }

        Self::reset();
        Overlay::open(Panel::Launcher);
    }

    pub fn open() {
        Self::reset();
        Overlay::open(Panel::Launcher);
    }

    // the entry picked with the arrows and enter
    pub fn launch_selected() {
        let (query, selected) = {
            let launcher = Self::read();

            (String::from(launcher.query()), launcher.selected())
        };

        let apps = Apps::read();
        let results = filtered(apps.list(), &query);

        let exec = if results.is_empty() {
            None
        } else {
            results
                .get(selected.min(results.len() - 1))
                .map(|app| String::from(app.exec()))
        };

        drop(results);
        drop(apps);

        let Some(exec) = exec else {
            return;
        };

        amane::spawn(&exec);
        Overlay::close();
    }
}

fn filtered<'a>(apps: &'a [DesktopApp], query: &str) -> Vec<&'a DesktopApp> {
    let needle = query.trim().to_lowercase();

    if needle.is_empty() {
        return apps.iter().collect();
    }

    apps.iter()
        .filter(|app| {
            app.name().to_lowercase().contains(&needle)
                || app
                    .description()
                    .is_some_and(|description| description.to_lowercase().contains(&needle))
        })
        .collect()
}

pub fn view(monitor: &Monitor) -> LayerWindow {
    let theme = Theme::read();
    let scheme = theme.scheme();

    let state = Overlay::read();
    let open = state.panel() == Panel::Launcher;

    let launcher = Launcher::read();
    let apps = Apps::read();

    let query = String::from(launcher.query());
    let results = filtered(apps.list(), &query);

    let selected = if results.is_empty() {
        0
    } else {
        launcher.selected().min(results.len() - 1)
    };

    let count = results.len();

    let search = overlay::search(
        "launcher",
        INPUT,
        SEARCH,
        crate::lang::text("Buscar aplicativos", "Search applications"),
        open,
        scheme,
        Launcher::set_query,
        |_| Launcher::launch_selected(),
    );

    let rows: Vec<Box<dyn Widget>> = results
        .iter()
        .take(40)
        .enumerate()
        .map(|(index, app)| boxed(result_row(scheme, app, index, index == selected)))
        .collect();

    let list: Box<dyn Widget> = if rows.is_empty() {
        boxed(empty(scheme))
    } else {
        boxed(
            ScrollArea::new("launcher-results", Column::new(rows).gap(4.0).width(Parent))
                .width(Parent)
                .height(RESULTS),
        )
    };

    let hint = if count == 0 {
        String::from(crate::lang::text("Nenhum aplicativo encontrado", "No applications match"))
    } else {
        format!(
            crate::lang::text("{count} aplicativo(s)  ·  ↑↓ selecionar  ·  Enter abrir", "{count} application(s)  ·  ↑↓ choose  ·  Enter opens")
        )
    };

    let footer = Row::new(vec![
        boxed(text(
            hint,
            11.5,
            scheme.on_surface_variant,
            amane::Weight::Medium,
        )),
        boxed(text(
            crate::lang::text("Esc fecha", crate::lang::text("Esc fecha", "Esc closes")),
            11.5,
            fade(scheme.on_surface_variant, 170),
            amane::Weight::Regular,
        )),
    ])
    .width(Parent)
    .justify(SpaceBetween);

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
            Column::new(vec![boxed(search), list, boxed(footer)])
                .width(Parent)
                .height(Parent)
                .gap(12.0),
        );

    let count = results.len();

    overlay::layer(
        "launcher",
        "cds-launcher",
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
        move |key| match key {
            Key::Down | Key::Tab => Launcher::move_by(1, count),
            Key::Up => Launcher::move_by(-1, count),
            Key::Enter => Launcher::launch_selected(),
            _ => {}
        },
    )
}

fn result_row(scheme: &Scheme, app: &DesktopApp, index: usize, selected: bool) -> Rectangle {
    let key = format!("launcher:{index}");
    let (fill, color) = row_fill(&key, selected, scheme);

    let icon_box: Box<dyn Widget> = match app.icon_path() {
        Some(path) if Image::loaded(path) => boxed(
            Rectangle::new()
                .width(36.0)
                .height(36.0)
                .radius(10.0)
                .clip()
                .fill(Image::cover(path).thumbnail(72, 72)),
        ),
        _ => boxed(
            Rectangle::new()
                .width(36.0)
                .height(36.0)
                .radius(10.0)
                .fill(scheme.primary_container)
                .align_child(Center, Center)
                .child(icon(APPS, 20.0, scheme.on_primary_container)),
        ),
    };

    let info = Column::new(vec![
        boxed(label(app.name(), 13.5, color, amane::Weight::Medium, 1)),
        boxed(label(
            app.description().unwrap_or(crate::lang::text("Aplicativo", crate::lang::text("Aplicativo", "Application"))),
            11.0,
            if selected {
                fade(color, 200)
            } else {
                scheme.on_surface_variant
            },
            amane::Weight::Regular,
            1,
        )),
    ])
    .gap(1.0)
    .width(Parent);

    let mut children: Vec<Box<dyn Widget>> = vec![icon_box, boxed(info)];

    if selected {
        children.push(boxed(icon(FORWARD, 18.0, scheme.on_secondary_container)));
    }

    let owned = key.clone();
    let exec = String::from(app.exec());
    let _name = String::from(app.name());

    Rectangle::new()
        .width(Parent)
        .height(52.0)
        .fill(fill)
        .radius(16.0)
        .padding(8.0)
        .cursor(Pointer)
        .on_hover(move |inside| crate::motion::set_hover(owned.clone(), inside))
        .on_click(move |button| {
            if button == amane::Button::Left {
                amane::spawn(&exec);
                Overlay::close();
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

fn empty(scheme: &Scheme) -> Rectangle {
    let circle = Rectangle::new()
        .width(64.0)
        .height(64.0)
        .fill(scheme.surface_container_highest)
        .radius(32.0)
        .align_child(Center, Center)
        .child(icon(SEARCH, 30.0, scheme.on_surface_variant));

    Rectangle::new()
        .width(Parent)
        .height(RESULTS)
        .align_child(Center, Center)
        .child(
            Column::new(vec![
                boxed(Row::new(vec![boxed(circle)]).width(Parent).justify(Center)),
                boxed(
                    Row::new(vec![boxed(text(
                        crate::lang::text("Nada encontrado", crate::lang::text("Nada encontrado", "Nothing found")),
                        15.0,
                        scheme.on_surface,
                        amane::Weight::Medium,
                    ))])
                    .width(Parent)
                    .justify(Center),
                ),
                boxed(
                    Row::new(vec![boxed(text(
                        crate::lang::text("Tente outro nome", crate::lang::text("Tente outro nome", "Try another name")),
                        12.0,
                        mix(scheme.on_surface_variant, scheme.on_surface, 0.1),
                        amane::Weight::Regular,
                    ))])
                    .width(Parent)
                    .justify(Center),
                ),
            ])
            .gap(8.0)
            .width(Parent)
            .align(Center),
        )
}
