use std::collections::BTreeMap;
use std::fs;
use std::path::{Path, PathBuf};
use std::thread;
use std::time::{Duration, Instant};

use amane::Service;

use crate::theme::{self, config_home, home_directory};

const EXTENSIONS: [&str; 3] = ["png", "jpg", "jpeg"];
const MAX_WALLPAPERS: usize = 2000;

pub const SCHEMES: [(&str, &str); 8] = [
    ("scheme-tonal-spot", "Tonal"),
    ("scheme-content", "Content"),
    ("scheme-expressive", "Expressivo"),
    ("scheme-neutral", "Neutro"),
    ("scheme-vibrant", "Vibrante"),
    ("scheme-rainbow", "Rainbow"),
    ("scheme-fruit-salad", "Frutas"),
    ("scheme-fidelity", "Fidelidade"),
];

pub struct Settings {
    wallpaper: String,
    scheme: String,
    dark: bool,
    directory: PathBuf,
    location: String,
    wallpapers: Vec<String>,
    flags: BTreeMap<String, bool>,
    hover_delay_ms: u64,
    scan_requested: bool,
    last_scan: Instant,
}

fn default_flag(name: &str) -> bool {
    !matches!(name, "dnd")
}

impl Service for Settings {
    fn new() -> Self {
        let mut settings = Self {
            wallpaper: String::new(),
            scheme: String::from("scheme-tonal-spot"),
            dark: true,
            directory: home_directory("HOME", "").join("Pictures/Wallpapers"),
            location: String::new(),
            wallpapers: Vec::new(),
            flags: BTreeMap::new(),
            hover_delay_ms: 500,
            scan_requested: true,
            last_scan: Instant::now(),
        };

        settings.load();

        // Create the default folder only when it is still the configured default.
        if settings.directory == home_directory("HOME", "").join("Pictures/Wallpapers") {
            let _ = fs::create_dir_all(&settings.directory);
        }

        settings
    }

    fn listen() {
        loop {
            let directory = {
                let settings = Self::read();

                if settings.scan_requested
                    || settings.last_scan.elapsed() >= Duration::from_secs(60)
                {
                    Some(settings.directory.clone())
                } else {
                    None
                }
            };

            if let Some(directory) = directory {
                // Directory I/O happens without holding the shared service lock.
                let found = scan_directory(&directory);
                let mut settings = Self::write();

                if settings.directory == directory {
                    if let Some(found) = found {
                        settings.wallpapers = found;
                    }

                    settings.scan_requested = false;
                    settings.last_scan = Instant::now();
                }
            }

            thread::sleep(Duration::from_secs(1));
        }
    }
}

impl Settings {
    pub fn wallpaper(&self) -> &str {
        &self.wallpaper
    }

    pub fn wallpapers(&self) -> &[String] {
        &self.wallpapers
    }

    pub fn dark(&self) -> bool {
        self.dark
    }

    pub fn hover_delay_ms(&self) -> u64 {
        self.hover_delay_ms
    }

    pub fn set_hover_delay_ms(value: u64) {
        let mut settings = Self::write();
        settings.hover_delay_ms = value.clamp(100, 2000);
        settings.save();
    }

    pub fn flag(&self, name: &str) -> bool {
        self.flags
            .get(name)
            .copied()
            .unwrap_or_else(|| default_flag(name))
    }

    pub fn set_flag(name: &str, value: bool) {
        let mut settings = Self::write();

        settings.flags.insert(String::from(name), value);
        settings.save();
    }

    pub fn toggle_flag(name: &str) {
        let value = !Self::read().flag(name);

        Self::set_flag(name, value);
    }

    pub fn refresh() -> String {
        let (wallpaper, scheme, dark) = {
            let settings = Self::read();

            (
                settings.wallpaper.clone(),
                settings.scheme.clone(),
                settings.dark,
            )
        };

        if wallpaper.is_empty() {
            return String::from("failed: set a wallpaper first");
        }

        theme::apply("image", &wallpaper, &scheme, dark)
    }

    pub fn scheme_name(&self) -> &str {
        self.scheme.as_str()
    }

    // where the wallpaper tab reads its images from
    pub fn directory_path(&self) -> String {
        display_path(&self.directory)
    }

    // the weather coordinates, empty when the ip decides where we are
    pub fn location(&self) -> &str {
        &self.location
    }

    // MIME of the location, "lat,lon" pairs only, so weather reads it safely
    pub fn coordinates(&self) -> Option<(f32, f32)> {
        let (latitude, longitude) = self.location.trim().split_once(',')?;

        let latitude = latitude.trim().parse::<f32>().ok()?;
        let longitude = longitude.trim().parse::<f32>().ok()?;

        if !(-90.0..=90.0).contains(&latitude)
            || !(-180.0..=180.0).contains(&longitude)
        {
            return None;
        }

        Some((latitude, longitude))
    }

    // changes the wallpaper source directory; the gallery refreshes in the background
    pub fn set_directory(path: &str) -> String {
        let path = expand_home_path(path);

        if !path.is_dir() {
            return format!("failed: no directory at {}", display_path(&path));
        }

        let mut settings = Self::write();

        settings.directory = path;
        settings.scan_requested = true;
        settings.save();

        // The service refreshes the gallery on its next background scan.
        // Never walk a user-selected directory from the input callback.
        String::from("ok")
    }

    // sets the weather spot, "lat,lon"; empty puts it back on auto
    pub fn set_location(location: &str) -> String {
        let location = location.trim().to_lowercase();

        if !location.is_empty() {
            let Some((latitude, longitude)) = location.split_once(',') else {
                return "failed: use latitude,longitude like -20.44,-54.65".into();
            };

            let (Ok(latitude), Ok(longitude)) = (
                latitude.trim().parse::<f32>(),
                longitude.trim().parse::<f32>(),
            ) else {
                return "failed: latitude and longitude are numbers".into();
            };

            if !(-90.0..=90.0).contains(&latitude)
                || !(-180.0..=180.0).contains(&longitude)
            {
                return "failed: latitude must be between -90 and 90 and longitude between -180 and 180".into();
            }
        }

        let mut settings = Self::write();

        settings.location = location;
        settings.save();

        String::from("ok")
    }

    pub fn set_wallpaper(path: &str) -> String {
        let path = expand_home_path(path);

        if !path.is_file() {
            return format!("failed: no image at {}", display_path(&path));
        }

        let path = path.to_string_lossy().into_owned();

        // the lock is released before matugen runs, so drawing never waits on it
        let (wallpaper, scheme, dark) = {
            let mut settings = Self::write();

            settings.wallpaper = path.clone();
            settings.save();

            (
                settings.wallpaper.clone(),
                settings.scheme.clone(),
                settings.dark,
            )
        };

        regenerate(&wallpaper, &scheme, dark, true)
    }

    // the views call this so the click never waits on a whole matugen run
    pub fn set_wallpaper_live(path: &str) {
        let path = expand_home_path(path);

        if !path.is_file() {
            return;
        }

        let path = path.to_string_lossy().into_owned();

        let (wallpaper, scheme, dark) = {
            let mut settings = Self::write();

            settings.wallpaper = path.clone();
            settings.save();

            (
                settings.wallpaper.clone(),
                settings.scheme.clone(),
                settings.dark,
            )
        };

        regenerate(&wallpaper, &scheme, dark, false);
    }

    pub fn set_dark(dark: bool) -> String {
        let (wallpaper, scheme) = {
            let mut settings = Self::write();

            settings.dark = dark;
            settings.save();

            (settings.wallpaper.clone(), settings.scheme.clone())
        };

        regenerate(&wallpaper, &scheme, dark, true)
    }

    pub fn set_dark_live(dark: bool) {
        let (wallpaper, scheme) = {
            let mut settings = Self::write();

            settings.dark = dark;
            settings.save();

            (settings.wallpaper.clone(), settings.scheme.clone())
        };

        regenerate(&wallpaper, &scheme, dark, false);
    }

    // changes the matugen scheme and regenerates everything with it
    pub fn set_scheme(scheme: &str) -> String {
        let (wallpaper, dark) = {
            let mut settings = Self::write();

            settings.scheme = String::from(scheme);
            settings.save();

            (settings.wallpaper.clone(), settings.dark)
        };

        regenerate(&wallpaper, scheme, dark, true)
    }

    pub fn set_scheme_live(scheme: &str) {
        let (wallpaper, dark) = {
            let mut settings = Self::write();

            settings.scheme = String::from(scheme);
            settings.save();

            (settings.wallpaper.clone(), settings.dark)
        };

        regenerate(&wallpaper, scheme, dark, false);
    }

    fn file() -> PathBuf {
        config_home().join("chloe-dynamic-shell/settings.conf")
    }

    fn load(&mut self) {
        let Ok(text) = fs::read_to_string(Self::file()) else {
            return;
        };

        for line in text.lines() {
            let Some((key, value)) = line.split_once('=') else {
                continue;
            };

            let value = value.trim();

            match key.trim() {
                "wallpaper" => self.wallpaper = String::from(value),
                "scheme" => self.scheme = String::from(value),
                "dark" => self.dark = value != "false",
                "directory" => self.directory = expand_home_path(value),
                "location" => self.location = String::from(value),
                "hover_delay_ms" => {
                    self.hover_delay_ms = value.parse::<u64>().unwrap_or(500).clamp(100, 2000)
                },
                other => {
                    if let Some(name) = other.strip_prefix("flag.") {
                        self.flags.insert(String::from(name), value == "true");
                    }
                }
            }
        }
    }

    fn save(&self) {
        let file = Self::file();

        if let Some(folder) = file.parent() {
            let _ = fs::create_dir_all(folder);
        }

        let mut text = format!(
            "wallpaper={}\nscheme={}\ndark={}\ndirectory={}\nlocation={}\nhover_delay_ms={}\n",
            self.wallpaper,
            self.scheme,
            self.dark,
            self.directory.display(),
            self.location,
            self.hover_delay_ms,
        );

        for (name, value) in &self.flags {
            text.push_str(&format!("flag.{name}={value}\n"));
        }

        // Publish settings atomically: a crash or power loss must not leave
        // a truncated config that resets the shell to its defaults.
        let staged = file.with_extension("conf.tmp");

        if fs::write(&staged, text).is_ok() {
            if fs::rename(&staged, &file).is_err() {
                let _ = fs::remove_file(staged);
            }
        }
    }

}

fn scan_directory(directory: &Path) -> Option<Vec<String>> {
    let mut found: Vec<String> = fs::read_dir(directory)
        .ok()?
        .flatten()
        .take(MAX_WALLPAPERS)
        .map(|entry| entry.path())
        .filter(|path| {
            path.is_file()
                && path.extension()
                    .and_then(|extension| extension.to_str())
                    .is_some_and(|extension| {
                        EXTENSIONS.contains(&extension.to_ascii_lowercase().as_str())
                    })
        })
        .map(|path| path.to_string_lossy().into_owned())
        .collect();

    found.sort();
    Some(found)
}

/*
 * one place decides whether a matugen run waits. the ipc answers only
 * after the colors exist, while the views spawn it and stay responsive.
 */
fn expand_home_path(path: &str) -> PathBuf {
    if path == "~" {
        return home_directory("HOME", "");
    }
    if let Some(rest) = path.strip_prefix("~/") {
        return home_directory("HOME", "").join(rest);
    }
    PathBuf::from(path)
}

fn display_path(path: &std::path::Path) -> String {
    let home = home_directory("HOME", "");
    if let Ok(relative) = path.strip_prefix(&home) {
        if relative.as_os_str().is_empty() {
            return String::from("~");
        }
        return format!("~/{}", relative.display());
    }
    path.to_string_lossy().into_owned()
}

fn regenerate(wallpaper: &str, scheme: &str, dark: bool, wait: bool) -> String {
    if wallpaper.is_empty() {
        return String::from("failed: set a wallpaper first");
    }

    if wait {
        theme::apply("image", wallpaper, scheme, dark)
    } else {
        theme::apply_async("image", wallpaper, scheme, dark);

        String::from("ok")
    }
}
