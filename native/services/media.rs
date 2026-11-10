use anyhow::{Context, Result, bail};
use zbus::{Connection, Proxy};
use zbus::fdo::DBusProxy;
use zbus::zvariant::OwnedValue;

#[derive(Debug, Clone, Default)]
pub struct MediaSnapshot {
    pub player: String,
    pub title: String,
    pub artist: String,
    pub album: String,
    pub status: String,
    pub art_url: String,
    pub position_ms: u64,
    pub length_ms: u64,
}

impl MediaSnapshot {
    pub fn is_playing(&self) -> bool { self.status.eq_ignore_ascii_case("Playing") }
    pub fn available(&self) -> bool { !self.player.is_empty() }
}

fn value_string(value: Option<&OwnedValue>) -> String {
    value.and_then(|value| String::try_from(value.clone()).ok()).unwrap_or_default()
}

fn value_strings(value: Option<&OwnedValue>) -> String {
    value.and_then(|value| Vec::<String>::try_from(value.clone()).ok())
        .unwrap_or_default().join(", ")
}

pub async fn snapshot() -> Result<MediaSnapshot> {
    let connection = Connection::session().await.context("session D-Bus is unavailable")?;
    let bus = DBusProxy::new(&connection).await?;
    let names = bus.list_names().await?;
    let player = names.into_iter().map(|name| name.to_string())
        .filter(|name| name.starts_with("org.mpris.MediaPlayer2."))
        .next().unwrap_or_default();
    if player.is_empty() { return Ok(MediaSnapshot::default()); }

    let proxy = Proxy::new(
        &connection,
        player.as_str(),
        "/org/mpris/MediaPlayer2",
        "org.freedesktop.DBus.Properties",
    ).await?;
    let player_proxy = Proxy::new(
        &connection,
        player.as_str(),
        "/org/mpris/MediaPlayer2",
        "org.mpris.MediaPlayer2.Player",
    ).await?;

    let status: String = player_proxy.get_property("PlaybackStatus").await.unwrap_or_else(|_| "Stopped".into());
    let metadata: std::collections::HashMap<String, OwnedValue> =
        proxy.call("Get", &("org.mpris.MediaPlayer2.Player", "Metadata")).await
            .ok()
            .and_then(|reply: (String, OwnedValue)| reply.1.try_into().ok())
            .unwrap_or_default();

    let title = value_string(metadata.get("xesam:title"));
    let artist = value_strings(metadata.get("xesam:artist"));
    let album = value_string(metadata.get("xesam:album"));
    let art_url = value_string(metadata.get("mpris:artUrl"));
    let position_ms = player_proxy.get_property::<i64>("Position").await.unwrap_or(0).max(0) as u64 / 1000;
    let length_ms = metadata.get("mpris:length")
        .and_then(|value| i64::try_from(value.clone()).ok())
        .unwrap_or(0).max(0) as u64 / 1000;

    Ok(MediaSnapshot { player, title, artist, album, status, art_url, position_ms, length_ms })
}

pub async fn control(action: &str) -> Result<()> {
    if !matches!(action, "PlayPause" | "Next" | "Previous" | "Stop") {
        bail!("unsupported MPRIS action");
    }
    let connection = Connection::session().await.context("session D-Bus is unavailable")?;
    let bus = DBusProxy::new(&connection).await?;
    let names = bus.list_names().await?;
    let player = names.into_iter().map(|name| name.to_string())
        .find(|name| name.starts_with("org.mpris.MediaPlayer2."))
        .context("no MPRIS player is currently available")?;
    let proxy = Proxy::new(
        &connection,
        player.as_str(),
        "/org/mpris/MediaPlayer2",
        "org.mpris.MediaPlayer2.Player",
    ).await?;
    proxy.call::<_, _, ()>(action, &()).await?;
    Ok(())
}
