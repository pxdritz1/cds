use amane::{Link, Network};

pub const WIFI_1: &str = "\u{e4ca}";
pub const WIFI_2: &str = "\u{e4d9}";
pub const WIFI_3: &str = "\u{e63e}";
pub const WIFI_OFF: &str = "\u{e648}";
pub const ETHERNET: &str = "\u{e8be}";

pub const BLUETOOTH: &str = "\u{e1a7}";
pub const BLUETOOTH_CONNECTED: &str = "\u{e1a8}";
pub const BLUETOOTH_OFF: &str = "\u{e1a9}";
pub const BLUETOOTH_SEARCHING: &str = "\u{e1aa}";

pub const VOLUME_OFF: &str = "\u{e04f}";
pub const VOLUME_MUTE: &str = "\u{e04e}";
pub const VOLUME_DOWN: &str = "\u{e04d}";
pub const VOLUME_UP: &str = "\u{e050}";

pub const BRIGHTNESS_LOW: &str = "\u{e1ad}";
pub const BRIGHTNESS_MEDIUM: &str = "\u{e1ae}";
pub const BRIGHTNESS_HIGH: &str = "\u{e1ac}";

pub const MIC: &str = "\u{e029}";
pub const MIC_OFF: &str = "\u{e02b}";

const BATTERY_BARS: [&str; 7] = [
    "\u{ebdc}", "\u{ebd9}", "\u{ebe0}", "\u{ebdd}", "\u{ebe2}", "\u{ebd4}", "\u{ebd2}",
];
const BATTERY_ALERT: &str = "\u{e19c}";
const BATTERY_FULL: &str = "\u{e1a4}";
const CHARGING_20: &str = "\u{f0a2}";
const CHARGING_50: &str = "\u{f0a4}";
const CHARGING_80: &str = "\u{f0a6}";
const CHARGING_FULL: &str = "\u{e1a3}";

pub fn wifi(network: &Network) -> &'static str {
    if !network.wifi_enabled() && network.link() != Link::Wired {
        return WIFI_OFF;
    }

    match network.link() {
        Link::Wired => ETHERNET,
        Link::Offline => WIFI_OFF,
        _ => wifi_level(network.strength()),
    }
}

pub fn wifi_level(strength: u8) -> &'static str {
    match strength {
        0..=33 => WIFI_1,
        34..=66 => WIFI_2,
        _ => WIFI_3,
    }
}

pub fn bluetooth(available: bool, powered: bool, connected: bool, scanning: bool) -> &'static str {
    if !available || !powered {
        BLUETOOTH_OFF
    } else if scanning {
        BLUETOOTH_SEARCHING
    } else if connected {
        BLUETOOTH_CONNECTED
    } else {
        BLUETOOTH
    }
}

pub fn volume(percent: u8, muted: bool) -> &'static str {
    if muted {
        VOLUME_OFF
    } else if percent == 0 {
        VOLUME_MUTE
    } else if percent < 50 {
        VOLUME_DOWN
    } else {
        VOLUME_UP
    }
}

pub fn brightness(percent: u8) -> &'static str {
    match percent {
        0..=33 => BRIGHTNESS_LOW,
        34..=66 => BRIGHTNESS_MEDIUM,
        _ => BRIGHTNESS_HIGH,
    }
}

pub fn microphone(muted: bool) -> &'static str {
    if muted { MIC_OFF } else { MIC }
}

pub fn battery(percent: u8, charging: bool) -> &'static str {
    if charging {
        return match percent {
            0..=34 => CHARGING_20,
            35..=64 => CHARGING_50,
            65..=94 => CHARGING_80,
            _ => CHARGING_FULL,
        };
    }

    if percent <= 10 {
        return BATTERY_ALERT;
    }

    if percent >= 96 {
        return BATTERY_FULL;
    }

    let bar = (usize::from(percent) * 7 / 100).min(6);

    BATTERY_BARS[bar]
}

/// Pick a useful Material Symbols fallback when a desktop entry has no
/// resolvable GTK/theme icon. Match both the visible name and executable ID.
pub fn application(name: &str, exec: &str) -> &'static str {
    let name = name.to_ascii_lowercase();
    let exec = exec.to_ascii_lowercase();
    let matches = |terms: &[&str]| terms.iter().any(|term| name.contains(term) || exec.contains(term));

    if matches(&["firefox", "chrom", "browser", "brave", "vivaldi", "opera", "web"]) {
        "\u{e80b}" // public / web
    } else if matches(&["terminal", "console", "alacritty", "kitty", "foot", "wezterm", "ptyxis"]) {
        "\u{e31a}" // terminal
    } else if matches(&["file", "nautilus", "dolphin", "thunar", "nemo", "directory"]) {
        "\u{e2c7}" // folder
    } else if matches(&["music", "spotify", "rhythmbox", "audacious", "player"]) {
        "\u{e405}" // music
    } else if matches(&["video", "vlc", "mpv", "celluloid", "kdenlive"]) {
        "\u{e04b}" // play
    } else if matches(&["chat", "discord", "telegram", "signal", "slack", "matrix", "messenger"]) {
        "\u{e0b7}" // chat
    } else if matches(&["code", "studio", "editor", "vim", "neovim", "zed", "codium", "idea"]) {
        "\u{e86f}" // code
    } else if matches(&["settings", "config", "control", "tweak"]) {
        "\u{e8b8}" // settings
    } else if matches(&["photo", "image", "gimp", "krita", "inkscape", "drawing"]) {
        "\u{e3f4}" // image
    } else if matches(&["camera", "screenshot"]) {
        "\u{e412}" // camera
    } else if matches(&["document", "writer", "office", "libreoffice", "word", "text"]) {
        "\u{e24d}" // description
    } else {
        "\u{e5c3}" // apps
    }
}
