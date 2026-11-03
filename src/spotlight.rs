/*
 * the spotlight: one centered search field, and under it everything it
 * matches — shell actions at the top, programs below. keyboard driven:
 * type, arrows, enter.
 */
use amane::{
    Center, Service,
    Apps, Button, Column, DesktopApp, Key, LayerWindow, Monitor, Padding, Parent, Pointer,
    Rectangle, Row, TextInput, Widget,
};

use crate::m3::{self, boxed, leading_icon, scale_label, Type};use crate::motion;
use crate::overlay::{self, Overlay, Panel, row_fill};
use crate::settings::Settings;
use crate::theme::{Scheme, Theme, fade};
use crate::ui::{Tab, Ui};

const WIDTH: f32 = 640.0;
const HEIGHT: f32 = 420.0;

const INPUT: &str = "cds-spotlight";

/*
 * the quick actions the spotlight knows; a fragment of the label or the
 * hint finds one, an empty query shows them all
 */
struct Spot {
    action: &'static str,
    label: &'static str,
    hint: &'static str,
    glyph: &'static str,
}

const ACTIONS: [Spot; 6] = [
    Spot {
        action: "dash",
        label: "Dash do sistema",
        hint: "relógio, clima, medidores",
        glyph: m3::WIDGETS,
    },
    Spot {
        action: "apps",
        label: "Grade de aplicativos",
        hint: "todos os programas instalados",
        glyph: m3::APPS,
    },
    Spot {
        action: "config",
        label: "Ajustes do shell",
        hint: "aparência, wallpapers, clima",
        glyph: m3::TUNE,
    },
    Spot {
        action: "wallpaper",
        label: "Trocar o wallpaper",
        hint: "um aleatório da pasta, na hora",
        glyph: m3::WALLPAPER,
    },
    Spot {
        action: "light",
        label: "Alternar claro / escuro",
        hint: "regenera as cores do matugen",
        glyph: m3::DARK_MODE,
    },
    Spot {
        action: "clipboard",
        label: "Clipboard",
        hint: "histórico de cópias",
        glyph: m3::PASTE,
    },
];

#[derive(Default)]
pub struct Spotlight {
    query: String,
    selected: usize,
}

impl amane::Service for Spotlight {
    fn new() -> Self {
        Self::default()
    }

    fn listen() {}
}

impl Spotlight {
    pub fn set_query(query: String) {
        let mut spotlight = Self::write();

        spotlight.query = query;
        spotlight.selected = 0;
    }

    pub fn move_by(delta: isize, count: usize) {
        if count == 0 {
            return;
        }

        let mut spotlight = Self::write();

        let last = count.saturating_sub(1) as isize;

        let current = spotlight.selected.min(count - 1) as isize;

        spotlight.selected = (current + delta).clamp(0, last) as usize;
    }

    pub fn reset() {
        TextInput::set_text(INPUT, "");

        let mut spotlight = Self::write();

        spotlight.query.clear();
        spotlight.selected = 0;
    }

    pub fn toggle() {
        let showing = Overlay::read().panel() == Panel::Spotlight;

        if showing {
            Overlay::close();

            return;
        }

        Self::reset();

        Overlay::open(Panel::Spotlight);
    }
}

fn matched_apps(query: &str) -> Vec<DesktopApp> {
    if query.is_empty() {
        return Vec::new();
    }

    let needle = query.to_lowercase();

    Apps::read()
        .list()
        .iter()
        .filter(|app| {
            let name = app.name().to_lowercase();

            name.contains(&needle)
                || app
                    .description()
                    .is_some_and(|description| description.to_lowercase().contains(&needle))
        })
        .take(6)
        .cloned()
        .collect()
}

fn matched_actions(query: &str) -> Vec<&'static Spot> {
    let needle = query.trim().to_lowercase();

    ACTIONS
        .iter()
        .filter(|spot| {
            needle.is_empty()
                || spot.label.to_lowercase().contains(&needle)
                || spot.hint.to_lowercase().contains(&needle)
        })
        .take(5)
        .collect()
}

// what one action does; shared by the click and by enter
fn run_action(action: &str) {
    match action {
        "dash" => {
            Overlay::close();
            Ui::open(Tab::Home);
        }
        "apps" => Overlay::open(Panel::Apps),
        "config" => Overlay::open(Panel::Config),
        "clipboard" => Overlay::open(Panel::Clipboard),
        "wallpaper" => {
            let wallpapers = Settings::read().wallpapers().to_vec();

            spot_light_wallpapers(wallpapers);
        }
        "light" => {
            let dark = Settings::read().dark();

            Settings::set_dark(!dark);
        }
        _ => {}
    }

    Overlay::close();
}

// picks one image of the wallpaper folder and regenerates the theme
fn spot_light_wallpapers(wallpapers: Vec<String>) {
    if wallpapers.is_empty() {
        return;
    }

    let seconds = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map_or(0, |elapsed| elapsed.as_secs());

    let index = (seconds % wallpapers.len() as u64) as usize;

    Settings::set_wallpaper(&wallpapers[index]);
}

pub fn view(monitor: &Monitor) -> LayerWindow {
    let theme = Theme::read();
    let scheme = theme.scheme();

    let open = Overlay::read().panel() == Panel::Spotlight;

    let query = {
        let spotlight = Spotlight::read();

        String::from(spotlight.query.as_str())
    };

    let apps = matched_apps(&query);

    let actions = matched_actions(&query);

    let count = apps.len() + actions.len();

    let selected = {
        let spotlight = Spotlight::read();

        if count == 0 {
            0
        } else {
            spotlight.selected.min(count - 1)
        }
    };

    let mut items: Vec<Box<dyn Widget>> = Vec::new();

    for (index, app) in apps.iter().enumerate() {
        items.push(boxed(app_row(app, &scheme, index, index == selected)));
    }

    for (offset, spot) in actions.iter().enumerate() {
        let index = apps.len() + offset;

        items.push(boxed(action_row(spot, &scheme, index, index == selected)));
    }

    let list: Box<dyn Widget> = if items.is_empty() {
        boxed(m3::empty_state(
            m3::SEARCH,
            "Nada encontrado",
            "Ações do shell e programas instalados respondem aqui",
            Parent,
            150.0,
            scheme,
        ))
    } else {
        boxed(Column::new(items).gap(2.0).width(Parent))
    };

    let search = overlay::search(
        "spotlight",
        INPUT,
        m3::SEARCH,
        "Buscar ações e programas…",
        open,
        scheme,
        move |text| {
            Spotlight::set_query(text);
        },
        move |_| {},
    );

    let footer = Row::new(vec![
        boxed(scale_label(
            Type::LabelMedium,
            "↑↓ navega · Enter executa",
            scheme.on_surface_variant,
            1,
        )),
        boxed(scale_label(
            Type::LabelMedium,
            "Esc fecha",
            scheme.on_surface_variant,
            1,
        )),
    ])
    .width(Parent)
    .justify(amane::SpaceBetween);

    let content: Rectangle = {
        let items = Column::new(vec![Box::new(search), list, Box::new(footer)])
            .width(Parent)
            .height(Parent)
            .gap(10.0);

        Rectangle::new()
            .width(Parent)
            .height(Parent)
            .padding(Padding {
                top: 12.0,
                right: 12.0,
                bottom: 12.0,
                left: 12.0,
            })
            .child(items)
    };

    overlay::layer(
        "spotlight",
        "cds-spotlight",
        &monitor.name,
        open,
        scheme,
        overlay::Frame {
            width: WIDTH,
            height: HEIGHT,
            radius: m3::shape::XL,
            place: overlay::Place::Spot,
        },
        content,
        move |key| match key {
            Key::Down | Key::Tab => Spotlight::move_by(1, count),
            Key::Up => Spotlight::move_by(-1, count),
            Key::Enter => run_by_index(&apps, &actions, selected),
            _ => {}
        },
    )
}

fn app_row(app: &DesktopApp, scheme: &Scheme, index: usize, selected: bool) -> Rectangle {
    let key = format!("spot-app:{index}");

    let headline = String::from(app.name());

    let supporting = String::from(app.description().unwrap_or("Programa"));

    let glyph = m3::APPS; // the icon image comes in the list; the glyph is the fallback

    let app = app.clone();

    let launch = move |button: Button| {
        if button == Button::Left {
            app.launch();

            Overlay::close();
        }
    };

    row(&key, glyph, &headline, &supporting, scheme, selected, launch)
}

fn action_row(spot: &'static Spot, scheme: &Scheme, index: usize, selected: bool) -> Rectangle {
    let key = format!("spot-action:{index}");

    let headline = String::from(spot.label);

    let supporting = String::from(spot.hint);

    let action = spot.action;

    let run = move |button: Button| {
        if button == Button::Left {
            run_action(action);
        }
    };

    row(&key, spot.glyph, &headline, &supporting, scheme, selected, run)
}

/*
 * one row of the spotlight: a leading icon in its container, the headline
 * and the supporting text, drawn flat with the hover/selected state layer
 */
fn row(
    key: &str,
    glyph: &str,
    headline: &str,
    supporting: &str,
    scheme: &Scheme,
    selected: bool,
    on_press: impl Fn(Button) + 'static,
) -> Rectangle {
    let (fill, color) = row_fill(key, selected, scheme);

    let state = motion::hover(key);

    let container_color = if selected {
        scheme.secondary_container
    } else {
        scheme.surface_container_highest
    };

    let soft = fade(color, 190);

    let owned = String::from(key);

    let lines: Vec<Box<dyn Widget>> = vec![
        boxed(scale_label(Type::BodyMedium, headline, color, 1)),
        boxed(scale_label(Type::LabelSmall, supporting, soft, 1)),
    ];

    let _ = state;

    Rectangle::new()
        .width(Parent)
        .height(52.0)
        .fill(fill)
        .radius(m3::shape::MD)
        .cursor(Pointer)
        .on_hover(move |inside| motion::set_hover(owned.clone(), inside))
        .on_click(on_press)
        .child(
            Row::new(vec![
                boxed(leading_icon(glyph, container_color, color)),
                boxed(Column::new(lines).gap(1.0).width(Parent)),
            ])
            .gap(14.0)
            .align(Center)
            .width(Parent)
            .height(Parent),
        )
}

// enter offers the highlighted row at the moment the key lands
fn run_by_index(apps: &[DesktopApp], actions: &[&Spot], selected: usize) {
    if selected < apps.len() {
        let app = apps[selected].clone();

        app.launch();

        Overlay::close();
    } else if let Some(spot) = actions.get(selected - apps.len()) {
        run_action(spot.action);
    }
}
