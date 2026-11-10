use anyhow::{Context, Result, bail};
use std::hash::{Hash, Hasher};
use std::path::PathBuf;
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
    pub artwork_path: Option<PathBuf>,
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

async fn cache_artwork(url: &str) -> Option<PathBuf> {
    if let Some(path) = url.strip_prefix("file://") {
        let path = PathBuf::from(path);
        return path.is_file().then_some(path);
    }
    if !(url.starts_with("https://") || url.starts_with("http://")) {
        return None;
    }

    let mut hasher = std::collections::hash_map::DefaultHasher::new();
    url.hash(&mut hasher);
    let directory = crate::paths::Paths::discover().ok()?.cache.join("artwork");
    std::fs::create_dir_all(&directory).ok()?;
    let stem = format!("{:016x}", hasher.finish());
    for extension in ["png", "jpg", "jpeg", "webp", "gif", "bmp"] {
        let cached = directory.join(format!("{stem}.{extension}"));
        if cached.is_file() { return Some(cached); }
    }

    let response = reqwest::get(url).await.ok()?.error_for_status().ok()?;
    if response.content_length().is_some_and(|length| length > 12 * 1024 * 1024) {
        return None;
    }
    let bytes = response.bytes().await.ok()?;
    if bytes.len() > 12 * 1024 * 1024 { return None; }
    let format = image::guess_format(&bytes).ok()?;
    let extension = match format {
        image::ImageFormat::Png => "png",
        image::ImageFormat::Jpeg => "jpg",
        image::ImageFormat::WebP => "webp",
        image::ImageFormat::Gif => "gif",
        image::ImageFormat::Bmp => "bmp",
        _ => return None,
    };
    let target = directory.join(format!("{stem}.{extension}"));
    let staged = target.with_extension(format!("{extension}.tmp"));
    tokio::fs::write(&staged, &bytes).await.ok()?;
    tokio::fs::rename(&staged, &target).await.ok()?;
    Some(target)
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
    let artwork_path = cache_artwork(&art_url).await;
    let position_ms = player_proxy.get_property::<i64>("Position").await.unwrap_or(0).max(0) as u64 / 1000;
    let length_ms = metadata.get("mpris:length")
        .and_then(|value| i64::try_from(value.clone()).ok())
        .unwrap_or(0).max(0) as u64 / 1000;

    Ok(MediaSnapshot { player, title, artist, album, status, art_url, artwork_path, position_ms, length_ms })
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
