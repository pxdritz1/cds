use std::collections::hash_map::DefaultHasher;
use std::fs::File;
use std::hash::{Hash, Hasher};
use std::path::Path;
use std::process;
use std::sync::Arc;
use std::thread;
use std::time::Duration;

use amane::{
    Apps, Argument, Bus, Button, Center, Image, Padding, Pointer, Rectangle, Row, Service, Stack,
    Value, Widget,
};

use crate::island::widgets::{APPS, icon, label};
use crate::theme::{Scheme, mix};

const WATCHER_PATH: &str = "/StatusNotifierWatcher";
const WATCHER_INTERFACE: &str = "org.freedesktop.StatusNotifierWatcher";
const KDE_WATCHER_INTERFACE: &str = "org.kde.StatusNotifierWatcher";
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
        Duration::from_secs(5)
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
        use zbus::fdo::{RequestNameFlags, RequestNameReply};

        let state = Arc::new(WatcherState::default());

        let builder = Builder::session()
            .and_then(|builder| {
                builder.serve_at(WATCHER_PATH, StatusNotifierWatcher(state.clone()))
            })
            .and_then(|builder| builder.serve_at(WATCHER_PATH, KdeStatusNotifierWatcher(state)));

        let connection = match builder.and_then(|builder| builder.build()) {
            Ok(connection) => connection,
            Err(error) => {
                eprintln!("CDS: não foi possível iniciar StatusNotifierWatcher: {error}");
                return;
            }
        };

        let request_name = |name: &str| match connection
            .request_name_with_flags(name, RequestNameFlags::DoNotQueue.into())
        {
            Ok(RequestNameReply::PrimaryOwner | RequestNameReply::AlreadyOwner) => true,
            Ok(_) => false,
            Err(error) => {
                eprintln!("CDS: falha ao reivindicar {name}: {error}");
                false
            }
        };

        let owns_freedesktop = request_name("org.freedesktop.StatusNotifierWatcher");

        let owns_kde = request_name("org.kde.StatusNotifierWatcher");

        if !owns_freedesktop && !owns_kde {
            eprintln!("CDS: outro StatusNotifierWatcher já está ativo no barramento");
            return;
        }

        eprintln!(
            "CDS: StatusNotifierWatcher ativo (freedesktop: {}, KDE: {})",
            owns_freedesktop, owns_kde,
        );

        loop {
            thread::sleep(Duration::from_secs(3600));
        }
    }
}

#[derive(Default)]
struct WatcherState {
    items: std::sync::Mutex<Vec<String>>,
    hosts: std::sync::Mutex<Vec<String>>,
}

#[derive(Clone)]
struct StatusNotifierWatcher(Arc<WatcherState>);

#[derive(Clone)]
struct KdeStatusNotifierWatcher(Arc<WatcherState>);

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

        let newly_registered = {
            let mut items = self
                .0
                .items
                .lock()
                .unwrap_or_else(|error| error.into_inner());

            if items.iter().any(|item| item == &address) {
                false
            } else {
                items.push(address.clone());
                items.sort();
                true
            }
        };

        if newly_registered {
            for interface in [WATCHER_INTERFACE, KDE_WATCHER_INTERFACE] {
                connection
                    .emit_signal(
                        None::<&str>,
                        WATCHER_PATH,
                        interface,
                        "StatusNotifierItemRegistered",
                        &(address.as_str(),),
                    )
                    .await
                    .map_err(|error| zbus::fdo::Error::Failed(error.to_string()))?;
            }
        }

        Ok(())
    }

    async fn register_status_notifier_host(
        &self,
        #[zbus(header)] header: zbus::message::Header<'_>,
        #[zbus(connection)] connection: &zbus::Connection,
        service: String,
    ) -> zbus::fdo::Result<()> {
        let host = if service.is_empty() {
            header
                .sender()
                .map(|sender| sender.to_string())
                .unwrap_or_default()
        } else {
            service
        };

        let newly_registered = {
            let mut hosts = self
                .0
                .hosts
                .lock()
                .unwrap_or_else(|error| error.into_inner());

            if hosts.iter().any(|item| item == &host) {
                false
            } else {
                hosts.push(host);
                true
            }
        };

        if newly_registered {
            for interface in [WATCHER_INTERFACE, KDE_WATCHER_INTERFACE] {
                connection
                    .emit_signal(
                        None::<&str>,
                        WATCHER_PATH,
                        interface,
                        "StatusNotifierHostRegistered",
                        &(),
                    )
                    .await
                    .map_err(|error| zbus::fdo::Error::Failed(error.to_string()))?;
            }
        }

        Ok(())
    }

    #[zbus(property)]
    fn registered_status_notifier_items(&self) -> Vec<String> {
        self.0
            .items
            .lock()
            .unwrap_or_else(|error| error.into_inner())
            .clone()
    }

    #[zbus(property)]
    fn is_status_notifier_host_registered(&self) -> bool {
        !self
            .0
            .hosts
            .lock()
            .unwrap_or_else(|error| error.into_inner())
            .is_empty()
    }

    #[zbus(property)]
    fn protocol_version(&self) -> i32 {
        0
    }
}

#[zbus::interface(name = "org.kde.StatusNotifierWatcher")]
impl KdeStatusNotifierWatcher {
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

        let newly_registered = {
            let mut items = self
                .0
                .items
                .lock()
                .unwrap_or_else(|error| error.into_inner());

            if items.iter().any(|item| item == &address) {
                false
            } else {
                items.push(address.clone());
                items.sort();
                true
            }
        };

        if newly_registered {
            for interface in [WATCHER_INTERFACE, KDE_WATCHER_INTERFACE] {
                connection
                    .emit_signal(
                        None::<&str>,
                        WATCHER_PATH,
                        interface,
                        "StatusNotifierItemRegistered",
                        &(address.as_str(),),
                    )
                    .await
                    .map_err(|error| zbus::fdo::Error::Failed(error.to_string()))?;
            }
        }

        Ok(())
    }

    async fn register_status_notifier_host(
        &self,
        #[zbus(header)] header: zbus::message::Header<'_>,
        #[zbus(connection)] connection: &zbus::Connection,
        service: String,
    ) -> zbus::fdo::Result<()> {
        let host = if service.is_empty() {
            header
                .sender()
                .map(|sender| sender.to_string())
                .unwrap_or_default()
        } else {
            service
        };

        let newly_registered = {
            let mut hosts = self
                .0
                .hosts
                .lock()
                .unwrap_or_else(|error| error.into_inner());

            if hosts.iter().any(|item| item == &host) {
                false
            } else {
                hosts.push(host);
                true
            }
        };

        if newly_registered {
            for interface in [WATCHER_INTERFACE, KDE_WATCHER_INTERFACE] {
                connection
                    .emit_signal(
                        None::<&str>,
                        WATCHER_PATH,
                        interface,
                        "StatusNotifierHostRegistered",
                        &(),
                    )
                    .await
                    .map_err(|error| zbus::fdo::Error::Failed(error.to_string()))?;
            }
        }

        Ok(())
    }

    #[zbus(property)]
    fn registered_status_notifier_items(&self) -> Vec<String> {
        self.0
            .items
            .lock()
            .unwrap_or_else(|error| error.into_inner())
            .clone()
    }

    #[zbus(property)]
    fn is_status_notifier_host_registered(&self) -> bool {
        !self
            .0
            .hosts
            .lock()
            .unwrap_or_else(|error| error.into_inner())
            .is_empty()
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

                if (host == &kde_host && owns_kde)
                    || (host == &freedesktop_host && owns_freedesktop)
                {
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

        let icon_path = resolve_icon(&icon_name, &id)
            .or_else(|| resolve_icon_pixmap(&bus, &service, &path, &interface));

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

fn item_property(bus: &Bus, service: &str, path: &str, interface: &str, name: &str) -> String {
    bus.property(service, path, interface, name)
        .text()
        .to_string()
}

fn resolve_icon_pixmap(bus: &Bus, service: &str, path: &str, interface: &str) -> Option<String> {
    let pixmaps = bus.property(service, path, interface, "IconPixmap");

    for pixmap in pixmaps.list() {
        let fields = pixmap.list();

        if fields.len() != 3 {
            continue;
        }

        let width = fields[0].number() as u32;
        let height = fields[1].number() as u32;

        if width == 0 || height == 0 || width > 4096 || height > 4096 {
            continue;
        }

        let expected = (width as usize)
            .checked_mul(height as usize)?
            .checked_mul(4)?;

        let bytes: Vec<u8> = fields[2]
            .list()
            .iter()
            .map(|value| value.number().clamp(0.0, 255.0) as u8)
            .collect();

        if bytes.len() != expected {
            continue;
        }

        // SNI IconPixmap uses ARGB32; PNG expects RGBA.
        let mut rgba = Vec::with_capacity(expected);

        for pixel in bytes.chunks_exact(4) {
            let [a, r, g, b] = [pixel[0], pixel[1], pixel[2], pixel[3]];
            rgba.extend_from_slice(&[r, g, b, a]);
        }

        let mut hasher = DefaultHasher::new();
        width.hash(&mut hasher);
        height.hash(&mut hasher);
        bytes.hash(&mut hasher);

        let output = std::env::temp_dir().join(format!("amane-sni-{:016x}.png", hasher.finish()));

        if !output.is_file() {
            let file = File::create(&output).ok()?;
            let mut encoder = png::Encoder::new(file, width, height);

            encoder.set_color(png::ColorType::Rgba);
            encoder.set_depth(png::BitDepth::Eight);

            let mut writer = encoder.write_header().ok()?;
            writer.write_image_data(&rgba).ok()?;
        }

        return Some(output.to_string_lossy().into_owned());
    }

    None
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

const ICON_SIZE: f32 = 42.0;
const ICON_GAP: f32 = 8.0;
const TRAY_PAD: f32 = 10.0;
const EMPTY_WIDTH: f32 = 220.0;
const EMPTY_HEIGHT: f32 = 46.0;

/*
 * SNI items are presented as a compact row of icons. The surface grows with
 * the number of registered items instead of reserving a fixed-width list.
 */
pub fn menu(scheme: &Scheme) -> (Box<dyn Widget>, f32, f32) {
    let items = Tray::read().items().to_vec();

    let (body, width, body_height) = if items.is_empty() {
        (
            Box::new(
                Rectangle::new()
                    .width(EMPTY_WIDTH)
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
            EMPTY_WIDTH,
            EMPTY_HEIGHT,
        )
    } else {
        let icons = items
            .iter()
            .map(|item| boxed(item_icon(scheme, item)))
            .collect();

        let width = items.len() as f32 * ICON_SIZE
            + items.len().saturating_sub(1) as f32 * ICON_GAP;

        (
            Box::new(
                Row::new(icons)
                    .gap(ICON_GAP)
                    .align(Center)
                    .width(width)
                    .height(ICON_SIZE),
            ) as Box<dyn Widget>,
            width,
            ICON_SIZE,
        )
    };

    let width = width + TRAY_PAD * 2.0;
    let height = body_height + TRAY_PAD * 2.0;

    (
        Box::new(
            Rectangle::new()
                .width(width)
                .height(height)
                .fill(scheme.surface_container_highest)
                .radius(20.0)
                .clip()
                .padding(Padding {
                    top: TRAY_PAD,
                    right: TRAY_PAD,
                    bottom: TRAY_PAD,
                    left: TRAY_PAD,
                })
                .align_child(Center, Center)
                .child(Stack::new(vec![body])),
        ) as Box<dyn Widget>,
        width,
        height,
    )
}

fn item_icon(scheme: &Scheme, item: &TrayItem) -> Rectangle {
    let key = format!("tray-sni:{}:{}", item.service, item.id);
    let state = crate::motion::hover(&key);
    let owned = key;

    let icon_widget: Box<dyn Widget> = match item.icon_path.as_deref() {
        Some(path) if Image::loaded(path) => Box::new(
            Rectangle::new()
                .width(26.0)
                .height(26.0)
                .radius(7.0)
                .clip()
                .fill(Image::cover(path).thumbnail(64, 64)),
        ),
        _ => Box::new(icon(APPS, 24.0, scheme.on_surface)),
    };

    let item = item.clone();

    Rectangle::new()
        .width(ICON_SIZE)
        .height(ICON_SIZE)
        .fill(mix(
            scheme.surface_container_highest,
            scheme.on_surface,
            0.10 * state,
        ))
        .radius(13.0)
        .cursor(Pointer)
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
        .align_child(Center, Center)
        .child(Stack::new(vec![icon_widget]))
}

fn boxed(widget: impl Widget + 'static) -> Box<dyn Widget> {
    Box::new(widget)
}
