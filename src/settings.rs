use std::fs;
use std::path::PathBuf;
use std::time::Duration;

use amane::Service;

use crate::theme::{self, config_home, home_directory};

const EXTENSIONS: [&str; 3] = ["png", "jpg", "jpeg"];

pub struct Settings {
    wallpaper: String,
    scheme: String,
    dark: bool,
    directory: PathBuf,
    wallpapers: Vec<String>,
}

impl Service for Settings {
    fn new() -> Self {
        let mut settings = Self {
            wallpaper: String::new(),
            scheme: String::from("scheme-tonal-spot"),
            dark: true,
            directory: home_directory("HOME", "").join("Pictures/Wallpapers"),
            wallpapers: Vec::new(),
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

    pub fn set_wallpaper(path: &str) {
        let mut settings = Self::write();

        settings.wallpaper = String::from(path);
        settings.save();
        settings.generate();
    }

    pub fn set_dark(dark: bool) {
        let mut settings = Self::write();

        settings.dark = dark;
        settings.save();
        settings.generate();
    }

    fn generate(&self) {
        if !self.wallpaper.is_empty() {
            theme::apply("image", &self.wallpaper, &self.scheme, self.dark);
        }
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
                _ => {}
            }
        }
    }

    fn save(&self) {
        let file = Self::file();

        if let Some(folder) = file.parent() {
            let _ = fs::create_dir_all(folder);
        }

        let text = format!(
            "wallpaper={}\nscheme={}\ndark={}\ndirectory={}\n",
            self.wallpaper,
            self.scheme,
            self.dark,
            self.directory.display()
        );

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
