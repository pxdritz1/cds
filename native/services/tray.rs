use std::sync::{Arc, Mutex, OnceLock};
use std::time::Duration;

use anyhow::Result;

const WATCHER_PATH: &str = "/StatusNotifierWatcher";
const FREEDESKTOP_WATCHER: &str = "org.freedesktop.StatusNotifierWatcher";
const KDE_WATCHER: &str = "org.kde.StatusNotifierWatcher";

#[derive(Default)]
struct WatcherState {
    items: Mutex<Vec<String>>,
    hosts: Mutex<Vec<String>>,
}

static STATE: OnceLock<Arc<WatcherState>> = OnceLock::new();

fn state() -> Arc<WatcherState> {
    STATE.get_or_init(|| Arc::new(WatcherState::default())).clone()
}

#[derive(Clone)]
struct StatusNotifierWatcher(Arc<WatcherState>);
#[derive(Clone)]
struct KdeStatusNotifierWatcher(Arc<WatcherState>);

fn item_address(header: &zbus::message::Header<'_>, service: String) -> zbus::fdo::Result<String> {
    if service.starts_with('/') {
        let sender = header.sender()
            .ok_or_else(|| zbus::fdo::Error::Failed("StatusNotifierItem registration has no sender".into()))?;
        Ok(format!("{sender}{service}"))
    } else if service.contains('/') {
        Ok(service)
    } else {
        Ok(format!("{service}/StatusNotifierItem"))
    }
}

fn host_address(header: &zbus::message::Header<'_>, service: String) -> String {
    if service.is_empty() {
        header.sender().map(|sender| sender.to_string()).unwrap_or_default()
    } else {
        service
    }
}

async fn register_item(
    state: &WatcherState,
    header: &zbus::message::Header<'_>,
    connection: &zbus::Connection,
    service: String,
) -> zbus::fdo::Result<()> {
    let address = item_address(header, service)?;
    let inserted = {
        let mut items = state.items.lock().unwrap_or_else(|error| error.into_inner());
        if items.contains(&address) { false } else {
            items.push(address.clone());
            items.sort();
            true
        }
    };
    if inserted {
        for interface in [FREEDESKTOP_WATCHER, KDE_WATCHER] {
            connection.emit_signal(
                None::<&str>, WATCHER_PATH, interface, "StatusNotifierItemRegistered",
                &(address.as_str(),),
            ).await.map_err(|error| zbus::fdo::Error::Failed(error.to_string()))?;
        }
    }
    Ok(())
}

async fn register_host(
    state: &WatcherState,
    header: &zbus::message::Header<'_>,
    connection: &zbus::Connection,
    service: String,
) -> zbus::fdo::Result<()> {
    let host = host_address(header, service);
    let inserted = {
        let mut hosts = state.hosts.lock().unwrap_or_else(|error| error.into_inner());
        if hosts.contains(&host) { false } else { hosts.push(host); true }
    };
    if inserted {
        for interface in [FREEDESKTOP_WATCHER, KDE_WATCHER] {
            connection.emit_signal(
                None::<&str>, WATCHER_PATH, interface, "StatusNotifierHostRegistered", &(),
            ).await.map_err(|error| zbus::fdo::Error::Failed(error.to_string()))?;
        }
    }
    Ok(())
}

#[zbus::interface(name = "org.freedesktop.StatusNotifierWatcher")]
impl StatusNotifierWatcher {
    async fn register_status_notifier_item(
        &self,
        #[zbus(header)] header: zbus::message::Header<'_>,
        #[zbus(connection)] connection: &zbus::Connection,
        service: String,
    ) -> zbus::fdo::Result<()> {
        register_item(&self.0, &header, connection, service).await
    }

    async fn register_status_notifier_host(
        &self,
        #[zbus(header)] header: zbus::message::Header<'_>,
        #[zbus(connection)] connection: &zbus::Connection,
        service: String,
    ) -> zbus::fdo::Result<()> {
        register_host(&self.0, &header, connection, service).await
    }

    #[zbus(property)]
    fn registered_status_notifier_items(&self) -> Vec<String> {
        self.0.items.lock().unwrap_or_else(|error| error.into_inner()).clone()
    }

    #[zbus(property)]
    fn is_status_notifier_host_registered(&self) -> bool {
        !self.0.hosts.lock().unwrap_or_else(|error| error.into_inner()).is_empty()
    }

    #[zbus(property)]
    fn protocol_version(&self) -> i32 { 0 }
}

#[zbus::interface(name = "org.kde.StatusNotifierWatcher")]
impl KdeStatusNotifierWatcher {
    async fn register_status_notifier_item(
        &self,
        #[zbus(header)] header: zbus::message::Header<'_>,
        #[zbus(connection)] connection: &zbus::Connection,
        service: String,
    ) -> zbus::fdo::Result<()> {
        register_item(&self.0, &header, connection, service).await
    }

    async fn register_status_notifier_host(
        &self,
        #[zbus(header)] header: zbus::message::Header<'_>,
        #[zbus(connection)] connection: &zbus::Connection,
        service: String,
    ) -> zbus::fdo::Result<()> {
        register_host(&self.0, &header, connection, service).await
    }

    #[zbus(property)]
    fn registered_status_notifier_items(&self) -> Vec<String> {
        self.0.items.lock().unwrap_or_else(|error| error.into_inner()).clone()
    }

    #[zbus(property)]
    fn is_status_notifier_host_registered(&self) -> bool {
        !self.0.hosts.lock().unwrap_or_else(|error| error.into_inner()).is_empty()
    }

    #[zbus(property)]
    fn protocol_version(&self) -> i32 { 0 }
}

pub fn registered_items() -> Vec<String> {
    state().items.lock().unwrap_or_else(|error| error.into_inner()).clone()
}

#[derive(Debug, Clone, Default)]
pub struct TrayItemSnapshot {
    pub address: String,
    pub title: String,
    pub icon_name: String,
    pub status: String,
    pub menu_path: String,
}

pub async fn snapshot() -> anyhow::Result<Vec<TrayItemSnapshot>> {
    let connection = zbus::Connection::session().await?;
    let addresses = registered_items();
    let mut result = Vec::with_capacity(addresses.len());
    for address in addresses {
        let Some((service, path)) = address.split_once('/') else { continue };
        let path = format!("/{path}");
        let proxy = zbus::Proxy::new(
            &connection, service, path.as_str(), "org.freedesktop.StatusNotifierItem",
        ).await?;
        let title = proxy.get_property::<String>("Title").await.unwrap_or_default();
        let icon_name = proxy.get_property::<String>("IconName").await.unwrap_or_default();
        let status = proxy.get_property::<String>("Status").await.unwrap_or_default();
        let menu_path = proxy.get_property::<zbus::zvariant::OwnedObjectPath>("Menu")
            .await.map(|value| value.to_string()).unwrap_or_default();
        result.push(TrayItemSnapshot { address, title, icon_name, status, menu_path });
    }
    Ok(result)
}

pub async fn context_menu(address: &str) -> anyhow::Result<()> {
    let (service, path) = address.split_once('/')
        .ok_or_else(|| anyhow::anyhow!("invalid StatusNotifier address"))?;
    let path = format!("/{path}");
    let connection = zbus::Connection::session().await?;
    let proxy = zbus::Proxy::new(
        &connection, service, path.as_str(), "org.freedesktop.StatusNotifierItem",
    ).await?;
    proxy.call::<_, _, ()>("ContextMenu", &(0_i32, 0_i32)).await?;
    Ok(())
}

pub async fn activate(address: &str) -> anyhow::Result<()> {
    let (service, path) = address.split_once('/')
        .ok_or_else(|| anyhow::anyhow!("invalid StatusNotifier address"))?;
    let path = format!("/{path}");
    let connection = zbus::Connection::session().await?;
    let proxy = zbus::Proxy::new(
        &connection, service, path.as_str(), "org.freedesktop.StatusNotifierItem",
    ).await?;
    proxy.call::<_, _, ()>("Activate", &(0_i32, 0_i32)).await?;
    Ok(())
}

pub fn start_server() -> Result<()> {
    let watcher_state = state();
    std::thread::Builder::new().name("cds-status-notifier-watcher".into()).spawn(move || {
        let result = zbus::block_on(async move {
            let connection = zbus::connection::Builder::session()?
                .serve_at(WATCHER_PATH, StatusNotifierWatcher(watcher_state.clone()))?
                .serve_at(WATCHER_PATH, KdeStatusNotifierWatcher(watcher_state))?
                .build().await?;
            let owns_freedesktop = connection.request_name(FREEDESKTOP_WATCHER).await.is_ok();
            let owns_kde = connection.request_name(KDE_WATCHER).await.is_ok();
            if !owns_freedesktop && !owns_kde {
                tracing::warn!("Another StatusNotifierWatcher already owns both bus names");
                return Ok::<(), zbus::Error>(());
            }
            futures_util::future::pending::<()>().await;
            #[allow(unreachable_code)]
            Ok::<(), zbus::Error>(())
        });
        if let Err(error) = result {
            tracing::error!(%error, "Could not start StatusNotifierWatcher");
        }
    })?;

    std::thread::Builder::new().name("cds-status-notifier-host".into()).spawn(|| {
        let result = zbus::block_on(async {
            let connection = zbus::Connection::session().await?;
            let host = format!("org.freedesktop.StatusNotifierHost-{}", std::process::id());
            connection.request_name(host.as_str()).await?;
            loop {
                let watcher = zbus::Proxy::new(
                    &connection, FREEDESKTOP_WATCHER, WATCHER_PATH, FREEDESKTOP_WATCHER,
                ).await;
                if let Ok(watcher) = watcher {
                    let registered = watcher.call::<_, _, ()>(
                        "RegisterStatusNotifierHost", &(host.as_str(),),
                    ).await;
                    if registered.is_ok() { break; }
                }
                tokio::time::sleep(Duration::from_secs(2)).await;
            }
            futures_util::future::pending::<()>().await;
            #[allow(unreachable_code)]
            Ok::<(), zbus::Error>(())
        });
        if let Err(error) = result {
            tracing::error!(%error, "Could not register CDS as a StatusNotifier host");
        }
    })?;
    Ok(())
}
