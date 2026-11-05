use std::collections::BTreeMap;
use std::fs;
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicU64, Ordering};
use std::thread;
use std::time::{Duration, Instant};

use amane::Service;

use crate::theme::{self, config_home, home_directory};

const EXTENSIONS: [&str; 3] = ["png", "jpg", "jpeg"];
const MAX_WALLPAPERS: usize = 2000;

static SCAN_GENERATION: AtomicU64 = AtomicU64::new(1);

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
    language: String,
    pinned_apps: Vec<String>,
    media_title_max_width: u32,
    wallpaper_rotation: bool,
    wallpaper_interval_minutes: u32,
    motion_style: String,
    workspaces_alignment: String,
    center_alignment: String,
    status_alignment: String,
    status_positions: BTreeMap<String, String>,
}

fn default_flag(name: &str) -> bool {
    !matches!(name, "dnd" | "trans_meter_colors" | "clock_analog")
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
            language: String::from("pt-BR"),
            pinned_apps: Vec::new(),
            media_title_max_width: 320,
            wallpaper_rotation: false,
            wallpaper_interval_minutes: 30,
            motion_style: String::from("m3"),
            workspaces_alignment: String::from("left"),
            center_alignment: String::from("center"),
            status_alignment: String::from("right"),
            status_positions: ["network", "bluetooth", "volume", "microphone", "battery"]
                .into_iter()
                .map(|name| (String::from(name), String::from("left")))
                .collect(),
        };

        settings.load();

        // Create the default folder only when it is still the configured default.
        if settings.directory == home_directory("HOME", "").join("Pictures/Wallpapers") {
            let _ = fs::create_dir_all(&settings.directory);
        }

        settings
    }

    fn listen() {
        let mut scanned_generation = 0;
        let mut last_scan: Option<Instant> = None;
        let mut last_rotation = Instant::now();

        loop {
            let generation = SCAN_GENERATION.load(Ordering::Relaxed);
            let due = generation != scanned_generation
                || last_scan.is_none_or(|last| last.elapsed() >= Duration::from_secs(60));

            if due {
                let directory = Self::read().directory.clone();

                // Directory I/O happens without holding the shared service lock.
                let found = scan_directory(&directory);

                if generation == SCAN_GENERATION.load(Ordering::Relaxed) {
                    if let Some(found) = found {
                        let mut settings = Self::write();

                        if settings.directory == directory && settings.wallpapers != found {
                            settings.wallpapers = found;
                        }
                    }

                    scanned_generation = generation;
                    last_scan = Some(Instant::now());
                }
            }

            let rotation = {
                let settings = Self::read();
                if !settings.wallpaper_rotation || settings.wallpapers.len() < 2
                    || last_rotation.elapsed() < Duration::from_secs(settings.wallpaper_interval_minutes as u64 * 60)
                {
                    None
                } else {
                    let next = settings.wallpapers.iter()
                        .position(|path| path == &settings.wallpaper)
                        .map_or(0, |current| (current + 1) % settings.wallpapers.len());
                    Some(settings.wallpapers[next].clone())
                }
            };

            if let Some(path) = rotation {
                Self::set_wallpaper_live(&path);
                last_rotation = Instant::now();
            } else if !Self::read().wallpaper_rotation {
                last_rotation = Instant::now();
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


    pub fn media_title_max_width(&self) -> u32 {
        self.media_title_max_width
    }

    pub fn wallpaper_rotation(&self) -> bool {
        self.wallpaper_rotation
    }

    pub fn wallpaper_interval_minutes(&self) -> u32 {
        self.wallpaper_interval_minutes
    }

    pub fn motion_style(&self) -> &str {
        &self.motion_style
    }

    pub fn workspaces_alignment(&self) -> &str {
        &self.workspaces_alignment
    }

    pub fn center_alignment(&self) -> &str {
        &self.center_alignment
    }

    pub fn status_alignment(&self) -> &str {
        &self.status_alignment
    }

    pub fn status_position(&self, indicator: &str) -> &str {
        self.status_positions
            .get(indicator)
            .map(String::as_str)
            .unwrap_or("left")
    }

    pub fn set_status_position(indicator: &str, position: &str) {
        if !matches!(indicator, "network" | "bluetooth" | "volume" | "microphone" | "battery") {
            return;
        }

        let position = match position {
            "center" => "center",
            "right" => "right",
            _ => "left",
        };

        let mut settings = Self::write();
        settings.status_positions.insert(String::from(indicator), String::from(position));
        settings.save();
        drop(settings);
        crate::ui::Ui::refresh();
    }

    pub fn set_motion_style(value: &str) {
        let mut settings = Self::write();
        settings.motion_style = if value == "fluent" { String::from("fluent") } else { String::from("m3") };
        settings.save();
    }

    pub fn set_workspaces_alignment(value: &str) {
        let mut settings = Self::write();
        settings.workspaces_alignment = if value == "right" { String::from("right") } else { String::from("left") };
        settings.save();
    }

    pub fn set_center_alignment(value: &str) {
        let mut settings = Self::write();
        settings.center_alignment = match value {
            "left" => String::from("left"),
            "right" => String::from("right"),
            _ => String::from("center"),
        };
        settings.save();
    }

    pub fn set_status_alignment(value: &str) {
        let mut settings = Self::write();
        settings.status_alignment = if value == "left" { String::from("left") } else { String::from("right") };
        settings.save();
    }

    pub fn set_wallpaper_rotation(enabled: bool) {
        let mut settings = Self::write();
        settings.wallpaper_rotation = enabled;
        settings.save();
        drop(settings);
        crate::ui::Ui::refresh();
    }

    pub fn set_wallpaper_interval_minutes(value: u32) {
        let mut settings = Self::write();
        settings.wallpaper_interval_minutes = value.clamp(1, 1440);
        settings.save();
        drop(settings);
        crate::ui::Ui::refresh();
    }

    pub fn set_media_title_max_width(value: u32) {
        let mut settings = Self::write();
        settings.media_title_max_width = value.clamp(104, 520);
        settings.save();
        drop(settings);
        crate::ui::Ui::refresh();
    }

    pub fn language(&self) -> &str {
        &self.language
    }

    // Pinned applications are shared by the launcher and Spotlight, not the Dock.
    pub fn is_pinned(&self, exec: &str) -> bool {
        self.pinned_apps.iter().any(|pinned| pinned == exec)
    }

    pub fn toggle_pinned(exec: &str) {
        let mut settings = Self::write();
        if let Some(index) = settings.pinned_apps.iter().position(|item| item == exec) {
            settings.pinned_apps.remove(index);
        } else {
            settings.pinned_apps.push(String::from(exec));
        }
        settings.save();
        drop(settings);
        crate::ui::Ui::refresh();
    }

    pub fn set_language(language: &str) {
        let mut settings = Self::write();
        settings.language = if language == "en" { String::from("en") } else { String::from("pt-BR") };
        settings.save();
        drop(settings);

        // The formatted date is produced by the system locale; refresh it
        // immediately instead of waiting for the next minute boundary.
        crate::clock::Clock::refresh();
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
        drop(settings);
        crate::ui::Ui::refresh();
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
        SCAN_GENERATION.fetch_add(1, Ordering::Relaxed);
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
        if !SCHEMES.iter().any(|(name, _)| *name == scheme) {
            return String::from("failed: unsupported Matugen scheme");
        }

        let (wallpaper, dark) = {
            let mut settings = Self::write();

            settings.scheme = String::from(scheme);
            settings.save();

            (settings.wallpaper.clone(), settings.dark)
        };

        regenerate(&wallpaper, scheme, dark, true)
    }

    pub fn set_scheme_live(scheme: &str) {
        if !SCHEMES.iter().any(|(name, _)| *name == scheme) {
            return;
        }

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
                "wallpaper" => self.wallpaper = expand_home_path(value).to_string_lossy().into_owned(),
                "scheme" => {
                    if SCHEMES.iter().any(|(name, _)| *name == value) {
                        self.scheme = String::from(value);
                    }
                },
                "dark" => self.dark = value != "false",
                "directory" => self.directory = expand_home_path(value),
                "location" => self.location = String::from(value),
                "language" => self.language = if value == "en" { String::from("en") } else { String::from("pt-BR") },
                "pinned_app" => { if !self.pinned_apps.iter().any(|item| item == value) { self.pinned_apps.push(String::from(value)); } },
                "media_title_max_width" => {
                    self.media_title_max_width = value.parse::<u32>().unwrap_or(320).clamp(104, 520)
                },
                "wallpaper_rotation" => self.wallpaper_rotation = value == "true",
                "wallpaper_interval_minutes" => {
                    self.wallpaper_interval_minutes = value.parse::<u32>().unwrap_or(30).clamp(1, 1440)
                },
                "motion_style" => self.motion_style = if value == "fluent" { String::from("fluent") } else { String::from("m3") },
                "workspaces_alignment" => self.workspaces_alignment = if value == "right" { String::from("right") } else { String::from("left") },
                "center_alignment" => self.center_alignment = match value { "left" => String::from("left"), "right" => String::from("right"), _ => String::from("center") },
                "status_alignment" => self.status_alignment = if value == "left" { String::from("left") } else { String::from("right") },
                "status_position.network" | "status_position.bluetooth" | "status_position.volume"
                | "status_position.microphone" | "status_position.battery" => {
                    let name = key.trim().strip_prefix("status_position.").unwrap_or("");
                    let position = match value {
                        "center" => "center",
                        "right" => "right",
                        _ => "left",
                    };
                    self.status_positions.insert(String::from(name), String::from(position));
                },
                "hover_delay_ms" => {
                    self.hover_delay_ms = value.parse::<u64>().unwrap_or(500).clamp(100, 2000)
                },
                other => {
                    if let Some(name) = other.strip_prefix("flag.") {
                        if !matches!(name, "show_dock" | "dock_magnification" | "dock_reserve_space" | "dock_auto_hide") {
                            self.flags.insert(String::from(name), value == "true");
                        }
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
            "wallpaper={}\nscheme={}\ndark={}\ndirectory={}\nlocation={}\nhover_delay_ms={}\nlanguage={}\nmedia_title_max_width={}\nwallpaper_rotation={}\nwallpaper_interval_minutes={}\nmotion_style={}\nworkspaces_alignment={}\ncenter_alignment={}\nstatus_alignment={}\n",
            self.wallpaper,
            self.scheme,
            self.dark,
            self.directory.display(),
            self.location,
            self.hover_delay_ms,
            self.language,
            self.media_title_max_width,
            self.wallpaper_rotation,
            self.wallpaper_interval_minutes,
            self.motion_style,
            self.workspaces_alignment,
            self.center_alignment,
            self.status_alignment,
        );

        for app in &self.pinned_apps {
            text.push_str(&format!("pinned_app={app}\n"));
        }

        for (name, value) in &self.status_positions {
            text.push_str(&format!("status_position.{name}={value}\n"));
        }

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
        .map(|entry| entry.path())
        .filter(|path| {
            path.is_file()
                && path.extension()
                    .and_then(|extension| extension.to_str())
                    .is_some_and(|extension| {
                        EXTENSIONS.contains(&extension.to_ascii_lowercase().as_str())
                    })
        })
        .take(MAX_WALLPAPERS)
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
    let expanded = if path == "~" {
        home_directory("HOME", "")
    } else if let Some(rest) = path.strip_prefix("~/") {
        home_directory("HOME", "").join(rest)
    } else {
        PathBuf::from(path)
    };

    // Store one stable absolute representation whenever the target exists.
    fs::canonicalize(&expanded).unwrap_or(expanded)
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
