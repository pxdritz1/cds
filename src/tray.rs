use std::path::Path;
use std::process;
use std::thread;
use std::time::Duration;

use amane::{
    Apps, Argument, Bus, Button, Center, Column, Image, Padding, Parent, Pointer, Rectangle, Row,
    ScrollArea, Service, Stack, Value, Widget,
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
    interface: String,
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
        // D-Bus calls can wait for another process; never block the draw thread.
        let service = self.service.clone();
        let path = self.path.clone();
        let interface = self.interface.clone();
        let method = method.to_string();

        thread::spawn(move || {
            let bus = Bus::session();
            let arguments = [Argument::from(0_i32), Argument::from(0_i32)];

            bus.call(&service, &path, &interface, &method, &arguments);
        });
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
        // Starting this tiny service registers the shell as an SNI host on
        // its own service thread; no D-Bus call is made by the drawing thread.
        // Start the in-process watcher before registering this shell as a host.
        let _watcher = TrayWatcher::read();
        let _host = TrayHost::read();

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
}


/*
 * The shell provides its own StatusNotifierWatcher. This is intentionally
 * backed by a separate zbus connection: Amane's Bus wrapper does not expose
 * the caller's unique name, which is required when clients register an
 * object path instead of a well-known service name.
 */
struct TrayWatcher;

impl Service for TrayWatcher {
    fn new() -> Self {
        Self
    }

    fn listen() {
        use zbus::blocking::connection::Builder;

        let watcher = StatusNotifierWatcher::default();
        let connection = match Builder::session()
            .and_then(|builder| builder.name("org.freedesktop.StatusNotifierWatcher"))
            .and_then(|builder| builder.serve_at(WATCHER_PATH, watcher))
            .and_then(|builder| builder.build())
        {
            Ok(connection) => connection,
            Err(error) => {
                eprintln!("CDS: não foi possível iniciar StatusNotifierWatcher: {error}");
                return;
            }
        };

        // Keep the KDE compatibility name on the same connection. A single
        // watcher instance then serves both the Freedesktop and KDE names.
        if let Err(error) = connection.request_name("org.kde.StatusNotifierWatcher") {
            eprintln!("CDS: não foi possível registrar alias KDE do watcher: {error}");
        }

        loop {
            thread::sleep(Duration::from_secs(3600));
        }
    }
}

#[derive(Default)]
struct StatusNotifierWatcher {
    items: std::sync::Mutex<Vec<String>>,
    hosts: std::sync::Mutex<Vec<String>>,
}

#[zbus::interface(name = "org.freedesktop.StatusNotifierWatcher")]
impl StatusNotifierWatcher {
    async fn register_status_notifier_item(
        &self,
        #[zbus(header)] header: zbus::message::Header<'_>,
        #[zbus(connection)] connection: &zbus::Connection,
        service: String,
    ) -> zbus::fdo::Result<()> {
        let address = if service.starts_with('/') {
            let sender = header
                .sender()
                .ok_or_else(|| zbus::fdo::Error::Failed("chamador D-Bus sem sender".into()))?;
            format!("{sender}{service}")
        } else if service.contains('/') {
            service
        } else {
            format!("{service}/StatusNotifierItem")
        };

        let mut items = self.items.lock().unwrap_or_else(|error| error.into_inner());
        if !items.iter().any(|item| item == &address) {
            items.push(address.clone());
            items.sort();
            connection
                .emit_signal(
                    None::<&str>,
                    WATCHER_PATH,
                    WATCHER_INTERFACE,
                    "StatusNotifierItemRegistered",
                    &(address.as_str(),),
                )
                .await
                .map_err(|error| zbus::fdo::Error::Failed(error.to_string()))?;
        }

        Ok(())
    }

    async fn register_status_notifier_host(
        &self,
        #[zbus(header)] header: zbus::message::Header<'_>,
        #[zbus(connection)] connection: &zbus::blocking::Connection,
        service: String,
    ) -> zbus::fdo::Result<()> {
        let host = if service.is_empty() {
            header.sender().map(|sender| sender.to_string()).unwrap_or_default()
        } else {
            service
        };

        let mut hosts = self.hosts.lock().unwrap_or_else(|error| error.into_inner());
        if !hosts.iter().any(|entry| entry == &host) {
            hosts.push(host);
            connection
                .emit_signal(
                    None::<&str>,
                    WATCHER_PATH,
                    WATCHER_INTERFACE,
                    "StatusNotifierHostRegistered",
                    &(),
                )
                .await
                .map_err(|error| zbus::fdo::Error::Failed(error.to_string()))?;
        }

        Ok(())
    }

    #[zbus(property)]
    fn registered_status_notifier_items(&self) -> Vec<String> {
        self.items.lock().unwrap_or_else(|error| error.into_inner()).clone()
    }

    #[zbus(property)]
    fn is_status_notifier_host_registered(&self) -> bool {
        !self.hosts.lock().unwrap_or_else(|error| error.into_inner()).is_empty()
    }

    #[zbus(property)]
    fn protocol_version(&self) -> i32 {
        0
    }

}

struct TrayHost;

impl Service for TrayHost {
    fn new() -> Self {
        Self
    }

    fn listen() {
        let bus = Bus::session();
        let pid = process::id();
        let freedesktop_host = format!("org.freedesktop.StatusNotifierHost-{pid}");
        let kde_host = format!("org.kde.StatusNotifierHost-{pid}");

        let owns_freedesktop = bus.own(&freedesktop_host);
        let owns_kde = bus.own(&kde_host);

        if !owns_freedesktop && !owns_kde {
            return;
        }

        // A watcher may start after the shell. Keep checking until it appears.
        loop {
            if let Some((destination, interface)) = watcher_destination(&bus) {
                let host = if destination == "org.kde.StatusNotifierWatcher" {
                    &kde_host
                } else {
                    &freedesktop_host
                };

                if (host == &kde_host && owns_kde) || (host == &freedesktop_host && owns_freedesktop) {
                    bus.call(
                        destination,
                        WATCHER_PATH,
                        interface,
                        "RegisterStatusNotifierHost",
                        &[Argument::from(host.clone())],
                    );
                }

                break;
            }

            thread::sleep(Duration::from_secs(2));
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

fn watcher_destination(bus: &Bus) -> Option<(&'static str, &'static str)> {
    let freedesktop = bus.property(
        "org.freedesktop.StatusNotifierWatcher",
        WATCHER_PATH,
        WATCHER_INTERFACE,
        "RegisteredStatusNotifierItems",
    );

    if !matches!(freedesktop, Value::Nothing) {
        return Some((
            "org.freedesktop.StatusNotifierWatcher",
            "org.freedesktop.StatusNotifierWatcher",
        ));
    }

    // KDE's long-standing implementation uses its own interface name.
    let kde = bus.property(
        "org.kde.StatusNotifierWatcher",
        WATCHER_PATH,
        "org.kde.StatusNotifierWatcher",
        "RegisteredStatusNotifierItems",
    );

    if !matches!(kde, Value::Nothing) {
        return Some((
            "org.kde.StatusNotifierWatcher",
            "org.kde.StatusNotifierWatcher",
        ));
    }

    None
}

fn scan_items() -> Option<Vec<TrayItem>> {
    let bus = Bus::session();
    let (watcher, interface) = watcher_destination(&bus)?;

    let registered = bus.property(
        watcher,
        WATCHER_PATH,
        interface,
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

        let interface = item_interface(&bus, &service, &path);

        let status = item_property(&bus, &service, &path, &interface, "Status");
        if status == "Passive" {
            continue;
        }

        let id = item_property(&bus, &service, &path, &interface, "Id");
        let title = item_property(&bus, &service, &path, &interface, "Title");

        let normal_icon = item_property(&bus, &service, &path, &interface, "IconName");
        let attention_icon = if status == "NeedsAttention" {
            item_property(&bus, &service, &path, &interface, "AttentionIconName")
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
            interface,
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

fn item_interface(bus: &Bus, service: &str, path: &str) -> String {
    let modern = bus.property(service, path, ITEM_INTERFACE, "Id");

    if !matches!(modern, Value::Nothing) {
        return ITEM_INTERFACE.to_string();
    }

    let kde = bus.property(service, path, "org.kde.StatusNotifierItem", "Id");

    if !matches!(kde, Value::Nothing) {
        return "org.kde.StatusNotifierItem".to_string();
    }

    ITEM_INTERFACE.to_string()
}

fn item_property(
    bus: &Bus,
    service: &str,
    path: &str,
    interface: &str,
    name: &str,
) -> String {
    bus.property(service, path, interface, name)
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
