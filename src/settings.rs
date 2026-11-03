use std::collections::BTreeMap;
use std::fs;
use std::path::PathBuf;
use std::time::Duration;

use amane::Service;

use crate::theme::{self, config_home, home_directory};

const EXTENSIONS: [&str; 3] = ["png", "jpg", "jpeg"];

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
        };

        settings.load();
        settings.scan();

        settings
    }

    fn interval() -> Duration {
        Duration::from_secs(5)
    }

    fn update(&mut self) -> bool {
        self.scan()
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
    pub fn directory_path(&self) -> &str {
        self.directory.to_str().unwrap_or("")
    }

    // the weather coordinates, empty when the ip decides where we are
    pub fn location(&self) -> &str {
        &self.location
    }

    // MIME of the location, "lat,lon" pairs only, so weather reads it safely
    pub fn coordinates(&self) -> Option<(f32, f32)> {
        let (latitude, longitude) = self.location.trim().split_once(',')?;

        Some((
            latitude.trim().parse().ok()?,
            longitude.trim().parse().ok()?,
        ))
    }

    // moves the wallpaper source directory, rescanning right away
    pub fn set_directory(path: &str) -> String {
        if fs::metadata(path).is_err() {
            return format!("failed: no directory at {path}");
        }

        let mut settings = Self::write();

        settings.directory = PathBuf::from(path);
        settings.save();

        settings.scan();

        String::from("ok")
    }

    // sets the weather spot, "lat,lon"; empty puts it back on auto
    pub fn set_location(location: &str) -> String {
        let location = location.trim().to_lowercase();

        if !location.is_empty() {
            let Some((latitude, longitude)) = location.split_once(',') else {
                return "failed: use latitude,longitude like -20.44,-54.65".into();
            };

            if latitude.trim().parse::<f32>().is_err() || longitude.trim().parse::<f32>().is_err() {
                return "failed: latitude and longitude are numbers".into();
            }
        }

        let mut settings = Self::write();

        settings.location = location;
        settings.save();

        String::from("ok")
    }

    pub fn set_wallpaper(path: &str) -> String {
        if !PathBuf::from(path).is_file() {
            return format!("failed: no image at {path}");
        }

        // the lock is released before matugen runs, so drawing never waits on it
        let (wallpaper, scheme, dark) = {
            let mut settings = Self::write();

            settings.wallpaper = String::from(path);
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
        if !PathBuf::from(path).is_file() {
            return;
        }

        let (wallpaper, scheme, dark) = {
            let mut settings = Self::write();

            settings.wallpaper = String::from(path);
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
                "directory" => self.directory = PathBuf::from(value),
                "location" => self.location = String::from(value),
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
            "wallpaper={}\nscheme={}\ndark={}\ndirectory={}\nlocation={}\n",
            self.wallpaper,
            self.scheme,
            self.dark,
            self.directory.display(),
            self.location,
        );

        for (name, value) in &self.flags {
            text.push_str(&format!("flag.{name}={value}\n"));
        }

        let _ = fs::write(file, text);
    }

    fn scan(&mut self) -> bool {
        let mut found: Vec<String> = fs::read_dir(&self.directory)
            .map(|entries| {
                entries
                    .flatten()
                    .map(|entry| entry.path())
                    .filter(|path| {
                        path.extension()
                            .and_then(|extension| extension.to_str())
                            .is_some_and(|extension| {
                                EXTENSIONS.contains(&extension.to_lowercase().as_str())
                            })
                    })
                    .map(|path| path.to_string_lossy().into_owned())
                    .collect()
            })
            .unwrap_or_default();

        found.sort();

        let changed = found != self.wallpapers;

        self.wallpapers = found;

        changed
    }
}

/*
 * one place decides whether a matugen run waits. the ipc answers only
 * after the colors exist, while the views spawn it and stay responsive.
 */
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
