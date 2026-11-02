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

    pub fn scheme_name(&self) -> &str {
        self.scheme.as_str()
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

        theme::apply("image", &wallpaper, &scheme, dark)
    }

    pub fn set_dark(dark: bool) {
        let (wallpaper, scheme) = {
            let mut settings = Self::write();

            settings.dark = dark;
            settings.save();

            (settings.wallpaper.clone(), settings.scheme.clone())
        };

        if wallpaper.is_empty() {
            return;
        }

        theme::apply("image", &wallpaper, &scheme, dark);
    }

    // changes the matugen scheme and regenerates everything with it
    pub fn set_scheme(scheme: &str) -> String {
        let (wallpaper, dark) = {
            let mut settings = Self::write();

            settings.scheme = String::from(scheme);
            settings.save();

            (settings.wallpaper.clone(), settings.dark)
        };

        if wallpaper.is_empty() {
            return String::from("failed: set a wallpaper first");
        }

        theme::apply("image", &wallpaper, scheme, dark)
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

pub fn command(arguments: &[String]) -> String {
    let [action, rest @ ..] = arguments else {
        return usage();
    };

    match action.as_str() {
        "wallpaper" => match rest.first() {
            Some(path) => Settings::set_wallpaper(path),
            None => format!("wallpaper={}", Settings::read().wallpaper()),
        },

        "wallpapers" => Settings::read().wallpapers().join("\n"),

        "dark" => {
            let dark = match rest.first().map(String::as_str) {
                Some("true") | Some("on") => true,
                Some("false") | Some("off") => false,
                Some("toggle") => !Settings::read().dark(),
                _ => return usage(),
            };

            Settings::set_dark(dark);

            String::from("ok")
        }

        "scheme" => match rest.first().map(String::as_str) {
            Some(name) => Settings::set_scheme(name),
            None => format!("scheme={}", Settings::read().scheme),
        },

        // regenerates matugen for the shell and every external app template
        "refresh" => {
            let settings = Settings::read();

            let (wallpaper, scheme, dark) = (
                settings.wallpaper().to_string(),
                settings.scheme.clone(),
                settings.dark(),
            );

            drop(settings);

            if wallpaper.is_empty() {
                return String::from("failed: set a wallpaper first");
            }

            theme::apply("image", &wallpaper, &scheme, dark)
        }

        _ => usage(),
    }
}

fn usage() -> String {
    String::from(
        "usage: settings wallpaper [image|nothing] | wallpapers | dark [true|false|toggle] | scheme [name] | refresh",
    )
}
