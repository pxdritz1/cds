use amane::{Color, Service, watch_file};
use std::env;
use std::fs;
use std::path::PathBuf;
use std::process::{Command, Stdio};
use std::sync::{Mutex, OnceLock};
use std::thread;
use std::time::{Duration, Instant};

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
                let mut scheme = Self {
                    $($role: Color::from($default),)*
                };

                // the settings file decides the fallback before matugen runs
                if configured_dark() {
                    for (role, hex) in DARK_ROLES {
                        scheme.assign(role, Color::from(*hex));
                    }
                }

                scheme
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
    // Primary colors - Material 3 Expressive
    primary = "#6750a4",
    on_primary = "#ffffff",
    primary_container = "#eaddff",
    on_primary_container = "#21005d",

    // Secondary colors
    secondary = "#625b71",
    on_secondary = "#ffffff",
    secondary_container = "#e8def8",
    on_secondary_container = "#1e192b",

    // Tertiary colors
    tertiary = "#7d5260",
    on_tertiary = "#ffffff",
    tertiary_container = "#ffd8e4",
    on_tertiary_container = "#31111d",

    // Error colors
    error = "#ba1a1a",
    on_error = "#ffffff",
    error_container = "#ffdad6",
    on_error_container = "#410002",

    // Background and surface colors
    background = "#fdf8ff",
    on_background = "#1c1b1f",
    surface = "#fdf8ff",
    on_surface = "#1c1b1f",
    surface_tint = "#6750a4",
    surface_dim = "#e7dff8",
    surface_bright = "#fdf8ff",
    surface_variant = "#e7dff8",
    on_surface_variant = "#49454f",
    surface_container_lowest = "#fdf8ff",
    surface_container_low = "#f7f2fb",
    surface_container = "#f0ecf7",
    surface_container_high = "#e7dff8",
    surface_container_highest = "#e0d8f3",

    // Outline colors
    outline = "#79747e",
    outline_variant = "#79747e",

    // Inverse colors
    inverse_surface = "#313033",
    inverse_on_surface = "#f4edf7",
    inverse_primary = "#d0bcff",

    // Effect colors
    scrim = "#000000",
    shadow = "#000000",
}

/*
 * material 3 roles that matugen does not emit, kept here so a dark
 * session without a generated palette still starts on dark surfaces
 */
const DARK_ROLES: &[(&str, &str)] = &[
    ("primary", "#cfbdfe"),
    ("on_primary", "#36275d"),
    ("primary_container", "#4d3d75"),
    ("on_primary_container", "#e9ddff"),
    ("secondary", "#cbc2db"),
    ("on_secondary", "#332d41"),
    ("secondary_container", "#4a4458"),
    ("on_secondary_container", "#e8def8"),
    ("tertiary", "#efb8c8"),
    ("on_tertiary", "#4a2532"),
    ("tertiary_container", "#633b48"),
    ("on_tertiary_container", "#ffd9e3"),
    ("error", "#ffb4ab"),
    ("on_error", "#690005"),
    ("error_container", "#93000a"),
    ("on_error_container", "#ffdad6"),
    ("background", "#141218"),
    ("on_background", "#e6e0e9"),
    ("surface", "#141218"),
    ("on_surface", "#e6e0e9"),
    ("surface_tint", "#cfbdfe"),
    ("surface_dim", "#141218"),
    ("surface_bright", "#36343a"),
    ("surface_variant", "#49454e"),
    ("on_surface_variant", "#cac4cf"),
    ("surface_container_lowest", "#0f0d13"),
    ("surface_container_low", "#1d1b20"),
    ("surface_container", "#211f24"),
    ("surface_container_high", "#2b292f"),
    ("surface_container_highest", "#36343a"),
    ("outline", "#948f99"),
    ("outline_variant", "#49454e"),
    ("inverse_surface", "#e6e0e9"),
    ("inverse_on_surface", "#322f35"),
    ("inverse_primary", "#65558f"),
];

fn configured_dark() -> bool {
    let path = config_home().join("chloe-dynamic-shell/settings.conf");

    fs::read_to_string(path)
        .ok()
        .and_then(|text| {
            text.lines().find_map(|line| {
                line.strip_prefix("dark=")
                    .map(|value| value.trim() == "true")
            })
        })
        .unwrap_or(true)
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

        // matugen leaves these two out, so they follow the generated surface
        scheme.surface_dim = mix(scheme.surface, Color::BLACK, 0.05);
        scheme.surface_bright = mix(scheme.surface, Color::WHITE, 0.08);

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

// a color laid over another, like the state layers material uses on hover
pub fn mix(base: Color, over: Color, amount: f32) -> Color {
    let amount = amount.clamp(0.0, 1.0);

    let channel = |a: u8, b: u8| {
        let a = f32::from(a);
        let b = f32::from(b);

        (a + (b - a) * amount).round() as u8
    };

    Color::rgba(
        channel(base.red(), over.red()),
        channel(base.green(), over.green()),
        channel(base.blue(), over.blue()),
        channel(base.alpha(), over.alpha()),
    )
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
    String::from(
        "usage: theme set <image> [scheme] [dark|light] | theme color <hex> [scheme] [dark|light]",
    )
}

pub fn apply(source: &str, value: &str, scheme: &str, dark: bool) -> String {
    if value.contains('\'') {
        return String::from("failed: the value cannot contain quotes");
    }

    run_matugen(source, value, scheme, dark)
}

/*
 * the views change themes from the click handlers, where waiting for a
 * whole matugen run would freeze the bar; a single worker coalesces jobs and lets the file
 * watcher refresh the scheme when it lands
 */
type MatugenJob = (String, String, String, bool);

static MATUGEN_JOBS: OnceLock<std::sync::mpsc::Sender<MatugenJob>> = OnceLock::new();

pub fn apply_async(source: &str, value: &str, scheme: &str, dark: bool) {
    let sender = MATUGEN_JOBS.get_or_init(|| {
        let (sender, receiver) = std::sync::mpsc::channel::<MatugenJob>();

        if let Err(error) = thread::Builder::new()
            .name(String::from("cds-matugen"))
            .spawn(move || {
                while let Ok(mut job) = receiver.recv() {
                    // Only the newest queued appearance matters. Discard stale
                    // requests so rapid wallpaper/theme changes cannot build
                    // an unbounded queue of worker threads or old palettes.
                    while let Ok(newer) = receiver.try_recv() {
                        job = newer;
                    }

                    let (source, value, scheme, dark) = job;
                    let _ = run_matugen(&source, &value, &scheme, dark);
                }
            })
        {
            eprintln!("CDS: could not start the Matugen worker: {error}");
        }

        sender
    });

    let job = (
        source.to_owned(),
        value.to_owned(),
        scheme.to_owned(),
        dark,
    );

    if sender.send(job).is_err() {
        eprintln!("CDS: Matugen worker unavailable; appearance change skipped");
    }
}

static MATUGEN_LOCK: OnceLock<Mutex<()>> = OnceLock::new();

fn run_matugen(source: &str, value: &str, scheme: &str, dark: bool) -> String {
    if value.contains('\'') || value.contains('\n') {
        return String::from("failed: invalid matugen input");
    }

    let lock = MATUGEN_LOCK.get_or_init(|| Mutex::new(()));
    let _guard = match lock.lock() {
        Ok(guard) => guard,
        Err(poisoned) => poisoned.into_inner(),
    };

    let staged = colors_path().with_extension("colors.tmp");
    let Some(config) = write_matugen_config(&staged) else {
        return String::from("failed: could not write the matugen config");
    };

    let mode = if dark { "dark" } else { "light" };
    let mut command = Command::new("matugen");
    command.args([source, value, "-c"])
        .arg(&config)
        .args(["-t", scheme, "-m", mode, "-q"]);
    if source == "image" {
        command.args(["--prefer", "saturation"]);
    }
    command.arg("--continue-on-error")
        .stdout(Stdio::null())
        .stderr(Stdio::null());

    let mut child = match command.spawn() {
        Ok(child) => child,
        Err(_) => return String::from("failed: could not start matugen"),
    };

    let started = Instant::now();
    let status = loop {
        match child.try_wait() {
            Ok(Some(status)) => break Some(status),
            Ok(None) if started.elapsed() < Duration::from_secs(45) => {
                thread::sleep(Duration::from_millis(100));
            }
            _ => {
                let _ = child.kill();
                let _ = child.wait();
                let _ = fs::remove_file(&staged);
                return String::from("failed: matugen timed out or could not be checked");
            }
        }
    };

    if !status.is_some_and(|status| status.success()) {
        let _ = fs::remove_file(&staged);
        return String::from("failed: matugen could not generate the colors");
    }

    match fs::read_to_string(&staged) {
        Ok(text) if text.contains("primary=") => {}
        _ => {
            let _ = fs::remove_file(&staged);
            return String::from("failed: matugen finished without a valid palette");
        }
    }

    if fs::rename(&staged, colors_path()).is_err() {
        let _ = fs::remove_file(&staged);
        return String::from("failed: could not publish the new color palette");
    }

    String::from("ok")
}

/*
 * matugen only reads one config, so the shell's template is added to a
 * copy of the user's. that keeps alacritty, niri, spicetify and the rest
 * updating together with the shell colors.
 */
fn write_matugen_config(output: &std::path::Path) -> Option<PathBuf> {
    let folder = cache_home().join("chloe-dynamic-shell/matugen");
    fs::create_dir_all(&folder).ok()?;

    let template = folder.join("colors.tpl");
    let config = folder.join("config.toml");

    fs::write(&template, include_str!("../matugen/colors.tpl")).ok()?;

    let mut text = String::new();

    if let Ok(user) = fs::read_to_string(config_home().join("matugen/config.toml")) {
        text.push_str(&without_shell_section(&user));

        if !text.ends_with('\n') {
            text.push('\n');
        }
    }

    text.push_str(&format!(
        "\n[templates.cds_shell]\ninput_path = {}\noutput_path = {}\n",
        toml_string(&template),
        toml_string(output)
    ));

    let temporary = config.with_extension("toml.tmp");
    fs::write(&temporary, text).ok()?;
    fs::rename(&temporary, &config).ok()?;

    Some(config)
}

// drops an older copy of the shell section, so a second one cannot be added
fn without_shell_section(text: &str) -> String {
    let mut kept = String::new();
    let mut skipping = false;

    for line in text.lines() {
        let header = line.trim_start().starts_with('[');

        if header {
            skipping = line.trim() == "[templates.cds_shell]"
                || line.trim() == "[templates.shell]"
                || line.trim() == "[templates.chloe_dynamic_shell]";
        }

        if !skipping {
            kept.push_str(line);
            kept.push('\n');
        }
    }

    kept
}

fn toml_string(path: &std::path::Path) -> String {
    format!("\"{}\"", path.display().to_string().replace('\\', "\\\\"))
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

    // A missing HOME should not panic the shell during startup. Fall back
    // to the current directory, then /tmp if the process has no cwd either.
    let home = env::var_os("HOME")
        .map(PathBuf::from)
        .or_else(|| env::current_dir().ok())
        .unwrap_or_else(|| PathBuf::from("/tmp"));

    home.join(fallback)
}
