use std::sync::{Arc, Mutex, OnceLock, atomic::{AtomicU32, Ordering}};
use std::time::{SystemTime, UNIX_EPOCH};

use anyhow::Result;
use serde::{Deserialize, Serialize};
use zbus::zvariant::OwnedValue;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NotificationRecord {
    pub id: u32,
    pub app_name: String,
    pub app_icon: String,
    pub summary: String,
    pub body: String,
    pub received_unix_ms: u128,
    pub expire_timeout_ms: i32,
}

static NEXT_ID: AtomicU32 = AtomicU32::new(1);
static NOTIFICATIONS: OnceLock<Arc<Mutex<Vec<NotificationRecord>>>> = OnceLock::new();

fn state() -> Arc<Mutex<Vec<NotificationRecord>>> {
    NOTIFICATIONS.get_or_init(|| Arc::new(Mutex::new(Vec::new()))).clone()
}

pub fn snapshot() -> Vec<NotificationRecord> {
    let now = now_ms();
    let mut notifications = state().lock().unwrap_or_else(|error| error.into_inner());
    notifications.retain(|notification| {
        notification.expire_timeout_ms <= 0
            || now.saturating_sub(notification.received_unix_ms) < notification.expire_timeout_ms as u128
    });
    notifications.clone()
}

pub async fn dismiss_remote(id: u32) -> anyhow::Result<()> {
    let connection = zbus::Connection::session().await?;
    let proxy = zbus::Proxy::new(
        &connection,
        "org.freedesktop.Notifications",
        "/org/freedesktop/Notifications",
        "org.freedesktop.Notifications",
    ).await?;
    proxy.call::<_, _, ()>("CloseNotification", &(id,)).await?;
    Ok(())
}

pub fn dismiss(id: u32) {
    state().lock().unwrap_or_else(|error| error.into_inner()).retain(|item| item.id != id);
}

fn now_ms() -> u128 {
    SystemTime::now().duration_since(UNIX_EPOCH).unwrap_or_default().as_millis()
}

#[derive(Clone)]
struct NotificationServer;

#[zbus::interface(name = "org.freedesktop.Notifications")]
impl NotificationServer {
    fn get_capabilities(&self) -> Vec<String> {
        vec!["body".into(), "persistence".into()]
    }

    fn get_server_information(&self) -> (String, String, String, String) {
        ("CDS".into(), "Chloe Dynamic Shell".into(), env!("CARGO_PKG_VERSION").into(), "1.2".into())
    }

    #[allow(clippy::too_many_arguments)]
    fn notify(
        &self,
        app_name: String,
        replaces_id: u32,
        app_icon: String,
        summary: String,
        body: String,
        _actions: Vec<String>,
        _hints: std::collections::HashMap<String, OwnedValue>,
        expire_timeout: i32,
    ) -> u32 {
        let mut notifications = state().lock().unwrap_or_else(|error| error.into_inner());
        let id = if replaces_id != 0 && notifications.iter().any(|item| item.id == replaces_id) {
            replaces_id
        } else {
            NEXT_ID.fetch_add(1, Ordering::Relaxed).max(1)
        };
        notifications.retain(|item| item.id != id);
        notifications.insert(0, NotificationRecord {
            id,
            app_name,
            app_icon,
            summary,
            body,
            received_unix_ms: now_ms(),
            expire_timeout_ms: if expire_timeout == 0 { 5_000 } else { expire_timeout },
        });
        notifications.truncate(50);
        id
    }

    async fn close_notification(
        &self,
        #[zbus(connection)] connection: &zbus::Connection,
        id: u32,
    ) -> zbus::fdo::Result<()> {
        dismiss(id);
        connection.emit_signal(
            None::<&str>,
            "/org/freedesktop/Notifications",
            "org.freedesktop.Notifications",
            "NotificationClosed",
            &(id, 3_u32),
        ).await.map_err(|error| zbus::fdo::Error::Failed(error.to_string()))?;
        Ok(())
    }
}

pub fn start_server() -> Result<()> {
    let _ = state();
    std::thread::Builder::new().name("cds-notifications-dbus".into()).spawn(|| {
        let result = zbus::block_on(async {
            let _connection = zbus::connection::Builder::session()?
                .name("org.freedesktop.Notifications")?
                .serve_at("/org/freedesktop/Notifications", NotificationServer)?
                .build()
                .await?;
            futures_util::future::pending::<()>().await;
            #[allow(unreachable_code)]
            Ok::<(), zbus::Error>(())
        });
        if let Err(error) = result {
            tracing::error!(%error, "Could not own org.freedesktop.Notifications on the session bus");
        }
    })?;
    Ok(())
}
