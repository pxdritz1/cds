use std::fs;

use amane::{
    Apps, Button, Center, Column, DesktopApp, Image, Padding, Parent, Pointer, Rectangle, Row,
    ScrollArea, Service, Stack, Widget,
};

use crate::island::widgets::{APPS, DELETE, icon, label};
use crate::theme::{Scheme, config_home, mix};

/*
 * the tray pinned to the compact bar: the chevron opens this little menu
 * of apps, and a right click on any icon shows the context menu for it.
 * pinned apps live in tray.conf, one app name per line.
 */
#[derive(Default)]
pub struct Tray {
    open: bool,
    context: Option<String>,
    pinned: Vec<String>,
}

impl Service for Tray {
    fn new() -> Self {
        Self {
            open: false,
            context: None,
            pinned: load_pinned(),
        }
    }

    // nothing polls; input and ipc change the state
    fn listen() {}
}

impl Tray {
    pub fn open(&self) -> bool {
        self.open
    }

    pub fn context(&self) -> Option<String> {
        self.context.clone()
    }

    pub fn pinned(&self) -> &[String] {
        &self.pinned
    }

    pub fn toggle() {
        let open = {
            let mut tray = Self::write();

            if tray.open {
                tray.context = None;
            }

            tray.open = !tray.open;

            tray.open
        };

        // the tray and an overlay would both want the pointer
        if open {
            crate::overlay::Overlay::close();
        }
    }

    pub fn close() {
        let mut tray = Self::write();

        tray.open = false;
        tray.context = None;
    }

    // right click on an app icon asks for its menu, by app name
    pub fn open_context(name: impl Into<String>) {
        let mut tray = Self::write();

        if tray.open {
            tray.context = Some(name.into());
        }
    }

    pub fn close_context() {
        Self::write().context = None;
    }

    pub fn pin(name: impl Into<String>) {
        let mut tray = Self::write();

        let name = name.into();

        if tray.pinned.iter().any(|pinned| pinned == &name) {
            return;
        }

        tray.pinned.push(name);
        tray.pinned.sort();

        let pinned = tray.pinned.clone();

        drop(tray);

        save(&pinned);
    }

    pub fn unpin(name: impl Into<String>) {
        let mut tray = Self::write();

        let name = name.into();

        tray.pinned.retain(|pinned| pinned != &name);

        // the context menu belongs to the app that left
        if tray.context.as_deref() == Some(name.as_str()) {
            tray.context = None;
        }

        let pinned = tray.pinned.clone();

        drop(tray);

        save(&pinned);
    }

    // finds the installed app and spawns it, closing the tray
    pub fn launch(name: impl Into<String>) {
        let mut tray = Self::write();

        let name = name.into();

        tray.open = false;
        tray.context = None;

        drop(tray);

        let exec = Apps::read()
            .list()
            .iter()
            .find(|app| app.name() == name)
            .map(|app| String::from(app.exec()));

        let Some(exec) = exec else {
            return;
        };

        amane::spawn(&exec);
    }

    /*
     * the desktop apps currently pinned, resolved against the installed
     * ones; a tray that was never pinned falls back to a seeded list of
     * the usual players, until the user pins apps they care about
     */
    pub fn resolved() -> Vec<DesktopApp> {
        let apps = Apps::read();

        let list = apps.list();

        let names: Vec<String> = {
            let tray = Self::read();

            if tray.pinned.is_empty() {
                seeded()
            } else {
                tray.pinned.clone()
            }
        };

        let mut resolved: Vec<DesktopApp> = Vec::new();

        let mut seen: Vec<String> = Vec::new();

        for name in names {
            if seen.contains(&name) {
                continue;
            }

            if let Some(app) = list.iter().find(|app| app.name() == name) {
                seen.push(name.clone());
                resolved.push(app.clone());
            }
        }

        resolved
    }
}

// a fallback tray for a first run, until the user pins apps they like
fn seeded() -> Vec<String> {
    let apps = Apps::read();

    let wanted = [
        "firefox",
        "brave",
        "chromium",
        "vesktop",
        "discord",
        "spotify",
        "thunar",
        "alacritty",
    ];

    let installed: Vec<String> = apps
        .list()
        .iter()
        .filter(|app| {
            wanted
                .iter()
                .any(|name| app.name().to_lowercase().contains(name))
        })
        .map(|app| String::from(app.name()))
        .take(8)
        .collect();

    if installed.len() >= 3 {
        return installed;
    }

    apps.list()
        .iter()
        .map(|app| String::from(app.name()))
        .take(8)
        .collect()
}

fn file() -> Option<std::path::PathBuf> {
    let folder = config_home().join("chloe-dynamic-shell");

    fs::create_dir_all(&folder).ok()?;

    Some(folder.join("tray.conf"))
}

fn save(pinned: &[String]) {
    let Some(path) = file() else {
        return;
    };

    let mut text = String::new();

    for name in pinned {
        text.push_str(name);
        text.push('\n');
    }

    let _ = fs::write(path, text);
}

// reads a saved tray, one app name per line
fn load_pinned() -> Vec<String> {
    let Some(path) = file() else {
        return Vec::new();
    };

    let Ok(text) = fs::read_to_string(path) else {
        return Vec::new();
    };

    text.lines()
        .map(|line| line.trim())
        .filter(|line| !line.is_empty())
        .map(String::from)
        .collect()
}

pub fn command(arguments: &[String]) -> String {
    let [action, rest @ ..] = arguments else {
        return usage();
    };

    let name = rest.join(" ");

    match action.as_str() {
        "toggle" => Tray::toggle(),

        "close" => Tray::close(),

        "pin" if !name.is_empty() => Tray::pin(name),

        "unpin" if !name.is_empty() => Tray::unpin(name),

        "list" => return Tray::read().pinned().join("\n"),

        _ => return usage(),
    }

    String::from("ok")
}

fn usage() -> String {
    String::from("usage: tray toggle | close | pin <app> | unpin <app> | list")
}

const GRID: usize = 4;const ICON: f32 = 46.0;
const PAD: f32 = 10.0;
const GAP: f32 = 6.0;
const ROW_HEIGHT: f32 = 44.0;
const EMPTY_HEIGHT: f32 = 46.0;

const MENU_WIDTH: f32 = PAD * 2.0 + GRID as f32 * ICON + (GRID as f32 - 1.0) * GAP;

/*
 * the menu under the compact bar while the tray is open: a grid of pinned
 * apps, or the context menu of the app that was right clicked, or a hint
 * when nothing was pinned yet.
 * the caller places it inside the island sheet, so the island keeps the
 * fill and the shadow.
 */
pub fn menu(scheme: &Scheme) -> (Box<dyn Widget>, f32, f32) {
    let context = Tray::read().context();

    let apps = Tray::resolved();

    let (body, body_height) = if let Some(name) = context {
        context_menu(scheme, &apps, &name)
    } else if apps.is_empty() {
        (
            Box::new(
                Rectangle::new()
                    .width(MENU_WIDTH)
                    .height(EMPTY_HEIGHT)
                    .align_child(Center, Center)
                    .child(label(
                        "Nenhum app fixado — fixe na aba Apps",
                        12.0,
                        scheme.on_surface_variant,
                        amane::Weight::Medium,
                        1,
                    )),
            ) as Box<dyn Widget>,
            EMPTY_HEIGHT,
        )
    } else {
        let rows = apps.len().div_ceil(GRID);

        if rows > 2 {
            (
                Box::new(
                    ScrollArea::new("tray-grid", grid(scheme, &apps))
                        .width(MENU_WIDTH)
                        .height(2.0 * ICON + GAP),
                ) as Box<dyn Widget>,
                2.0 * ICON + GAP,
            )
        } else {
            (
                Box::new(grid(scheme, &apps)) as Box<dyn Widget>,
                rows as f32 * ICON + (rows - 1) as f32 * GAP,
            )
        }
    };

    /*
     * the menu is an inset card against the sheet, so the grid does not
     * read as if it belongs to the bar
     */
    let height = body_height + PAD;

    (
        Box::new(
            Rectangle::new()
                .width(MENU_WIDTH)
                .height(height)
                .fill(scheme.surface_container_highest)
                .radius(18.0)
                .clip()
                .padding(Padding {
                    top: PAD / 2.0,
                    right: 0.0,
                    bottom: PAD / 2.0,
                    left: 0.0,
                })
                .on_click(|button| {
                    if button == Button::Left {
                        Tray::close_context();
                    }
                })
                .align_child(Center, Center)
                .child(Stack::new(vec![body])),
        ) as Box<dyn Widget>,
        MENU_WIDTH,
        height,
    )
}

// the grid of pinned apps, from four icons to a row
fn grid(scheme: &Scheme, apps: &[DesktopApp]) -> Column {
    let rows: Vec<Box<dyn Widget>> = apps
        .chunks(GRID)
        .map(|chunk| {
            let row: Vec<Box<dyn Widget>> = chunk
                .iter()
                .map(|app| Box::new(app_button(scheme, app)) as Box<dyn Widget>)
                .collect();

            Box::new(Row::new(row).gap(GAP)) as Box<dyn Widget>
        })
        .collect();

    Column::new(rows)
        .gap(GAP)
        .width(MENU_WIDTH)
        .align(Center)
}

// one of the pinned apps: left click opens it, right click asks the menu
fn app_button(scheme: &Scheme, app: &DesktopApp) -> Rectangle {
    let name = String::from(app.name());

    let state = crate::motion::hover(&format!("tray:{name}"));

    let owned = name.clone();

    let glyph_box: Box<dyn Widget> = match app.icon_path() {
        Some(path) if Image::loaded(path) => Box::new(
            Rectangle::new()
                .width(26.0)
                .height(26.0)
                .radius(8.0)
                .clip()
                .fill(Image::cover(path).thumbnail(64, 64)),
        ),
        _ => Box::new(icon(APPS, 24.0, scheme.on_surface)),
    };

    let launch_name = name.clone();

    Rectangle::new()
        .width(ICON)
        .height(ICON)
        .fill(mix(
            scheme.surface_container_highest,
            scheme.on_surface,
            0.08 * state,
        ))
        .radius(15.0)
        .cursor(Pointer)
        .on_hover(move |inside| crate::motion::set_hover(owned.clone(), inside))
        .on_click(move |button| match button {
            Button::Left => Tray::launch(launch_name.clone()),
            Button::Right => Tray::open_context(name.clone()),
            _ => {}
        })
        .align_child(Center, Center)
        .child(Stack::new(vec![glyph_box]))
}

// the context menu of one app: open it, or take it off the tray
fn context_menu(scheme: &Scheme, apps: &[DesktopApp], name: &str) -> (Box<dyn Widget>, f32) {
    if !apps.iter().any(|app| app.name() == name) {
        Tray::close_context();

        return (
            Box::new(
                Rectangle::new()
                    .width(MENU_WIDTH)
                    .height(EMPTY_HEIGHT)
                    .align_child(Center, Center)
                    .child(label(
                        "App não encontrado",
                        12.0,
                        scheme.on_surface_variant,
                        amane::Weight::Medium,
                        1,
                    )),
            ) as Box<dyn Widget>,
            EMPTY_HEIGHT,
        );
    }

    let launch_name = String::from(name);

    let unpin_name = String::from(name);

    let rows: Vec<Box<dyn Widget>> = vec![
        Box::new(context_row(scheme, "abrir", "Abrir", APPS, move || {
            Tray::launch(launch_name.clone())
        })),
        Box::new(context_row(scheme, "desafixar", "Desafixar", DELETE, move || {
            Tray::unpin(unpin_name.clone())
        })),
    ];

    (
        Box::new(
            Column::new(rows)
                .gap(4.0)
                .width(MENU_WIDTH - GAP)
                .align(Center),
        ) as Box<dyn Widget>,
        ROW_HEIGHT * 2.0 + 4.0,
    )
}

// an entry of the context menu
fn context_row(
    scheme: &Scheme,
    key: &'static str,
    label_text: &'static str,
    glyph: &'static str,
    on_click: impl Fn() + 'static,
) -> Rectangle {
    let hover_key = format!("tray-context:{key}");

    let state = crate::motion::hover(&hover_key);

    let owned = hover_key;

    Rectangle::new()
        .width(MENU_WIDTH - GAP)
        .height(ROW_HEIGHT)
        .fill(mix(
            scheme.surface_container_highest,
            scheme.on_surface,
            0.08 * state,
        ))
        .radius(12.0)
        .cursor(Pointer)
        .padding(Padding {
            top: 0.0,
            right: 6.0,
            bottom: 0.0,
            left: 14.0,
        })
        .on_hover(move |inside| crate::motion::set_hover(owned.clone(), inside))
        .on_click(move |button| {
            if button == Button::Left {
                on_click();
            }
        })
        .child(
            Row::new(vec![
                Box::new(icon(glyph, 18.0, scheme.on_surface_variant)),
                Box::new(label(
                    label_text,
                    12.5,
                    scheme.on_surface,
                    amane::Weight::Medium,
                    1,
                )),
            ])
            .gap(10.0)
            .align(Center)
            .width(Parent),
        )
}
