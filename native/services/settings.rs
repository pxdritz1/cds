use std::collections::{BTreeMap, BTreeSet};
use std::path::PathBuf;

use anyhow::{Context, Result};

use crate::paths::Paths;

/// Persistent CDS preferences. The line-based format deliberately remains
/// compatible with the original CDS settings.conf so upgrades are lossless.
#[derive(Debug, Clone)]
pub struct Settings {
    pub language: String,
    pub dark: bool,
    pub scheme: String,
    pub wallpaper: String,
    pub wallpaper_directory: String,
    pub location: String,
    pub motion_style: String,
    pub hover_delay_ms: u64,
    pub media_title_max_width: u32,
    pub wallpaper_rotation: bool,
    pub wallpaper_interval_minutes: u32,
    pub flags: BTreeMap<String, bool>,
    pub pinned_apps: BTreeSet<String>,
}

impl Default for Settings {
    fn default() -> Self {
        Self {
            language: "pt-BR".into(),
            dark: true,
            scheme: "scheme-tonal-spot".into(),
            wallpaper: String::new(),
            wallpaper_directory: dirs_home().join("Pictures/Wallpapers").to_string_lossy().into_owned(),
            location: String::new(),
            motion_style: "m3".into(),
            hover_delay_ms: 500,
            media_title_max_width: 320,
            wallpaper_rotation: false,
            wallpaper_interval_minutes: 30,
            flags: BTreeMap::new(),
            pinned_apps: BTreeSet::new(),
        }
    }
}

fn dirs_home() -> PathBuf {
    std::env::var_os("HOME").map(PathBuf::from).unwrap_or_else(|| PathBuf::from("/"))
}

impl Settings {
    pub fn load() -> Self {
        let mut settings = Self::default();
        if let Ok(paths) = Paths::discover() {
            let file = paths.config.join("settings.conf");
            if let Ok(contents) = std::fs::read_to_string(file) {
                settings.parse_legacy_format(&contents);
            }
        }
        settings
    }

    fn parse_legacy_format(&mut self, contents: &str) {
        for line in contents.lines() {
            let Some((key, value)) = line.split_once('=') else { continue };
            let key = key.trim();
            let value = value.trim();
            match key {
                "language" => self.language = if value == "en" { "en" } else { "pt-BR" }.into(),
                "dark" => self.dark = value != "false",
                "scheme" => self.scheme = value.into(),
                "wallpaper" => self.wallpaper = value.into(),
                "directory" => self.wallpaper_directory = value.into(),
                "location" => self.location = value.into(),
                "motion_style" => self.motion_style = if value == "fluent" { "fluent" } else { "m3" }.into(),
                "hover_delay_ms" => self.hover_delay_ms = value.parse().unwrap_or(500).clamp(100, 2000),
                "media_title_max_width" => self.media_title_max_width = value.parse().unwrap_or(320).clamp(104, 520),
                "wallpaper_rotation" => self.wallpaper_rotation = value == "true",
                "wallpaper_interval_minutes" => self.wallpaper_interval_minutes = value.parse().unwrap_or(30).clamp(1, 1440),
                other if other.starts_with("pinned.") => {
                    let exec = other.trim_start_matches("pinned.");
                    if value == "true" { self.pinned_apps.insert(exec.to_owned()); }
                    else { self.pinned_apps.remove(exec); }
                }
                other if other.starts_with("flag.") => {
                    self.flags.insert(other.trim_start_matches("flag.").to_owned(), value == "true");
                }
                _ => {}
            }
        }
    }

    pub fn is_pinned(&self, key: &str) -> bool {
        self.pinned_apps.contains(key)
    }

    pub fn toggle_pinned(&mut self, key: &str) -> Result<bool> {
        let pinned = if self.pinned_apps.remove(key) { false } else {
            self.pinned_apps.insert(key.to_owned());
            true
        };
        self.save()?;
        Ok(pinned)
    }

    pub fn flag(&self, name: &str) -> bool {
        self.flags.get(name).copied().unwrap_or(!matches!(name, "dnd" | "trans_meter_colors" | "clock_analog"))
    }

    pub fn set_flag(&mut self, name: &str, value: bool) -> Result<()> {
        self.flags.insert(name.to_owned(), value);
        self.save()
    }

    pub fn save(&self) -> Result<()> {
        let paths = Paths::discover()?;
        std::fs::create_dir_all(&paths.config)?;
        let mut output = format!(
            "scheme={}\ndark={}\nwallpaper={}\ndirectory={}\nlocation={}\nhover_delay_ms={}\nlanguage={}\nmedia_title_max_width={}\nwallpaper_rotation={}\nwallpaper_interval_minutes={}\nmotion_style={}\n",
            self.scheme, self.dark, self.wallpaper, self.wallpaper_directory, self.location,
            self.hover_delay_ms, self.language, self.media_title_max_width,
            self.wallpaper_rotation, self.wallpaper_interval_minutes, self.motion_style
        );
        for key in &self.pinned_apps { output.push_str(&format!("pinned.{key}=true\n")); }
        for (key, value) in &self.flags { output.push_str(&format!("flag.{key}={value}\n")); }
        let target = paths.config.join("settings.conf");
        let staged = target.with_extension("conf.tmp");
        std::fs::write(&staged, output).context("could not stage CDS settings")?;
        std::fs::rename(&staged, &target).context("could not atomically save CDS settings")?;
        Ok(())
    }
}
