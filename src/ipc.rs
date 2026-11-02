/*
 * the ipc surface. short, plain names, no "overlay" namespace to greet:
 *
 *   amane ipc call open launcher|apps|spotlight|clipboard|power|control|config
 *   amane ipc call closed
 *   amane ipc call dash [home|media|clima|papeis|alertas]
 *   amane ipc call tray [toggle|close|list|focus <id>|close-window <id>]
 *   amane ipc call settings [wallpaper|wallpapers|dark|scheme|directory|location|refresh]
 *   amane ipc call weather
 *   amane ipc call theme ...
 *
 * the old "island"/"overlay" names survive as thin aliases so old
 * keybinds keep working, but the docs now point at the short ones.
 */
use amane::Service;

use crate::overlay::{Overlay, Panel};
use crate::ui::Ui;

// one handler per registered name; amane passes only what follows it

pub fn open(arguments: &[String]) -> String {
    show(arguments)
}

pub fn closed(_arguments: &[String]) -> String {
    Overlay::close();

    Ui::close();

    crate::tray::Tray::close();

    String::from("ok")
}

pub fn dash(arguments: &[String]) -> String {
    toggle_dash(arguments)
}

pub fn settings(arguments: &[String]) -> String {
    settings_verb(arguments)
}

// the dispatcher under one name, useful for scripting the whole surface
pub fn root(arguments: &[String]) -> String {
    let Some(action) = arguments.first() else {
        return String::from(
            "usage: open launcher|apps|spotlight|clipboard|power|control|config | closed | dash [aba] | tray ... | settings ... | weather | theme ...",
        );
    };

    let rest = &arguments[1..];

    match action.as_str() {
        "open" => show(rest),
        "closed" | "close" => closed(&[]),
        "dash" => toggle_dash(rest),
        "tray" => crate::tray::command(rest),
        "settings" => settings_verb(rest),
        "weather" | "clima" => crate::weather::command(rest),
        "theme" => crate::theme::command(rest),
        // the panel may name itself: `open clipboard` == `clipboard`
        "launcher" | "apps" | "spotlight" | "clipboard" | "power" | "control" | "config" => {
            show(arguments)
        }
        _ => format!("failed: unknown command {action}"),
    }
}

/*
 * the panels, by simple name. each toggle opens its island; asking for
 * the one already open steps it down.
 */
fn show(arguments: &[String]) -> String {
    let Some(name) = arguments.first() else {
        return String::from(
            "usage: open launcher|apps|spotlight|clipboard|power|control|config",
        );
    };

    let panel = match name.as_str() {
        "launcher" | "menu" => Panel::Launcher,
        "apps" | "grade" => Panel::Apps,
        "spotlight" => Panel::Spotlight,
        "clipboard" | "clip" => Panel::Clipboard,
        "power" => Panel::Power,
        "control" => Panel::Control,
        "config" | "settings" => Panel::Config,
        _ => return format!("failed: unknown panel {name}"),
    };

    Overlay::toggle(panel);

    String::from("ok")
}

fn toggle_dash(arguments: &[String]) -> String {
    let [action, rest @ ..] = arguments else {
        Ui::toggle();

        return String::from("ok");
    };

    match action.as_str() {
        "toggle" => {
            Ui::toggle();

            String::from("ok")
        }
        "close" => {
            Ui::close();

            String::from("ok")
        }
        "open" | "aba" => {
            Ui::open(crate::ui::Tab::parse(rest.first().map(String::as_str)));

            String::from("ok")
        }
        // `dash clima` opens the dash on the tab directly
        name => {
            if name == "tray" || name == "toggle" {
                Ui::toggle();

                return String::from("ok");
            }

            Ui::open(crate::ui::Tab::parse(Some(name)));

            String::from("ok")
        }
    }
}

/*
 * the preferences themselves. the config panel uses these same verbs,
 * so the ipc and the visual clicks stay in step.
 */
fn settings_verb(arguments: &[String]) -> String {
    use crate::settings::{SCHEMES, Settings};

    const USAGE: &str = "usage: settings wallpaper [image] | wallpapers | dark [true|false|toggle] | scheme [name] | directory [path] | location [lat,lon|auto] | refresh";

    let [action, rest @ ..] = arguments else {
        return String::from(USAGE);
    };

    let rest: Vec<&str> = rest.iter().map(String::as_str).collect();

    match action.as_str() {
        "wallpaper" => match rest.first() {
            Some(path) => Settings::set_wallpaper(path),
            None => format!("wallpaper={}", Settings::read().wallpaper()),
        },

        "wallpapers" => Settings::read().wallpapers().join("\n"),

        "dark" => {
            let offered = rest.first().copied().unwrap_or("");

            let dark = match offered {
                "true" | "on" => true,
                "false" | "off" => false,
                "toggle" => !Settings::read().dark(),
                _ => return String::from("usage: settings dark [true|false|toggle]"),
            };

            Settings::set_dark(dark);

            String::from("ok")
        }

        "scheme" => {
            let Some(name) = rest.first() else {
                return format!(
                    "scheme={} {}",
                    Settings::read().scheme_name(),
                    schemePretty(),
                );
            };

            let matches = SCHEMES
                .iter()
                .any(|(scheme, _)| *scheme == *name)
                || SCHEMES
                    .iter()
                    .any(|(_, pretty)| pretty.to_lowercase() == name.to_lowercase());

            if !matches {
                return format!("failed: schemes are {prettyList}", prettyList = schemePretty());
            }

            // accept the pretty name from the panel by translating it
            let scheme_name = if SCHEMES.iter().any(|(scheme, _)| *scheme == *name) {
                String::from(*name)
            } else {
                SCHEMES
                    .iter()
                    .find(|(_, pretty)| pretty.to_lowercase() == name.to_lowercase())
                    .map(|(scheme, _)| String::from(*scheme))
                    .unwrap_or_default()
            };

            Settings::set_scheme(&scheme_name)
        }

        "directory" => match rest.join(" ").trim() {
            "" => format!("directory={}", Settings::read().directory_path()),
            path => Settings::set_directory(path),
        },

        "location" => match rest.join(" ").trim() {
            "" => format!("location={}", Settings::read().location()),
            "auto" | "ip" => Settings::set_location(""),
            spot => Settings::set_location(spot),
        },

        "refresh" => Settings::refresh(),

        _ => String::from(USAGE),
    }
}

fn schemePretty() -> String {
    use crate::settings::SCHEMES;

    SCHEMES
        .iter()
        .map(|(scheme, _)| scheme.replace("scheme-", ""))
        .collect::<Vec<_>>()
        .join(", ")
}

// ---- the old names, kept running as aliases --------------------------------

pub fn island_legacy(arguments: &[String]) -> String {
    let Some(action) = arguments.first() else {
        return String::from("alias: island toggle|close|open [aba]|tray");
    };

    let rest = &arguments[1..];

    match action.as_str() {
        "toggle" => {
            Ui::toggle();

            String::from("ok")
        }
        "close" => {
            Ui::close();

            crate::tray::Tray::close();

            String::from("ok")
        }
        "open" => dash(rest),
        // the tray never lived under island for views, only the toggle
        "tray" => crate::tray::command(rest),
        _ => format!("failed: unknown island verb {action}"),
    }
}

pub fn overlay_legacy(arguments: &[String]) -> String {
    let Some(panel) = arguments.first() else {
        return String::from("alias: overlay launcher|clipboard|power|settings|close");
    };

    match panel.as_str() {
        "close" => {
            Overlay::close();

            String::from("ok")
        }
        "settings" | "config" => show(&[String::from("config")]),
        other => show(&[String::from(other)]),
    }
}
