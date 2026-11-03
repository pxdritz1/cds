use std::path::Path;
use std::process;
use std::thread;
use std::time::Duration;

use amane::{
    Apps, Argument, Bus, Button, Center, Column, Image, Padding, Parent, Pointer, Rectangle, Row,
    ScrollArea, Service, SpaceBetween, Stack, Value, Widget,
};

use crate::island::widgets::{APPS, icon, label};
use crate::theme::{Scheme, mix};

const WATCHER_PATH: &str = "/StatusNotifierWatcher";
const WATCHER_INTERFACE: &str = "org.freedesktop.StatusNotifierWatcher";
const ITEM_INTERFACE: &str = "org.freedesktop.StatusNotifierItem";

#[derive(Clone, PartialEq)]
pub struct TrayItem {
    service: String,
    path: String,
    id: String,
    title: String,
    status: String,
    icon_path: Option<String>,
}

impl TrayItem {
    fn activate(&self) {
        self.call_item("Activate");
    }

    fn context_menu(&self) {
        self.call_item("ContextMenu");
    }

    fn call_item(&self, method: &str) {
        // SNI coordinates are hints. Amane's click callback currently exposes
        // the button but not the global pointer position, so use the origin.
        let bus = Bus::session();
        let arguments = [Argument::from(0_i32), Argument::from(0_i32)];

        bus.call(&self.service, &self.path, ITEM_INTERFACE, method, &arguments);
    }

    fn display_name(&self) -> &str {
        if self.title.is_empty() {
            &self.id
        } else {
            &self.title
        }
    }
}

#[derive(Default)]
pub struct Tray {
    open: bool,
    items: Vec<TrayItem>,
}

impl Service for Tray {
    fn new() -> Self {
        Self::default()
    }

    fn interval() -> Duration {
        Duration::from_millis(2500)
    }

    fn update(&mut self) -> bool {
        let Some(items) = scan_items() else {
            // A missing watcher or a temporary D-Bus failure must not erase
            // the last valid tray contents.
            return false;
        };

        if self.items == items {
            return false;
        }

        self.items = items;
        true
    }

    fn listen() {
        let bus = Bus::session();
        let host = format!("org.freedesktop.StatusNotifierHost-{}", process::id());

        if bus.own(&host) {
            if let Some(watcher) = watcher_destination(&bus) {
                bus.call(
                    watcher,
                    WATCHER_PATH,
                    WATCHER_INTERFACE,
                    "RegisterStatusNotifierHost",
                    &[Argument::from(host)],
                );
            }
        }

        // Polling is intentional here: it covers item registration, property
        // changes and watcher restarts without blocking the drawing thread.
        loop {
            thread::sleep(Self::interval());

            let mut tray = Self::write();

            if !tray.update() {
                tray.quiet();
            }
        }
    }
}

impl Tray {
    pub fn open(&self) -> bool {
        self.open
    }

    pub fn items(&self) -> &[TrayItem] {
        &self.items
    }

    pub fn toggle() {
        let open = {
            let mut tray = Self::write();

            tray.open = !tray.open;

            tray.open
        };

        // The tray and an overlay would both want the pointer.
        if open {
            crate::overlay::Overlay::close();
        }
    }

    pub fn close() {
        Self::write().open = false;
    }
}

pub fn command(arguments: &[String]) -> String {
    let [action, ..] = arguments else {
        return usage();
    };

    match action.as_str() {
        "toggle" => Tray::toggle(),
        "close" => Tray::close(),
        "list" => {
            let rows: Vec<String> = Tray::read()
                .items()
                .iter()
                .map(|item| format!("{} [{}]", item.display_name(), item.status))
                .collect();

            return if rows.is_empty() {
                String::from("nenhum StatusNotifierItem registrado")
            } else {
                rows.join("\n")
            };
        }
        _ => return usage(),
    }

    String::from("ok")
}

fn usage() -> String {
    String::from("usage: tray toggle | close | list")
}

fn watcher_destination(bus: &Bus) -> Option<&'static str> {
    let freedesktop = bus.property(
        "org.freedesktop.StatusNotifierWatcher",
        WATCHER_PATH,
        WATCHER_INTERFACE,
        "RegisteredStatusNotifierItems",
    );

    if !matches!(freedesktop, Value::Nothing) {
        return Some("org.freedesktop.StatusNotifierWatcher");
    }

    let kde = bus.property(
        "org.kde.StatusNotifierWatcher",
        WATCHER_PATH,
        WATCHER_INTERFACE,
        "RegisteredStatusNotifierItems",
    );

    if !matches!(kde, Value::Nothing) {
        return Some("org.kde.StatusNotifierWatcher");
    }

    None
}

fn scan_items() -> Option<Vec<TrayItem>> {
    let bus = Bus::session();
    let watcher = watcher_destination(&bus)?;

    let registered = bus.property(
        watcher,
        WATCHER_PATH,
        WATCHER_INTERFACE,
        "RegisteredStatusNotifierItems",
    );

    if matches!(registered, Value::Nothing) {
        return None;
    }

    let mut items = Vec::new();

    for registered_item in registered.list() {
        let address = registered_item.text();

        if address.is_empty() {
            continue;
        }

        let (service, path) = item_address(address);

        let status = item_property(&bus, &service, &path, "Status");
        if status == "Passive" {
            continue;
        }

        let id = item_property(&bus, &service, &path, "Id");
        let title = item_property(&bus, &service, &path, "Title");

        let normal_icon = item_property(&bus, &service, &path, "IconName");
        let attention_icon = if status == "NeedsAttention" {
            item_property(&bus, &service, &path, "AttentionIconName")
        } else {
            String::new()
        };

        let icon_name = if attention_icon.is_empty() {
            normal_icon
        } else {
            attention_icon
        };

        let icon_path = resolve_icon(&icon_name, &id);

        items.push(TrayItem {
            service,
            path,
            id,
            title,
            status,
            icon_path,
        });
    }

    items.sort_by(|a, b| {
        a.display_name()
            .to_lowercase()
            .cmp(&b.display_name().to_lowercase())
    });

    Some(items)
}

fn item_address(address: &str) -> (String, String) {
    match address.split_once('/') {
        Some((service, path)) => (service.to_string(), format!("/{path}")),
        None => (address.to_string(), "/StatusNotifierItem".to_string()),
    }
}

fn item_property(bus: &Bus, service: &str, path: &str, name: &str) -> String {
    bus.property(service, path, ITEM_INTERFACE, name)
        .text()
        .to_string()
}

fn resolve_icon(icon_name: &str, id: &str) -> Option<String> {
    if icon_name.is_empty() {
        return None;
    }

    let path = Path::new(icon_name);

    // The protocol allows either a themed icon name or an absolute path.
    if path.is_absolute() && path.is_file() {
        return Some(icon_name.to_string());
    }

    let needle = icon_name.to_lowercase();
    let app_id = id.to_lowercase();

    let apps = Apps::read();

    let app = apps.list().iter().find(|app| {
        let name = app.name().to_lowercase();
        let icon = app.icon().unwrap_or("").to_lowercase();

        icon == needle
            || name == needle
            || name == app_id
            || (!app_id.is_empty() && name.contains(app_id.as_str()))
    })?;

    app.icon_path()
        .map(|path| path.to_string_lossy().into_owned())
}

const ROW_HEIGHT: f32 = 46.0;
const PAD: f32 = 10.0;
const GAP: f32 = 6.0;
const EMPTY_HEIGHT: f32 = 46.0;
const MENU_WIDTH: f32 = 300.0;
const SCROLL_ROWS: f32 = 5.0;

/*
 * SNI items are listed in the expandable tray. Left click activates the
 * application; right click asks the item to display its own context menu.
 */
pub fn menu(scheme: &Scheme) -> (Box<dyn Widget>, f32, f32) {
    let items = Tray::read().items().to_vec();

    let (body, body_height) = if items.is_empty() {
        (
            Box::new(
                Rectangle::new()
                    .width(MENU_WIDTH)
                    .height(EMPTY_HEIGHT)
                    .align_child(Center, Center)
                    .child(label(
                        "Nenhum indicador de bandeja registrado",
                        11.5,
                        scheme.on_surface_variant,
                        amane::Weight::Medium,
                        1,
                    )),
            ) as Box<dyn Widget>,
            EMPTY_HEIGHT,
        )
    } else {
        let height = SCROLL_ROWS * ROW_HEIGHT + (SCROLL_ROWS - 1.0) * GAP;
        let needs_scroll = items.len() as f32 > SCROLL_ROWS;

        let rows = Column::new(
            items
                .iter()
                .map(|item| boxed(item_row(scheme, item)))
                .collect(),
        )
        .gap(GAP)
        .width(MENU_WIDTH);

        if needs_scroll {
            (
                Box::new(
                    ScrollArea::new("sni-tray-items", rows)
                        .width(MENU_WIDTH)
                        .height(height),
                ) as Box<dyn Widget>,
                height,
            )
        } else {
            (
                Box::new(rows) as Box<dyn Widget>,
                items.len() as f32 * ROW_HEIGHT + (items.len() - 1) as f32 * GAP,
            )
        }
    };

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
                .align_child(Center, Center)
                .child(Stack::new(vec![body])),
        ) as Box<dyn Widget>,
        MENU_WIDTH,
        height,
    )
}

fn item_row(scheme: &Scheme, item: &TrayItem) -> Rectangle {
    let key = format!("tray-sni:{}:{}", item.service, item.id);
    let state = crate::motion::hover(&key);
    let owned = key;

    let icon_widget: Box<dyn Widget> = match item.icon_path.as_deref() {
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

    let item = item.clone();
    let title = item.display_name().to_string();
    let status = item.status.clone();

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
            Button::Left => {
                Tray::close();
                item.activate();
            }
            Button::Right => {
                Tray::close();
                item.context_menu();
            }
            _ => {}
        })
        .child(
            Row::new(vec![
                Box::new(Stack::new(vec![icon_widget])),
                Box::new(
                    Column::new(vec![
                        Box::new(label(
                            title,
                            12.5,
                            scheme.on_surface,
                            amane::Weight::Medium,
                            1,
                        )),
                        Box::new(label(
                            status,
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

fn boxed(widget: impl Widget + 'static) -> Box<dyn Widget> {
    Box::new(widget)
}
