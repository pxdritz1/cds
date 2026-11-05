/*
 * the ipc surface. short, plain names, no "overlay" namespace to greet:
 *
 *   amane ipc call open launcher|apps|spotlight|clipboard|power|control|config|color|emoji
 *   amane ipc call closed
 *   amane ipc call dash [home|media|clima|papeis|alertas]
 *   amane ipc call tray [toggle|close|list|focus <id>|close-window <id>]
 *   amane ipc call mode [islands|dock|toggle|status]\n *   amane ipc call settings [wallpaper|wallpapers|dark|scheme|directory|location|mode|layout|motion|dock-position|dock-alignment|position|clock|refresh]
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

// locks the session with the lock screen registered on App
pub fn open_lock(_arguments: &[String]) -> String {
    amane::Lock::start();

    String::from("ok")
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

/// Switch the complete shell between its two mutually exclusive interfaces.
pub fn mode(arguments: &[String]) -> String {
    use crate::settings::Settings;

    match arguments.first().map(String::as_str) {
        Some("islands") | Some("island") => {
            Settings::set_shell_mode(1);
            String::from("ok: mode=islands")
        }
        Some("dock") => {
            Settings::set_shell_mode(2);
            String::from("ok: mode=dock")
        }
        Some("toggle") => {
            let next = if Settings::read().shell_mode() == 1 { 2 } else { 1 };
            Settings::set_shell_mode(next);
            format!("ok: mode={}", if next == 1 { "islands" } else { "dock" })
        }
        Some("status") | None => format!("mode={}", if Settings::read().shell_mode() == 1 { "islands" } else { "dock" }),
        Some(other) => format!("failed: unknown UI mode {other}; use islands|dock|toggle|status"),
    }
}


// the dispatcher under one name, useful for scripting the whole surface
pub fn root(arguments: &[String]) -> String {
    let Some(action) = arguments.first() else {
        return String::from(
            "usage: open launcher|apps|spotlight|clipboard|power|control|config | closed | dash [tab] | mode [islands|dock|toggle|status] | tray ... | settings ... | weather | theme ...",
        );
    };

    let rest = &arguments[1..];

    match action.as_str() {
        "open" => show(rest),
        "closed" | "close" => closed(&[]),
        // locks the session with the lock screen registered on App
        "lock" => open_lock(&[]),
        "dash" => toggle_dash(rest),
        "tray" => crate::tray::command(rest),
        "settings" => settings_verb(rest),
        "mode" | "ui" => mode(rest),
        "weather" | "clima" => crate::weather::command(rest),
        "theme" => crate::theme::command(rest),
        // the panel may name itself: `open clipboard` == `clipboard`
        "launcher" | "apps" | "spotlight" | "clipboard" | "power" | "control" | "config" | "color" | "emoji" => {
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
        return String::from("usage: open launcher|apps|spotlight|clipboard|power|control|config|color|emoji");
    };

    let panel = match name.as_str() {
        "launcher" | "menu" => Panel::Launcher,
        "apps" | "grade" => Panel::Apps,
        "spotlight" => Panel::Spotlight,
        "clipboard" | "clip" => Panel::Clipboard,
        "power" => Panel::Power,
        "control" => Panel::Control,
        "config" | "settings" => Panel::Config,
        "color" | "color-picker" => Panel::Color,
        "emoji" | "emoji-picker" => Panel::Emoji,
        _ => return format!("failed: unknown panel {name}"),
    };

    if Overlay::read().panel() != panel {
        match panel {
            Panel::Color => crate::color::ColorPicker::reset(),
            Panel::Emoji => crate::emoji::EmojiPicker::reset(),
            _ => {}
        }
    }

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

    const USAGE: &str = "usage: settings wallpaper [image] | wallpapers | dark [true|false|toggle] | scheme [name] | directory [path] | location [lat,lon|auto] | mode [islands|dock] | layout [1|2] | motion [m3|fluent] | dock-position [top|bottom] | dock-alignment [left|center|right] | position <workspaces|center|status> <alignment> | clock [digital|analog] | refresh";

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

            Settings::set_dark(dark)
        }

        "scheme" => {
            let Some(name) = rest.first() else {
                return format!(
                    "scheme={} {}",
                    Settings::read().scheme_name(),
                    schemePretty(),
                );
            };

            let matches = SCHEMES.iter().any(|(scheme, _)| *scheme == *name)
                || SCHEMES
                    .iter()
                    .any(|(_, pretty)| pretty.to_lowercase() == name.to_lowercase());

            if !matches {
                return format!(
                    "failed: schemes are {prettyList}",
                    prettyList = schemePretty()
                );
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

        "mode" => match rest.first().copied() {
            None => format!("mode={}", if Settings::read().shell_mode() == 1 { "islands" } else { "dock" }),
            Some("islands" | "island" | "1") => { Settings::set_shell_mode(1); String::from("ok: mode=islands") },
            Some("dock" | "2" | "3") => { Settings::set_shell_mode(2); String::from("ok: mode=dock") },
            _ => String::from("usage: settings mode [islands|dock]"),
        },

        "layout" => match rest.first().copied() {
            None => format!("layout={}", Settings::read().shell_mode()),
            Some(value) => match value.parse::<u8>() {
                Ok(1) => { Settings::set_shell_mode(1); String::from("ok: mode=islands") }
                Ok(2 | 3) => { Settings::set_shell_mode(2); String::from("ok: mode=dock") }
                _ => String::from("usage: settings layout [1|2]"),
            },
        },

        "motion" => match rest.first().copied() {
            None => format!("motion={}", Settings::read().motion_style()),
            Some("m3") => {
                Settings::set_motion_style("m3");
                String::from("ok: motion=m3")
            }
            Some("fluent") => {
                Settings::set_motion_style("fluent");
                String::from("ok: motion=fluent")
            }
            _ => String::from("usage: settings motion [m3|fluent]"),
        },

        "dock-position" => match rest.first().copied() {
            None => format!("dock-position={}", Settings::read().dock_position()),
            Some(value @ ("top" | "bottom")) => {
                Settings::set_dock_position(value);
                format!("ok: dock-position={value}")
            }
            _ => String::from("usage: settings dock-position [top|bottom]"),
        },

        "dock-alignment" => match rest.first().copied() {
            None => format!("dock-alignment={}", Settings::read().dock_alignment()),
            Some(value @ ("left" | "center" | "right")) => {
                Settings::set_dock_alignment(value);
                format!("ok: dock-alignment={value}")
            }
            _ => String::from("usage: settings dock-alignment [left|center|right]"),
        },

        "position" => match rest.as_slice() {
            ["workspaces", alignment @ ("left" | "right")] => {
                Settings::set_workspaces_alignment(alignment);
                format!("ok: workspaces={alignment}")
            }
            ["center", alignment @ ("left" | "center" | "right")] => {
                Settings::set_center_alignment(alignment);
                format!("ok: center={alignment}")
            }
            ["status", alignment @ ("left" | "right")] => {
                Settings::set_status_alignment(alignment);
                format!("ok: status={alignment}")
            }
            [] => {
                let settings = Settings::read();
                format!(
                    "workspaces={}\ncenter={}\nstatus={}",
                    settings.workspaces_alignment(),
                    settings.center_alignment(),
                    settings.status_alignment(),
                )
            }
            _ => String::from("usage: settings position <workspaces left|right | center left|center|right | status left|right>"),
        },

        "clock" => match rest.first().copied() {
            None => format!(
                "clock={}",
                if Settings::read().flag("clock_analog") { "analog" } else { "digital" }
            ),
            Some("analog") => {
                Settings::set_flag("clock_analog", true);
                String::from("ok: clock=analog")
            }
            Some("digital") => {
                Settings::set_flag("clock_analog", false);
                String::from("ok: clock=digital")
            }
            _ => String::from("usage: settings clock [digital|analog]"),
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
