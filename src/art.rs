use std::collections::{HashMap, HashSet};
use std::hash::{Hash, Hasher};
use std::path::PathBuf;
use std::process::Command;
use std::sync::{Mutex, OnceLock};
use std::thread;

use amane::{Center, Image, Rectangle};

use crate::island::widgets::{MUSIC, icon};
use crate::theme::Scheme;

#[derive(Default)]
struct RemoteArt {
    paths: HashMap<String, PathBuf>,
    pending: HashSet<String>,
}

static REMOTE_ART: OnceLock<Mutex<RemoteArt>> = OnceLock::new();

fn remote_art() -> &'static Mutex<RemoteArt> {
    REMOTE_ART.get_or_init(|| Mutex::new(RemoteArt::default()))
}

fn artwork_path(source: &str) -> Option<PathBuf> {
    if let Some(path) = source.strip_prefix("file://") {
        let path = PathBuf::from(path);
        return path.is_file().then_some(path);
    }

    if source.starts_with('/') {
        let path = PathBuf::from(source);
        return path.is_file().then_some(path);
    }

    if !source.starts_with("https://") && !source.starts_with("http://") {
        return None;
    }

    {
        let mut cache = remote_art().lock().unwrap_or_else(|error| error.into_inner());

        if let Some(path) = cache.paths.get(source) {
            return path.is_file().then(|| path.clone());
        }

        if !cache.pending.insert(source.to_string()) {
            return None;
        }
    }

    let url = source.to_string();
    let mut hasher = std::collections::hash_map::DefaultHasher::new();
    url.hash(&mut hasher);
    let key = format!("{:016x}", hasher.finish());

    let cache_dir = std::env::var_os("XDG_CACHE_HOME")
        .map(PathBuf::from)
        .or_else(|| std::env::var_os("HOME").map(|home| PathBuf::from(home).join(".cache")))
        .map(|home| home.join("chloe-dynamic-shell/media-art"));

    let Some(cache_dir) = cache_dir else {
        remote_art().lock().unwrap_or_else(|error| error.into_inner()).pending.remove(&url);
        return None;
    };

    let extension = url
        .split(['?', '#'])
        .next()
        .unwrap_or(&url)
        .rsplit_once('.')
        .map(|(_, extension)| extension.to_ascii_lowercase())
        .filter(|extension| matches!(extension.as_str(), "png" | "jpg" | "jpeg" | "svg"))
        .unwrap_or_else(|| String::from("img"));

    let destination = cache_dir.join(format!("{key}.{extension}"));

    if destination.is_file() {
        let mut cache = remote_art().lock().unwrap_or_else(|error| error.into_inner());
        cache.pending.remove(&url);
        cache.paths.insert(url, destination.clone());
        return Some(destination);
    }

    let temporary = cache_dir.join(format!("{key}.part"));

    let worker_url = url.clone();
    let worker_destination = destination.clone();

    let spawned = thread::Builder::new()
        .name(String::from("cds-media-art"))
        .spawn(move || {
            let _ = std::fs::create_dir_all(&cache_dir);

            let downloaded = Command::new("curl")
                .args([
                    "--fail",
                    "--silent",
                    "--show-error",
                    "--location",
                    "--max-time",
                    "15",
                    "--max-filesize",
                    "12582912",
                    "--output",
                ])
                .arg(&temporary)
                .arg(&worker_url)
                .status()
                .is_ok_and(|status| status.success());

            let ready = downloaded
                && temporary.is_file()
                && std::fs::rename(&temporary, &worker_destination).is_ok();

            if !ready {
                let _ = std::fs::remove_file(&temporary);
            }

            let mut cache = remote_art().lock().unwrap_or_else(|error| error.into_inner());
            cache.pending.remove(&worker_url);

            if ready {
                cache.paths.insert(worker_url, worker_destination);
            }

            drop(cache);
            amane::request_frame();
        });

    if spawned.is_err() {
        remote_art().lock().unwrap_or_else(|error| error.into_inner()).pending.remove(&url);
    }

    None
}

// an album cover, or the music note placeholder when there is no image
pub fn artwork(path: &str, size: f32, radius: f32, scheme: &Scheme) -> Rectangle {
    let base = Rectangle::new()
        .width(size)
        .height(size)
        .radius(radius)
        .clip();

    if let Some(file) = artwork_path(path) {
        let pixels = (size * 2.0).round() as u32;
        base.fill(Image::cover(file).thumbnail(pixels, pixels))
    } else {
        base.fill(scheme.secondary_container)
            .align_child(Center, Center)
            .child(icon(MUSIC, size * 0.42, scheme.on_secondary_container))
    }
}

// metadata can be empty even while a player is open, so labels get a fallback
pub fn media_labels(media: &amane::Media) -> (String, String) {
    let active = media.active().is_some();

    let title = media.title();
    let artist = media.artist();

    let title = if !title.is_empty() {
        String::from(title)
    } else if active {
        String::from("Título desconhecido")
    } else {
        String::from("Nada tocando")
    };

    let artist = if !artist.is_empty() {
        String::from(artist)
    } else if active {
        String::from("Artista desconhecido")
    } else {
        String::from("Abra um player pra começar")
    };

    (title, artist)
}
