use std::sync::{Arc, Mutex, OnceLock};

use anyhow::Result;

static ITEMS: OnceLock<Arc<Mutex<Vec<String>>>> = OnceLock::new();
static HOSTS: OnceLock<Arc<Mutex<Vec<String>>>> = OnceLock::new();

fn items() -> Arc<Mutex<Vec<String>>> {
    ITEMS.get_or_init(|| Arc::new(Mutex::new(Vec::new()))).clone()
}
fn hosts() -> Arc<Mutex<Vec<String>>> {
    HOSTS.get_or_init(|| Arc::new(Mutex::new(Vec::new()))).clone()
}

#[derive(Clone)]
struct StatusNotifierWatcher;

#[zbus::interface(name = "org.freedesktop.StatusNotifierWatcher")]
impl StatusNotifierWatcher {
    fn register_status_notifier_item(
        &self,
        #[zbus(header)] header: zbus::message::Header<'_>,
        service: String,
    ) -> zbus::fdo::Result<()> {
        let address = if service.starts_with('/') {
            let sender = header.sender().ok_or_else(|| zbus::fdo::Error::Failed("missing D-Bus sender".into()))?;
            format!("{sender}{service}")
        } else if service.contains('/') {
            service
        } else {
            format!("{service}/StatusNotifierItem")
        };
        let mut items = items().lock().unwrap_or_else(|error| error.into_inner());
        if !items.contains(&address) { items.push(address); }
        Ok(())
    }

    fn register_status_notifier_host(&self, service: String) {
        let mut hosts = hosts().lock().unwrap_or_else(|error| error.into_inner());
        if !hosts.contains(&service) { hosts.push(service); }
    }

    #[zbus(property)]
    fn registered_status_notifier_items(&self) -> Vec<String> {
        items().lock().unwrap_or_else(|error| error.into_inner()).clone()
    }

    #[zbus(property)]
    fn is_status_notifier_host_registered(&self) -> bool { true }

    #[zbus(property)]
    fn protocol_version(&self) -> i32 { 0 }
}

pub fn registered_items() -> Vec<String> {
    items().lock().unwrap_or_else(|error| error.into_inner()).clone()
}

pub fn start_server() -> Result<()> {
    let _ = items();
    let _ = hosts();
    std::thread::Builder::new().name("cds-status-notifier-watcher".into()).spawn(|| {
        let result = zbus::block_on(async {
            let _connection = zbus::connection::Builder::session()?
                .name("org.freedesktop.StatusNotifierWatcher")?
                .serve_at("/StatusNotifierWatcher", StatusNotifierWatcher)?
                .build()
                .await?;
            futures_util::future::pending::<()>().await;
            #[allow(unreachable_code)]
            Ok::<(), zbus::Error>(())
        });
        if let Err(error) = result {
            tracing::error!(%error, "Could not own StatusNotifierWatcher on the session bus");
        }
    })?;
    Ok(())
}
