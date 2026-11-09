use std::env;
use std::path::PathBuf;

use amane::{
    Apps, Center, Column, Image, Key, LayerWindow, Monitor, Padding, Parent, Pointer, Rectangle,
    Row, ScrollArea, Service, SpaceBetween, TextInput, Widget,
};

use crate::island::widgets::{FORWARD, SEARCH, boxed, icon, label, text};
use crate::m3;
use crate::overlay::{self, Overlay, Panel, row_fill};
use crate::theme::{Scheme, Theme, fade, mix};
use crate::ui::{Tab, Ui};

const WIDTH: f32 = 640.0;
const HEIGHT: f32 = 460.0;
const PADDING: f32 = 16.0;
const RESULTS: f32 = 316.0;

const INPUT: &str = "cds-launcher-input";

#[derive(Clone)]
enum Action {
    App(String),
    Settings,
    ColorPicker,
    Wallpapers,
    TmuxCommand,
    Tmux(String),
}

struct ResultEntry {
    name: String,
    description: Option<String>,
    icon_path: Option<PathBuf>,
    glyph: Option<&'static str>,
    action: Action,
}

#[derive(Default)]
pub struct Launcher {
    query: String,
    selected: usize,
}

impl Service for Launcher {
    fn new() -> Self {
        Self::default()
    }

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
        launcher.selected = (launcher.selected as isize + delta)
            .rem_euclid(count as isize) as usize;
    }

    pub fn reset() {
        TextInput::set_text(INPUT, "");
        let mut launcher = Self::write();
        launcher.query.clear();
        launcher.selected = 0;
    }

    pub fn toggle() {
        if Overlay::read().panel() == Panel::Launcher {
            Overlay::close();
        } else {
            Self::reset();
            Overlay::open(Panel::Launcher);
        }
    }

    pub fn open() {
        Self::reset();
        Overlay::open(Panel::Launcher);
    }

    pub fn launch_selected() {
        let (query, selected) = {
            let launcher = Self::read();
            (String::from(launcher.query()), launcher.selected())
        };

        let entries = filtered(&query);
        let Some(entry) = entries.get(selected.min(entries.len().saturating_sub(1))) else {
            return;
        };

        launch(entry.action.clone());
    }
}

fn filtered(query: &str) -> Vec<ResultEntry> {
    if let Some(search) = query.strip_prefix('!') {
        let needle = search.trim().to_lowercase();
        return amane::output("tmux list-sessions -F '#{session_name}'")
            .lines()
            .map(str::trim)
            .filter(|name| !name.is_empty() && name.to_lowercase().contains(&needle))
            .map(|name| ResultEntry {
                name: String::from(name),
                description: None,
                icon_path: None,
                glyph: None,
                action: Action::Tmux(String::from(name)),
            })
            .collect();
    }

    if let Some(search) = query.strip_prefix('>') {
        let needle = search.trim().to_lowercase();
        let commands = [
            ("Settings", "\u{f0493}", Action::Settings),
            ("Color picker", "\u{f03d8}", Action::ColorPicker),
            ("Tmux sessions", "\u{f018d}", Action::TmuxCommand),
            ("Wallpapers", "\u{f02e9}", Action::Wallpapers),
        ];

        return commands
            .into_iter()
            .filter(|(name, _, _)| name.to_lowercase().contains(&needle))
            .map(|(name, glyph, action)| ResultEntry {
                name: String::from(name),
                description: None,
                icon_path: None,
                glyph: Some(glyph),
                action,
            })
            .collect();
    }

    let needle = query.trim().to_lowercase();
    let apps = Apps::read();
    let settings = crate::settings::Settings::read();
    let mut results: Vec<ResultEntry> = apps
        .list()
        .iter()
        .filter(|app| {
            needle.is_empty()
                || app.name().to_lowercase().contains(&needle)
                || app.description().is_some_and(|description| {
                    description.to_lowercase().contains(&needle)
                })
        })
        .map(|app| ResultEntry {
            name: String::from(app.name()),
            description: app.description().map(String::from),
            icon_path: app.icon_path().map(PathBuf::from),
            glyph: None,
            action: Action::App(String::from(app.exec())),
        })
        .collect();

    results.sort_by_key(|entry| match &entry.action {
        Action::App(exec) => !settings.is_pinned(exec),
        _ => false,
    });
    results
}

fn launch(action: Action) {
    match action {
        Action::App(exec) => {
            amane::spawn(&exec);
            Overlay::close();
        }
        Action::Settings => Overlay::open(Panel::Config),
        Action::ColorPicker => Overlay::open(Panel::Color),
        Action::Wallpapers => {
            Overlay::close();
            Ui::open(Tab::Wallpapers);
        }
        Action::TmuxCommand => {
            Launcher::set_query(String::from("!"));
            TextInput::set_text(INPUT, "!");
        }
        Action::Tmux(session) => {
            let Ok(terminal) = env::var("TERMINAL") else {
                return;
            };
            let escaped = session.replace('\'', "'\\''");
            amane::spawn(&format!("{terminal} -- tmux attach-session -t '{escaped}'"));
            Overlay::close();
        }
    }
}

pub fn view(monitor: &Monitor) -> LayerWindow {
    let theme = Theme::read();
    let scheme = theme.scheme();

    let open = Overlay::read().panel() == Panel::Launcher;
    let launcher = Launcher::read();
    let query = String::from(launcher.query());
    let results = filtered(&query);
    let selected = if results.is_empty() {
        0
    } else {
        launcher.selected().min(results.len() - 1)
    };
    let count = results.len();

    let placeholder = if query.starts_with('!') {
        crate::lang::text("Buscar sessão Tmux", "Search tmux sessions")
    } else if query.starts_with('>') {
        crate::lang::text("Buscar comando", "Search commands")
    } else {
        crate::lang::text("Buscar aplicativos · > comandos · ! Tmux", "Search apps · > commands · ! Tmux")
    };

    let search = overlay::search(
        "launcher",
        INPUT,
        SEARCH,
        placeholder,
        open,
        scheme,
        Launcher::set_query,
        |_| Launcher::launch_selected(),
    );

    let rows: Vec<Box<dyn Widget>> = results
        .iter()
        .take(40)
        .enumerate()
        .map(|(index, entry)| boxed(result_row(scheme, entry, index, index == selected)))
        .collect();

    let list: Box<dyn Widget> = if rows.is_empty() {
        boxed(empty(scheme, &query))
    } else {
        boxed(
            ScrollArea::new("launcher-results", Column::new(rows).gap(4.0).width(Parent))
                .width(Parent)
                .height(RESULTS),
        )
    };

    let hint = if count == 0 {
        String::from(crate::lang::text("Nenhum resultado encontrado", "No results found"))
    } else if query.starts_with('!') {
        String::from(crate::lang::text("↑↓ selecionar  ·  Enter conectar  ·  Esc fechar", "↑↓ choose  ·  Enter attach  ·  Esc close"))
    } else if query.starts_with('>') {
        String::from(crate::lang::text("↑↓ selecionar  ·  Enter executar  ·  Esc fechar", "↑↓ choose  ·  Enter run  ·  Esc close"))
    } else if crate::settings::Settings::read().language() == "en" {
        format!("{count} applications  ·  ↑↓ choose  ·  Enter open  ·  right-click pins")
    } else {
        format!("{count} aplicativos  ·  ↑↓ selecionar  ·  Enter abrir  ·  clique direito fixa")
    };

    let footer = Row::new(vec![
        boxed(text(hint, 11.5, scheme.on_surface_variant, amane::Weight::Medium)),
        boxed(text(
            crate::lang::text("Esc fecha", "Esc closes"),
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
            full_screen: false,
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

fn result_row(scheme: &Scheme, entry: &ResultEntry, index: usize, selected: bool) -> Rectangle {
    let key = format!("launcher:{index}");
    let (fill, color) = row_fill(&key, selected, scheme);

    let icon_box: Box<dyn Widget> = if let Some(glyph) = entry.glyph {
        boxed(
            Rectangle::new()
                .width(36.0)
                .height(36.0)
                .radius(10.0)
                .fill(scheme.primary_container)
                .align_child(Center, Center)
                .child(icon(glyph, 20.0, scheme.on_primary_container)),
        )
    } else if let Some(path) = entry.icon_path.as_ref().filter(|path| path.is_file()) {
        boxed(
            Rectangle::new()
                .width(36.0)
                .height(36.0)
                .radius(10.0)
                .clip()
                .fill(Image::cover(path).thumbnail(72, 72)),
        )
    } else {
        let glyph = match &entry.action {
            Action::App(exec) => crate::glyphs::application(&entry.name, exec),
            Action::Tmux(_) | Action::TmuxCommand => "\u{f018d}",
            _ => "\u{f013}",
        };
        boxed(
            Rectangle::new()
                .width(36.0)
                .height(36.0)
                .radius(10.0)
                .fill(scheme.primary_container)
                .align_child(Center, Center)
                .child(icon(glyph, 20.0, scheme.on_primary_container)),
        )
    };

    let description = entry.description.as_deref().unwrap_or(match &entry.action {
        Action::Tmux(_) => "Tmux session",
        Action::App(_) => crate::lang::text("Aplicativo", "Application"),
        _ => "",
    });

    let info = Column::new(vec![
        boxed(label(&entry.name, 13.5, color, amane::Weight::Medium, 1)),
        boxed(label(
            description,
            11.0,
            if selected { fade(color, 200) } else { scheme.on_surface_variant },
            amane::Weight::Regular,
            1,
        )),
    ])
    .gap(1.0)
    .width(Parent);

    let mut children: Vec<Box<dyn Widget>> = vec![icon_box, boxed(info)];
    let pinned = match &entry.action {
        Action::App(exec) => crate::settings::Settings::read().is_pinned(exec),
        _ => false,
    };

    if pinned {
        children.push(boxed(icon(m3::PIN, 16.0, scheme.primary)));
    }
    if selected {
        children.push(boxed(icon(FORWARD, 18.0, scheme.on_secondary_container)));
    }

    let owned = key.clone();
    let action = entry.action.clone();

    Rectangle::new()
        .width(Parent)
        .height(52.0)
        .fill(fill)
        .radius(16.0)
        .padding(8.0)
        .cursor(Pointer)
        .on_hover(move |inside| crate::motion::set_hover(owned.clone(), inside))
        .on_click(move |button| match button {
            amane::Button::Left => launch(action.clone()),
            amane::Button::Right => {
                if let Action::App(exec) = &action {
                    crate::settings::Settings::toggle_pinned(exec);
                }
            }
            _ => {}
        })
        .child(
            Row::new(children)
                .gap(12.0)
                .align(Center)
                .width(Parent)
                .height(Parent),
        )
}

fn empty(scheme: &Scheme, query: &str) -> Rectangle {
    let message = if query.starts_with('!') {
        crate::lang::text("Nenhuma sessão Tmux encontrada", "No Tmux sessions found")
    } else if query.starts_with('>') {
        crate::lang::text("Nenhum comando encontrado", "No commands found")
    } else {
        crate::lang::text("Nenhum aplicativo encontrado", "No applications found")
    };

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
                        message,
                        15.0,
                        scheme.on_surface,
                        amane::Weight::Medium,
                    ))])
                    .width(Parent)
                    .justify(Center),
                ),
                boxed(
                    Row::new(vec![boxed(text(
                        crate::lang::text("Tente outro termo", "Try another term"),
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
