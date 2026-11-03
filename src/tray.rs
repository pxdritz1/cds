use std::process::Command;
use std::thread;
use std::time::Duration;

use amane::{
    Apps, Button, Center, Column, Image, Padding, Parent, Pointer, Rectangle, Row, ScrollArea,
    Service, SpaceBetween, Stack, Widget,
};

use crate::island::widgets::{APPS, CLOSE, OPEN_NEW, icon, label};
use crate::theme::{Scheme, mix};

/*
 * the tray shows the apps running in the background: every window niri
 * knows about, mostly the ones parked on another workspace.
 * a click lifts the window, a right click asks for its menu.
 */
#[derive(Clone)]
pub struct WindowEntry {
    id: i64,
    app_id: String,
    title: String,
    focused: bool,
}

#[derive(Default)]
pub struct Tray {
    open: bool,
    context: Option<i64>,
    windows: Vec<WindowEntry>,
}

impl Service for Tray {
    fn new() -> Self {
        Self::default()
    }

    // the poll runs on the service thread; fast when nothing changed
    fn interval() -> Duration {
        Duration::from_millis(2500)
    }

    // niri is asked here, off the draw thread, so the shell never waits
    fn update(&mut self) -> bool {
        // A transient niri IPC failure must not erase the last known tray.
        let Some(windows) = scanned_windows() else {
            return false;
        };

        let same = self.windows.len() == windows.len()
            && self.windows.iter().zip(windows.iter()).all(|(a, b)| {
                a.id == b.id && a.app_id == b.app_id && a.title == b.title && a.focused == b.focused
            });

        if same {
            return false;
        }

        // a window that left takes its context menu with it
        if let Some(id) = self.context {
            if !windows.iter().any(|window| window.id == id) {
                self.context = None;
            }
        }

        self.windows = windows;

        true
    }
}

impl WindowEntry {
    fn from_json(object: &str) -> Option<Self> {
        let id: i64 = json_field(object, "id")?.parse().ok()?;

        let app_id = json_field(object, "app_id")?.replace('"', "");

        let title = json_field(object, "title")?.replace('"', "");

        let focused = json_field(object, "is_focused")?.contains("true");

        Some(WindowEntry {
            id,
            app_id: app_id.trim().to_string(),
            title: title.trim().to_string(),
            focused,
        })
    }

    fn display_name(&self) -> &str {
        if self.app_id.is_empty() {
            "App"
        } else {
            &self.app_id
        }
    }
}

impl Tray {
    pub fn open(&self) -> bool {
        self.open
    }

    pub fn context(&self) -> Option<i64> {
        self.context
    }

    pub fn windows(&self) -> &[WindowEntry] {
        &self.windows
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

    // right click on a row asks for its menu, by window id
    pub fn open_context(id: i64) {
        let mut tray = Self::write();

        if tray.open {
            tray.context = Some(id);
        }
    }

    pub fn close_context() {
        Self::write().context = None;
    }

    // lifting the app hides the menu and closes the tray
    pub fn focus(id: i64) {
        {
            let mut tray = Self::write();

            tray.open = false;
            tray.context = None;
        }

        niri(&format!("action focus-window --id {id}"));
    }

    // closes the window itself, like a tray icon would
    pub fn close_window(id: i64) {
        {
            let mut tray = Self::write();

            tray.context = None;
        }

        niri(&format!("action close-window --id {id}"));
    }

    // the desktop app behind a window app id, for the real icon
    fn icon_of(app_id: &str) -> Option<String> {
        let needle = app_id.to_lowercase();

        if needle.is_empty() {
            return None;
        }

        let apps = Apps::read();

        let app = apps.list().iter().find(|app| {
            let name = app.name().to_lowercase();

            // the icon name is usually the app id, like "firefox"
            let id = app.icon().unwrap_or("").to_lowercase();

            id == needle || name == needle || name.contains(needle.as_str())
        })?;

        let path = app
            .icon_path()
            .map(|path| path.to_string_lossy().into_owned());

        drop(apps);

        path
    }
}

pub fn command(arguments: &[String]) -> String {
    let [action, rest @ ..] = arguments else {
        return usage();
    };

    match action.as_str() {
        "toggle" => Tray::toggle(),

        "close" => Tray::close(),

        "focus" => match rest.first().and_then(|id| id.parse::<i64>().ok()) {
            Some(id) => Tray::focus(id),
            None => return usage(),
        },

        "close-window" => match rest.first().and_then(|id| id.parse::<i64>().ok()) {
            Some(id) => Tray::close_window(id),
            None => return usage(),
        },

        "list" => {
            let rows: Vec<String> = Tray::read()
                .windows()
                .iter()
                .map(|window| format!("{} — {}", window.app_id, window.title))
                .collect();

            return if rows.is_empty() {
                String::from("nada rodando")
            } else {
                rows.join("\n")
            };
        }

        _ => return usage(),
    }

    String::from("ok")
}

fn usage() -> String {
    String::from("usage: tray toggle | close | list | focus <id> | close-window <id>")
}

// runs one niri command, on a side thread, so ipc answers right away
fn niri(command: &str) {
    let parts: Vec<String> = command.split_whitespace().map(String::from).collect();

    thread::spawn(move || {
        let _ = Command::new("niri").arg("msg").args(parts).output();
    });
}

// takes the window objects out of niri's plain json answer
fn scanned_windows() -> Option<Vec<WindowEntry>> {
    let output = Command::new("niri")
        .args(["msg", "--json", "windows"])
        .output()
        .ok()?;

    if !output.status.success() {
        return None;
    }

    let text = String::from_utf8_lossy(&output.stdout);

    let mut windows: Vec<WindowEntry> = Vec::new();

    for object in json_objects(&text) {
        if let Some(window) = WindowEntry::from_json(&object) {
            windows.push(window);
        }
    }

    // focused first, then alphabetical
    windows.sort_by(|a, b| b.focused.cmp(&a.focused).then(a.app_id.cmp(&b.app_id)));

    Some(windows)
}

// scans {..} objects out of a json text, aware of quoted strings
fn json_objects(json: &str) -> Vec<String> {
    let mut objects: Vec<String> = Vec::new();

    let mut depth: usize = 0;

    let mut in_string = false;

    let mut escaped = false;

    let mut start: Option<usize> = None;

    for (index, letter) in json.char_indices() {
        if in_string {
            if escaped {
                escaped = false;
            } else if letter == '\\' {
                escaped = true;
            } else if letter == '"' {
                in_string = false;
            }

            continue;
        }

        match letter {
            '"' => in_string = true,
            '{' => {
                if depth == 0 {
                    start = Some(index);
                }

                depth += 1;
            }
            '}' => {
                if depth > 0 {
                    depth -= 1;
                }

                if depth == 0 {
                    if let Some(first) = start.take() {
                        objects.push(json[first..=index].to_string());
                    }
                }
            }
            _ => {}
        }
    }

    objects
}

// reads one plain json value, like "temperature_2m": 27.9,
fn json_field<'a>(json: &'a str, key: &str) -> Option<&'a str> {
    let needle = format!("\"{key}\":");

    let start = json.find(&needle)? + needle.len();

    let rest = &json[start..];

    let end = rest
        .find(|character| character == ',' || character == '}')
        .unwrap_or(rest.len());

    Some(rest[..end].trim())
}

const ROW_HEIGHT: f32 = 46.0;

const PAD: f32 = 10.0;

const GAP: f32 = 6.0;

const EMPTY_HEIGHT: f32 = 46.0;

const MENU_WIDTH: f32 = 300.0;

const SCROLL_ROWS: f32 = 5.0;

/*
 * the menu under the compact bar while the tray is open: rows of
 * background apps, or the context menu of the row that was right clicked,
 * or a hint when nothing runs.
 * the caller places it inside the island sheet, so the island keeps the
 * fill and the shadow.
 */
pub fn menu(scheme: &Scheme) -> (Box<dyn Widget>, f32, f32) {
    let (context, windows) = {
        let tray = Tray::read();

        (tray.context(), Vec::from(tray.windows()))
    };

    let owned: Vec<(&WindowEntry, Option<String>)> = windows
        .iter()
        .map(|window| (window, Tray::icon_of(&window.app_id)))
        .collect();

    let (body, body_height) = if let Some(id) = context {
        context_menu(scheme, &owned, id)
    } else if owned.is_empty() {
        (
            Box::new(
                Rectangle::new()
                    .width(MENU_WIDTH)
                    .height(EMPTY_HEIGHT)
                    .align_child(Center, Center)
                    .child(label(
                        "Nenhum app em segundo plano",
                        12.0,
                        scheme.on_surface_variant,
                        amane::Weight::Medium,
                        1,
                    )),
            ) as Box<dyn Widget>,
            EMPTY_HEIGHT,
        )
    } else {
        let height = SCROLL_ROWS * ROW_HEIGHT + (SCROLL_ROWS - 1.0) * GAP;

        let needs_scroll = owned.len() as f32 > SCROLL_ROWS;

        let rows = Column::new(
            owned
                .iter()
                .map(|(window, icon_path)| boxed(window_row(scheme, window, icon_path.as_deref())))
                .collect(),
        )
        .gap(GAP)
        .width(MENU_WIDTH);

        if needs_scroll {
            (
                Box::new(
                    ScrollArea::new("tray-rows", rows)
                        .width(MENU_WIDTH)
                        .height(height),
                ) as Box<dyn Widget>,
                height,
            )
        } else {
            (
                Box::new(rows) as Box<dyn Widget>,
                owned.len() as f32 * ROW_HEIGHT + (owned.len() - 1) as f32 * GAP,
            )
        }
    };

    /*
     * the menu is an inset card against the sheet, so the rows do not
     * read as if they belong to the bar
     */
    let height = body_height + PAD;

    (
        Box::new(
            Rectangle::new()
                .width(MENU_WIDTH)
                .height(height)
                .fill(scheme.surface_container_highest)
                .radius(20.0)
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

// one background app: its real icon when the desktop knows one
fn window_row(scheme: &Scheme, window: &WindowEntry, icon_path: Option<&str>) -> Rectangle {
    let key = format!("tray:{}", window.id);

    let state = crate::motion::hover(&key);

    let owned = key.clone();

    let id = window.id;

    let glyph_box: Box<dyn Widget> = match icon_path {
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

    let name = String::from(window.display_name());

    let title = String::from(window.title.clone());

    Rectangle::new()
        .width(MENU_WIDTH - GAP)
        .height(ROW_HEIGHT)
        .fill(mix(
            scheme.surface_container_highest,
            scheme.on_surface,
            0.08 * state,
        ))
        .radius(14.0)
        .cursor(Pointer)
        .padding(Padding {
            top: 0.0,
            right: 10.0,
            bottom: 0.0,
            left: 10.0,
        })
        .on_hover(move |inside| crate::motion::set_hover(owned.clone(), inside))
        .on_click(move |button| match button {
            Button::Left => Tray::focus(id),
            Button::Right => Tray::open_context(id),
            _ => {}
        })
        .child(
            Row::new(vec![
                Box::new(Stack::new(vec![glyph_box])),
                Box::new(
                    Column::new(vec![
                        Box::new(label(
                            name,
                            12.5,
                            scheme.on_surface,
                            amane::Weight::Medium,
                            1,
                        )),
                        Box::new(label(
                            title,
                            11.0,
                            scheme.on_surface_variant,
                            amane::Weight::Regular,
                            1,
                        )),
                    ])
                    .gap(1.0),
                ),
            ])
            .gap(10.0)
            .align(Center)
            .width(Parent),
        )
}

// the context menu of one window: lift it, or close it
fn context_menu(
    scheme: &Scheme,
    windows: &[(&WindowEntry, Option<String>)],
    id: i64,
) -> (Box<dyn Widget>, f32) {
    let Some((window, _)) = windows.iter().find(|(window, _)| window.id == id) else {
        Tray::close_context();

        return (
            Box::new(
                Rectangle::new()
                    .width(MENU_WIDTH)
                    .height(EMPTY_HEIGHT)
                    .align_child(Center, Center)
                    .child(label(
                        "Janela não encontrada",
                        12.0,
                        scheme.on_surface_variant,
                        amane::Weight::Medium,
                        1,
                    )),
            ) as Box<dyn Widget>,
            EMPTY_HEIGHT,
        );
    };

    let title = String::from(window.title.clone());

    let rows: Vec<Box<dyn Widget>> = vec![
        Box::new(context_row(
            scheme,
            "abrir",
            "Abrir janela",
            OPEN_NEW,
            move || Tray::focus(id),
        )),
        Box::new(context_row(scheme, "fechar", "Fechar", CLOSE, move || {
            Tray::close_window(id)
        })),
    ];

    (
        Box::new(
            Column::new(vec![
                Box::new(
                    Row::new(vec![boxed(label(
                        title,
                        11.5,
                        scheme.on_surface_variant,
                        amane::Weight::Medium,
                        1,
                    ))])
                    .width(MENU_WIDTH - GAP)
                    .justify(SpaceBetween),
                ),
                Box::new(Column::new(rows).gap(4.0).width(MENU_WIDTH - GAP)),
            ])
            .gap(4.0),
        ) as Box<dyn Widget>,
        ROW_HEIGHT * 2.0 + 4.0 + 24.0,
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

fn boxed(widget: impl Widget + 'static) -> Box<dyn Widget> {
    Box::new(widget)
}
