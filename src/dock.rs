use std::process::{Command, Stdio};
use std::thread;
use std::time::Duration;

use amane::{
    Apps, Button, Center, DesktopApp, Horizontal, Image, Layer, LayerWindow, Margin, Monitor,
    Parent, Pointer, Rectangle, Row, Service, Shadow, Vertical, Widget, Zone,
};

use crate::m3;
use crate::motion;
use crate::overlay::Overlay;
use crate::settings::Settings;
use crate::theme::{Scheme, Theme, fade};

const ACTION_ICON: f32 = 42.0;
const MAGNIFICATION: f32 = 1.32;

#[derive(Default, PartialEq, Eq)]
struct RunningApps {
    app_ids: Vec<String>,
}

impl Service for RunningApps {
    fn new() -> Self {
        Self::default()
    }

    fn listen() {
        loop {
            let fresh = read_running_app_ids();
            let changed = Self::read().app_ids != fresh;

            if changed {
                Self::write().app_ids = fresh;
                crate::ui::Ui::refresh();
            }

            thread::sleep(Duration::from_secs(2));
        }
    }
}

fn read_running_app_ids() -> Vec<String> {
    let output = Command::new("niri")
        .args(["msg", "--json", "windows"])
        .stdout(Stdio::piped())
        .stderr(Stdio::null())
        .output();

    let Ok(output) = output else { return Vec::new() };
    if !output.status.success() {
        return Vec::new();
    }

    let json = String::from_utf8_lossy(&output.stdout);
    let mut ids = Vec::new();

    for app_id in json_string_values(&json, "app_id") {
        if !app_id.is_empty() && !ids.iter().any(|existing| existing == &app_id) {
            ids.push(app_id);
        }
    }

    ids
}

fn json_string_values(input: &str, key: &str) -> Vec<String> {
    let marker = format!("\"{key}\"");
    let mut values = Vec::new();
    let mut offset = 0;

    while let Some(found) = input[offset..].find(&marker) {
        let key_end = offset + found + marker.len();
        let Some(colon) = input[key_end..].find(':') else { break };
        let mut index = key_end + colon + 1;

        while input.as_bytes().get(index).is_some_and(u8::is_ascii_whitespace) {
            index += 1;
        }

        if input.as_bytes().get(index) != Some(&b'"') {
            offset = index.saturating_add(1);
            continue;
        }

        index += 1;
        let mut value = String::new();
        let mut chars = input[index..].char_indices();
        let mut consumed = 0;
        let mut closed = false;

        while let Some((byte, character)) = chars.next() {
            consumed = byte + character.len_utf8();

            match character {
                '"' => {
                    closed = true;
                    break;
                }
                '\\' => {
                    let Some((next_byte, escaped)) = chars.next() else { break };
                    consumed = next_byte + escaped.len_utf8();
                    match escaped {
                        '"' => value.push('"'),
                        '\\' => value.push('\\'),
                        '/' => value.push('/'),
                        'n' => value.push('\n'),
                        'r' => value.push('\r'),
                        't' => value.push('\t'),
                        _ => value.push(escaped),
                    }
                }
                other => value.push(other),
            }
        }

        if closed {
            values.push(value);
        }

        offset = index.saturating_add(consumed).max(key_end + 1);
    }

    values
}

fn app_matches_id(app: &DesktopApp, app_id: &str) -> bool {
    let normalize = |value: &str| {
        let value = value.to_lowercase();
        let value = value.rsplit('/').next().unwrap_or(&value);
        value.strip_suffix(".desktop").unwrap_or(value).to_string()
    };

    let target = normalize(app_id);
    let name = normalize(app.name());
    let exec = app.exec().split_whitespace().next().unwrap_or("");
    let exec = normalize(exec);

    target == name || target == exec
        || (!target.is_empty() && (target.contains(&name) || name.contains(&target)))
}

pub fn view(_monitor: &Monitor) -> LayerWindow {
    crate::ui::Ui::watch();

    let settings = Settings::read();
    let mode = settings.shell_mode();
    let position = String::from(settings.dock_position());
    let alignment = String::from(settings.dock_alignment());
    let dock_enabled = settings.flag("show_dock");
    drop(settings);

    if !dock_enabled || (mode != 2 && mode != 3) {
        return LayerWindow::new()
            .width(1.0)
            .height(1.0)
            .visible(false)
            .namespace("cds-dock");
    }

    let theme = Theme::read();
    let scheme = theme.scheme();
    let apps = Apps::read();
    let settings = Settings::read();
    let pinned = settings.pinned_apps().to_vec();
    let icon_size = settings.dock_icon_size() as f32;
    let magnification = settings.flag("dock_magnification");
    drop(settings);

    let running_ids = RunningApps::read().app_ids.clone();
    let mut items: Vec<(DesktopApp, bool)> = Vec::new();

    for exec in &pinned {
        if let Some(app) = apps.list().iter().find(|app| app.exec() == exec) {
            let running = running_ids.iter().any(|id| app_matches_id(app, id));
            items.push((app.clone(), running));
        }
    }

    for app_id in &running_ids {
        if let Some(app) = apps.list().iter().find(|app| app_matches_id(app, app_id)) {
            if !items.iter().any(|(existing, _)| existing.exec() == app.exec()) {
                items.push((app.clone(), true));
            }
        }
    }
    drop(apps);

    // Slots and the reserved layer both grow with the configured icon size.
    // This keeps large icons from being clipped and makes the slider visible.
    let slot = (icon_size * MAGNIFICATION + 20.0).max(64.0);
    let dock_height = (icon_size * MAGNIFICATION + 40.0).max(96.0);
    let dock_width = (items.len() as f32 + 1.0) * (slot + 4.0) + 16.0;

    let mut children: Vec<Box<dyn Widget>> = items
        .iter()
        .map(|(app, running)| {
            Box::new(app_item(app, scheme, icon_size, magnification, *running, slot, dock_height))
                as Box<dyn Widget>
        })
        .collect();

    children.push(Box::new(action_item(
        "dock:trash",
        m3::DELETE,
        scheme,
        slot,
        dock_height,
        || amane::spawn("xdg-open trash:///"),
    )));

    let bar = Rectangle::new()
        .width(dock_width)
        .height(dock_height - 12.0)
        .fill(scheme.surface_container_high)
        .radius(30.0)
        .clip()
        .border(1.0, fade(scheme.outline_variant, 90))
        .shadow(
            Shadow::drop(scheme.shadow)
                .opacity(0.34)
                .blur(28.0)
                .offset(0.0, 8.0),
        )
        .padding(8.0)
        .child(
            Row::new(children)
                .gap(4.0)
                .align(Center)
                .justify(Center)
                .width(Parent)
                .height(Parent),
        );

    let horizontal = match alignment.as_str() {
        "left" => Horizontal::Left,
        "right" => Horizontal::Right,
        _ => Horizontal::Middle,
    };

    let margin = if position == "top" {
        Margin { top: 12, ..Margin::default() }
    } else {
        Margin { bottom: 14, ..Margin::default() }
    };

    LayerWindow::new()
        .width(dock_width)
        .height(dock_height)
        .anchor_vertical(if position == "top" { Vertical::Top } else { Vertical::Bottom })
        .anchor_horizontal(horizontal)
        .layer(Layer::Top)
        .space(Zone::Reserve)
        .margin(margin)
        .namespace("cds-dock")
        .visible(true)
        .child(
            Rectangle::new()
                .width(Parent)
                .height(Parent)
                .align_child(Center, Center)
                .child(bar),
        )
}

fn app_item(
    app: &DesktopApp,
    scheme: &Scheme,
    icon_base: f32,
    magnification: bool,
    running: bool,
    slot: f32,
    dock_height: f32,
) -> Rectangle {
    let exec = String::from(app.exec());
    let key = format!("dock:{}", exec);
    let hover = motion::hover(&key);
    let scale = motion::number(
        &format!("{key}:scale"),
        if magnification && hover > 0.5 { MAGNIFICATION } else { 1.0 },
        motion::style_duration(180, 130),
    );
    let size = icon_base * scale;
    let owned = key.clone();
    let launch = exec.clone();

    let icon: Box<dyn Widget> = match app.icon_path() {
        Some(path) if Image::loaded(path) => Box::new(
            Rectangle::new()
                .width(size)
                .height(size)
                .radius(13.0)
                .clip()
                .fill(Image::contain(path).thumbnail(128, 128)),
        ),
        _ => Box::new(
            Rectangle::new()
                .width(size)
                .height(size)
                .radius(13.0)
                .fill(scheme.primary_container)
                .align_child(Center, Center)
                .child(m3::icon(
                    crate::glyphs::application(app.name(), app.exec()),
                    size * 0.58,
                    scheme.on_primary_container,
                )),
        ),
    };

    let mut layers = vec![icon];
    if running {
        layers.push(Box::new(
            Rectangle::new()
                .width(7.0)
                .height(7.0)
                .radius(4.0)
                .fill(scheme.primary)
                .border(1.0, scheme.surface_container_high)
                .translate(0.0, size * 0.5 - 3.0),
        ));
    }

    Rectangle::new()
        .width(slot)
        .height(dock_height - 28.0)
        .cursor(Pointer)
        .on_hover(move |inside| motion::set_hover(owned.clone(), inside))
        .on_click(move |button| match button {
            Button::Left => {
                amane::spawn(&launch);
                Overlay::close();
            }
            Button::Right => Settings::toggle_pinned(&launch),
            _ => {}
        })
        .align_child(Center, Center)
        .child(amane::Stack::new(layers).width(size).height(size))
}

fn action_item(
    key: &'static str,
    glyph: &str,
    scheme: &Scheme,
    slot: f32,
    dock_height: f32,
    action: impl Fn() + 'static,
) -> Rectangle {
    let hover = motion::hover(key);
    let scale = motion::number(
        &format!("{key}:scale"),
        if hover > 0.5 { 1.18 } else { 1.0 },
        motion::style_duration(160, 120),
    );
    let owned = String::from(key);

    Rectangle::new()
        .width(slot)
        .height(dock_height - 28.0)
        .cursor(Pointer)
        .on_hover(move |inside| motion::set_hover(owned.clone(), inside))
        .on_click(move |button| {
            if button == Button::Left {
                action();
            }
        })
        .align_child(Center, Center)
        .child(
            Rectangle::new()
                .width(ACTION_ICON * scale)
                .height(ACTION_ICON * scale)
                .radius(14.0)
                .fill(scheme.surface_container_highest)
                .align_child(Center, Center)
                .child(m3::icon(glyph, ACTION_ICON * 0.58 * scale, scheme.on_surface)),
        )
}
