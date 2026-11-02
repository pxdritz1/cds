use std::env;
use std::fs;
use std::path::PathBuf;
use amane::{Color, Service, watch_file};

pub struct Typography {
    pub body: String,
    pub icon: String,
}

impl Typography {
    pub fn new() -> Self {
        Self {
            body: String::from("Google Sans Flex"),
            icon: String::from("Material Symbols Rounded"),
        }
    }
}

macro_rules! scheme {
    ($($role:ident = $default:literal,)*) => {
        pub struct Scheme {
            $(pub $role: Color,)*
        }

        impl Scheme {
            fn baseline() -> Self {
                Self {
                    $($role: Color::from($default),)*
                }
            }

            fn assign(&mut self, role: &str, color: Color) {
                match role {
                    $(stringify!($role) => self.$role = color,)*
                    _ => {}
                }
            }
        }
    };
}

scheme! {
    primary = "#cfbdfe",
    on_primary = "#36275d",
    primary_container = "#4d3d75",
    on_primary_container = "#e9ddff",
    secondary = "#cbc2db",
    on_secondary = "#332d41",
    secondary_container = "#4a4458",
    on_secondary_container = "#e8def8",
    tertiary = "#efb8c8",
    on_tertiary = "#4a2532",
    tertiary_container = "#633b48",
    on_tertiary_container = "#ffd9e3",
    error = "#ffb4ab",
    on_error = "#690005",
    error_container = "#93000a",
    on_error_container = "#ffdad6",
    background = "#141218",
    on_background = "#e6e0e9",
    surface = "#141218",
    on_surface = "#e6e0e9",
    surface_dim = "#141218",
    surface_bright = "#36343a",
    surface_variant = "#49454e",
    on_surface_variant = "#cac4cf",
    surface_container_lowest = "#0f0d13",
    surface_container_low = "#1d1b20",
    surface_container = "#211f24",
    surface_container_high = "#2b292f",
    surface_container_highest = "#36343a",
    outline = "#948f99",
    outline_variant = "#49454e",
    inverse_surface = "#e6e0e9",
    inverse_on_surface = "#322f35",
    inverse_primary = "#65558f",
    scrim = "#000000",
    shadow = "#000000",
}

impl Scheme {
    fn load() -> Self {
        let mut scheme = Self::baseline();
        let Ok(text) = fs::read_to_string(colors_path()) else {
            return scheme;
        };

        for line in text.lines() {
            if let Some((role, hex)) = line.split_once('=') {
                scheme.assign(role.trim(), Color::from(hex.trim()));
            }
        }

        scheme
    }
}

pub struct Theme {
    scheme: Scheme,
    typography: Typography,
}

impl Service for Theme {
    fn new() -> Self {
        Self {
            scheme: Scheme::load(),
            typography: Typography::new(),
        }
    }

    fn listen() {
        let path = colors_path();

        if let Some(folder) = path.parent() {
            let _ = fs::create_dir_all(folder);
        }

        for _ in watch_file(&path.to_string_lossy()) {
            let scheme = Scheme::load();
            Self::write().scheme = scheme;
        }
    }
}

impl Theme {
    pub fn scheme(&self) -> &Scheme {
        &self.scheme
    }

    pub fn typography(&self) -> &Typography {
        &self.typography
    }
}

pub fn fade(color: Color, alpha: u8) -> Color {
    Color::rgba(color.red(), color.green(), color.blue(), alpha)
}

pub fn command(arguments: &[String]) -> String {
    let [action, value, options @ ..] = arguments else {
        return usage();
    };

    let scheme = options.first().map_or("scheme-tonal-spot", String::as_str);
    let dark = options.get(1).is_none_or(|mode| mode != "light");

    match action.as_str() {
        "set" => apply("image", value, scheme, dark),
        "color" => apply("color hex", value, scheme, dark),
        _ => usage(),
    }
}

fn usage() -> String {
    String::from("usage: theme set <image> [scheme] [dark|light] | theme color <hex> [scheme] [dark|light]")
}

pub fn apply(source: &str, value: &str, scheme: &str, dark: bool) -> String {
    if value.contains('\'') {
        return String::from("failed: the value cannot contain quotes");
    }

    let Some(config) = write_matugen_config() else {
        return String::from("failed: could not write the matugen config");
    };

    let mode = if dark { "dark" } else { "light" };

    amane::spawn(&format!(
        "matugen {source} '{value}' -c '{}' -t {scheme} -m {mode} -q",
        config.display()
    ));

    String::from("ok")
}

fn write_matugen_config() -> Option<PathBuf> {
    let folder = config_home().join("chloe-dynamic-shell/matugen");
    fs::create_dir_all(&folder).ok()?;

    let template = folder.join("colors.tpl");
    let config = folder.join("config.toml");

    fs::write(&template, include_str!("../matugen/colors.tpl")).ok()?;

    let text = format!(
        "[config]\n\n[templates.shell]\ninput_path = \"{}\"\noutput_path = \"{}\"\n",
        template.display(),
        colors_path().display()
    );

    fs::write(&config, text).ok()?;

    Some(config)
}

pub fn colors_path() -> PathBuf {
    cache_home().join("chloe-dynamic-shell/colors.conf")
}

pub fn cache_home() -> PathBuf {
    home_directory("XDG_CACHE_HOME", ".cache")
}

pub fn config_home() -> PathBuf {
    home_directory("XDG_CONFIG_HOME", ".config")
}

pub fn home_directory(variable: &str, fallback: &str) -> PathBuf {
    if let Some(path) = env::var_os(variable) {
        return PathBuf::from(path);
    }

    let home = env::var_os("HOME").expect("failed to find home: HOME is not set");
    PathBuf::from(home).join(fallback)
}
