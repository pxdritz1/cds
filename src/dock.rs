use std::path::PathBuf;
use std::process::{Command, Stdio};
use std::thread;
use std::time::Duration;

use amane::{
    Apps, Button, Canvas, Center, DesktopApp, Horizontal, Image, InputArea, Layer, LayerWindow, Margin, Monitor,
    End, Full, Parent, Path, Pointer, Rectangle, Row, Service, Shadow, Shape, Start, Text, Vertical, Widget,
    Zone,
};

use crate::m3;
use crate::motion;
use crate::overlay::Overlay;
use crate::settings::Settings;
use crate::theme::{Scheme, Theme, fade};

const ACTION_ICON: f32 = 42.0;
const MAGNIFICATION: f32 = 1.32;

#[derive(Clone, PartialEq, Eq)]
struct RunningWindow {
    id: u64,
    app_id: String,
}

#[derive(Default, PartialEq, Eq)]
struct RunningApps {
    windows: Vec<RunningWindow>,
}

impl Service for RunningApps {
    fn new() -> Self {
        Self::default()
    }

    fn listen() {
        loop {
            let fresh = read_running_windows();
            let changed = Self::read().windows != fresh;

            if changed {
                Self::write().windows = fresh;
                crate::ui::Ui::refresh();
            }

            thread::sleep(Duration::from_secs(2));
        }
    }
}

fn read_running_windows() -> Vec<RunningWindow> {
    let output = Command::new("niri")
        .args(["msg", "--json", "windows"])
        .stdout(Stdio::piped())
        .stderr(Stdio::null())
        .output();

    let Ok(output) = output else { return Vec::new() };
    if !output.status.success() { return Vec::new(); }

    let json = String::from_utf8_lossy(&output.stdout);
    let mut windows = Vec::new();
    for object in json_objects(&json) {
        let app_ids = json_string_values(object, "app_id");
        let ids = json_number_values(object, "id");
        // Ignore parent objects and windows without an app_id; their nested
        // fields must never be paired with another window's identifier.
        if app_ids.len() != 1 || ids.len() != 1 { continue; }
        let app_id = app_ids.into_iter().next().unwrap_or_default();
        let id = ids[0];
        if !app_id.is_empty() && !windows.iter().any(|window: &RunningWindow| window.id == id) {
            windows.push(RunningWindow { id, app_id });
        }
    }
    windows
}

fn json_objects(input: &str) -> Vec<&str> {
    let mut starts = Vec::new();
    let mut objects = Vec::new();
    let mut in_string = false;
    let mut escaped = false;

    for (index, byte) in input.bytes().enumerate() {
        if in_string {
            if escaped {
                escaped = false;
            } else if byte == b'\\' {
                escaped = true;
            } else if byte == b'\"' {
                in_string = false;
            }
            continue;
        }

        match byte {
            b'\"' => in_string = true,
            b'{' => starts.push(index),
            b'}' => {
                if let Some(start) = starts.pop() {
                    objects.push(&input[start..=index]);
                }
            }
            _ => {}
        }
    }

    objects
}
fn json_number_values(input: &str, key: &str) -> Vec<u64> {
    let marker = format!("\"{}\"", key);
    let mut values = Vec::new();
    let mut offset = 0;
    while let Some(found) = input[offset..].find(&marker) {
        let key_end = offset + found + marker.len();
        let Some(colon) = input[key_end..].find(':') else { break };
        let mut index = key_end + colon + 1;
        while input.as_bytes().get(index).is_some_and(u8::is_ascii_whitespace) { index += 1; }
        let start = index;
        while input.as_bytes().get(index).is_some_and(u8::is_ascii_digit) { index += 1; }
        if start < index {
            if let Ok(value) = input[start..index].parse::<u64>() { values.push(value); }
        }
        offset = index.max(key_end + 1);
    }
    values
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

fn focus_window(id: u64) {
    let id = id.to_string();
    let _ = Command::new("niri")
        .args(["msg", "action", "focus-window", "--id", id.as_str()])
        .spawn();
}

fn app_matches_id(app: &DesktopApp, app_id: &str) -> bool {
    let normalize = |value: &str| {
        let value = value.to_lowercase();
        let value = value.rsplit('/').next().unwrap_or(&value);
        let value = value.strip_suffix(".desktop").unwrap_or(value);
        value.chars().filter(char::is_ascii_alphanumeric).collect::<String>()
    };

    let target = normalize(app_id);
    let name = normalize(app.name());
    let exec = app.exec().split_whitespace().next().unwrap_or("");
    let exec = normalize(exec);

    target == name || target == exec
        || (!target.is_empty() && (target.contains(&name) || name.contains(&target)))
}

pub fn view(monitor: &Monitor) -> LayerWindow {
    crate::ui::Ui::watch();

    let settings = Settings::read();
    let mode = settings.shell_mode();
    let position = String::from(settings.dock_position());
    let alignment = String::from(settings.dock_alignment());
    drop(settings);

    let transition = motion::mode_transition();
    let dock_scale = motion::mode_scale(mode, 2, transition);
    let pointer_key = format!("dock:pointer:{}", monitor.name);
    if dock_scale <= 0.001 {
        motion::set_pointer_x(pointer_key, None);
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

    let running_windows = RunningApps::read().windows.clone();
    let mut items: Vec<(DesktopApp, Vec<u64>, bool)> = Vec::new();

    for exec in &pinned {
        if let Some(app) = apps.list().iter().find(|app| app.exec() == exec) {
            let window_ids = running_windows.iter()
                .filter(|window| app_matches_id(app, &window.app_id))
                .map(|window| window.id)
                .collect();
            items.push((app.clone(), window_ids, true));
        }
    }

    let pinned_count = items.len();

    for window in &running_windows {
        if let Some(app) = apps.list().iter().find(|app| app_matches_id(app, &window.app_id)) {
            if !items.iter().any(|(existing, _, _)| existing.exec() == app.exec()) {
                let window_ids = running_windows.iter()
                    .filter(|candidate| app_matches_id(app, &candidate.app_id))
                    .map(|candidate| candidate.id)
                    .collect();
                items.push((app.clone(), window_ids, false));
            }
        }
    }
    drop(apps);

    // Keep enough headroom for icons to grow upward without resizing the layer.
    let base_slot = (icon_size * MAGNIFICATION + 24.0).max(72.0);
    // The mapped surface reserves headroom for the enlarged icons and badges.
    // The visible capsule remains compact and is anchored to the selected edge.
    let base_body_height = (icon_size * MAGNIFICATION + 32.0).max(96.0);
    let base_height = (base_body_height + 64.0).max(160.0);
    let has_separator = pinned_count > 0 && pinned_count < items.len();
    let base_width = (items.len() as f32 + 2.0) * (base_slot + 4.0)
        + 16.0 + if has_separator { 6.0 } else { 0.0 };
    let slot = (base_slot * dock_scale).max(1.0);
    let dock_height = (base_body_height * dock_scale).max(1.0);
    let dock_width = (base_width * dock_scale).max(1.0);
    let dock_side = if position == "top" { 1.0 } else { -1.0 };
    let magnification_enabled = magnification;
    if transition.is_some() {
        motion::set_pointer_x(pointer_key.clone(), None);
    }
    let pointer_x = if transition.is_some() {
        None
    } else {
        motion::pointer_x(&pointer_key)
    };

    let mut children: Vec<Box<dyn Widget>> = Vec::new();
    let mut row_cursor = 10.0;

    for (index, (app, window_ids, _pinned)) in items.iter().enumerate() {
        if index == pinned_count && has_separator {
            children.push(Box::new(
                Rectangle::new()
                    .width(2.0)
                    .height(34.0)
                    .radius(1.0)
                    .fill(fade(scheme.outline_variant, 150)),
            ));
            row_cursor += 6.0;
        }

        let center = row_cursor + slot * 0.5;
        let proximity = if magnification_enabled {
            pointer_x.map_or(0.0, |x| gaussian((x - center) / 46.0))
        } else {
            0.0
        };
        children.push(Box::new(app_item(
            app, scheme, icon_size * dock_scale, magnification_enabled,
            window_ids, proximity, slot, dock_height, dock_scale, dock_side,
        )));
        row_cursor += slot + 4.0;
    }

    let apps_center = row_cursor + slot * 0.5;
    let apps_proximity = if magnification_enabled {
        pointer_x.map_or(0.0, |x| gaussian((x - apps_center) / 46.0))
    } else {
        0.0
    };
    children.push(Box::new(action_item(
        "dock:apps", m3::APPS, scheme, slot, dock_height, apps_proximity, magnification_enabled, dock_scale, dock_side,
        || Overlay::toggle(crate::overlay::Panel::Apps),
    )));
    let trash_center = apps_center + slot + 4.0;
    let trash_proximity = if magnification_enabled {
        pointer_x.map_or(0.0, |x| gaussian((x - trash_center) / 46.0))
    } else {
        0.0
    };
    children.push(Box::new(action_item(
        "dock:trash", m3::DELETE, scheme, slot, dock_height, trash_proximity, magnification_enabled, dock_scale, dock_side,
        || amane::spawn("xdg-open trash:///"),
    )));

    let shape_height = (dock_height - 12.0).max(1.0);
    let shape_x = (base_width - dock_width) * 0.5;
    let shape_y = if dock_side < 0.0 {
        base_body_height - shape_height
    } else {
        0.0
    };

    // K4's Muelle keeps the drawing surface fixed and morphs the path inside
    // it. This avoids rebuilding/resizing the Wayland layer during animation.
    let shadow = Rectangle::new()
        .width(dock_width)
        .height(shape_height)
        .fill(amane::Color::TRANSPARENT)
        .radius(30.0)
        .shadow(
            Shadow::drop(scheme.shadow)
                .opacity(0.34)
                .blur(28.0)
                .offset(0.0, 8.0),
        )
        .translate(shape_x, shape_y);

    let content = Rectangle::new()
        .width(dock_width)
        .height(shape_height)
        .padding(8.0)
        .translate(shape_x, shape_y)
        .child(
            Row::new(children)
                .gap(4.0)
                .align(Center)
                .justify(Center)
                .width(Parent)
                .height(Parent),
        );

    let body = amane::Stack::new(vec![
        Box::new(shadow),
        Box::new(dock_shape(
            base_width,
            base_body_height,
            dock_width,
            shape_height,
            dock_side,
            scheme,
        )),
        Box::new(content),
    ])
    .width(base_width)
    .height(base_body_height);

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

    // Keep the Wayland surface at monitor size. Only the dock's inner
    // geometry changes; resizing a layer surface during magnification or a
    // mode transition makes the compositor reconfigure it and causes jitter.
    // The narrow input region means the transparent part of this surface does
    // not steal clicks from applications.
    let dock_x = match alignment.as_str() {
        "left" => 0.0,
        "right" => (monitor.width as f32 - base_width).max(0.0),
        _ => ((monitor.width as f32 - base_width) * 0.5).max(0.0),
    };
    let dock_y = if position == "top" {
        0.0
    } else {
        (monitor.height as f32 - base_height).max(0.0)
    };
    let input_region = if transition.is_some() {
        Vec::new()
    } else {
        vec![InputArea {
            x: dock_x.floor() as i32,
            y: dock_y.floor() as i32,
            width: base_width.ceil().min(monitor.width as f32) as i32,
            height: base_height.ceil().min(monitor.height as f32) as i32,
        }]
    };

    let window = LayerWindow::new()
        .width(Full)
        .height(Full)
        .anchor_vertical(Vertical::Top)
        .anchor_horizontal(Horizontal::Left)
        .layer(Layer::Top)
        .space(Zone::Ignore)
        .namespace("cds-dock")
        .visible(true)
        .input_region(input_region);
    let window = if transition.is_some() { window.click_through() } else { window };
    let pointer_move_key = pointer_key.clone();
    let pointer_leave_key = pointer_key;
    let horizontal_align = match alignment.as_str() {
        "left" => Start,
        "right" => End,
        _ => Center,
    };
    window.child(
        Rectangle::new()
            .width(Parent)
            .height(Parent)
            .on_move(move |point| {
                motion::set_pointer_x(pointer_move_key.clone(), Some(point.x - dock_x));
            })
            .on_hover(move |inside| {
                if !inside {
                    motion::set_pointer_x(pointer_leave_key.clone(), None);
                }
            })
            .align_child(
                horizontal_align,
                if position == "top" { Start } else { End },
            )
            .child(body),
    )
}

fn dock_shape(
    window_width: f32,
    canvas_height: f32,
    shape_width: f32,
    shape_height: f32,
    dock_side: f32,
    scheme: &Scheme,
) -> Canvas {
    let x0 = (window_width - shape_width) * 0.5;
    let origin_y = if dock_side < 0.0 {
        canvas_height - shape_height
    } else {
        0.0
    };
    let map_y = |y: f32| {
        if dock_side < 0.0 {
            origin_y + shape_height - y
        } else {
            origin_y + y
        }
    };

    let radius = 30.0_f32.min(shape_height * 0.5).min(shape_width / 3.0);
    let wing = 16.0_f32.min(shape_height * 0.5).min(shape_width / 6.0);
    let right = x0 + shape_width;
    let path = Path::new()
        .move_to(x0, map_y(0.0))
        .cubic_to(
            x0 + wing * 0.12,
            map_y(0.0),
            x0 + wing,
            map_y(wing * 0.55),
            x0 + wing,
            map_y(wing),
        )
        .line_to(x0 + wing, map_y(shape_height - radius))
        .cubic_to(
            x0 + wing,
            map_y(shape_height - radius * 0.25),
            x0 + wing + radius * 0.25,
            map_y(shape_height),
            x0 + wing + radius,
            map_y(shape_height),
        )
        .line_to(right - wing - radius, map_y(shape_height))
        .cubic_to(
            right - wing - radius * 0.25,
            map_y(shape_height),
            right - wing,
            map_y(shape_height - radius * 0.25),
            right - wing,
            map_y(shape_height - radius),
        )
        .line_to(right - wing, map_y(wing))
        .cubic_to(
            right - wing,
            map_y(wing * 0.55),
            right - wing * 0.12,
            map_y(0.0),
            right,
            map_y(0.0),
        )
        .line_to(x0, map_y(0.0))
        .close()
        .fill(scheme.surface_container_high)
        .stroke(1.0, fade(scheme.outline_variant, 90));

    Canvas::new()
        .width(window_width)
        .height(canvas_height)
        .shapes(vec![Box::new(path) as Box<dyn Shape>])
}

fn gaussian(distance: f32) -> f32 {
    (-distance * distance).exp()
}

#[cfg(test)]
mod tests {
    use super::gaussian;

    #[test]
    fn magnification_is_maximum_at_the_pointer() {
        assert!((gaussian(0.0) - 1.0).abs() < f32::EPSILON);
    }

    #[test]
    fn magnification_falls_off_symmetrically() {
        assert!((gaussian(-0.75) - gaussian(0.75)).abs() < f32::EPSILON);
        assert!(gaussian(1.0) < gaussian(0.5));
        assert!(gaussian(2.0) < gaussian(1.0));
    }
}

fn app_item(
    app: &DesktopApp,
    scheme: &Scheme,
    icon_base: f32,
    magnification: bool,
    window_ids: &[u64],
    proximity: f32,
    slot: f32,
    dock_height: f32,
    transition_scale: f32,
    dock_side: f32,
) -> Rectangle {
    let exec = String::from(app.exec());
    let key = format!("dock:{}", exec);
    let proximity = if magnification { proximity } else { 0.0 };
    let scale = motion::number(
        &format!("{key}:scale"),
        1.0 + 0.62 * proximity,
        motion::style_duration(90, 90),
    );
    let size = icon_base * scale;
    let rise = (8.0 + 10.0 * proximity) * transition_scale;
    let bounce = motion::bounce(&key) * dock_side;
    let launch = exec.clone();
    let bounce_key = key.clone();
    let focus_key = key.clone();
    let scroll_key = key.clone();
    let click_window_ids = window_ids.to_vec();
    let scroll_window_ids = window_ids.to_vec();
    let window_count = window_ids.len();

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
    if window_count > 0 {
        layers.push(Box::new(
            Rectangle::new()
                .width(7.0)
                .height(7.0)
                .radius(4.0)
                .fill(scheme.primary)
                .border(1.0, scheme.surface_container_high)
                .translate((size - 7.0) * 0.5, if dock_side < 0.0 { size - 3.0 } else { -4.0 }),
        ));
    }
    if window_count > 1 {
        layers.push(Box::new(
            Rectangle::new()
                .width(19.0)
                .height(19.0)
                .radius(10.0)
                .fill(scheme.primary)
                .border(2.0, scheme.surface_container_high)
                .translate(size - 10.0, if dock_side < 0.0 { -9.0 } else { size - 10.0 })
                .align_child(Center, Center)
                .child(Text::new(window_count.to_string()).size(9.5).color(scheme.on_primary).weight(amane::Weight::Bold).tight()),
        ));
    }

    let label_alpha = motion::number(
        &format!("{key}:label"),
        ((proximity - 0.82) / 0.18).clamp(0.0, 1.0),
        motion::style_duration(100, 80),
    );
    layers.push(Box::new(
        Rectangle::new()
            .width(100.0)
            .height(22.0)
            .radius(8.0)
            .fill(scheme.surface_container_highest)
            .opacity(label_alpha)
            .translate(-(100.0 - size) * 0.5, if dock_side < 0.0 { -30.0 } else { size + 8.0 })
            .align_child(Center, Center)
            .child(Text::new(app.name()).size(11.0).color(scheme.on_surface).weight(amane::Weight::Medium).elide()),
    ));

    Rectangle::new()
        .width(slot)
        .height((dock_height - 28.0).max(1.0))
        .cursor(Pointer)
        .on_click(move |button| match button {
            Button::Left => {
                motion::trigger_bounce(bounce_key.clone());
                if !click_window_ids.is_empty() {
                    let index = motion::dock_window_index(&focus_key, click_window_ids.len());
                    if let Some(id) = click_window_ids.get(index) {
                        focus_window(*id);
                    }
                } else {
                    amane::spawn(&launch);
                }
                Overlay::close();
            }
            Button::Right => Settings::toggle_pinned(&launch),
            _ => {}
        })
        .on_scroll(move |scroll: Scroll| {
            if scroll_window_ids.len() < 2 || scroll.y == 0.0 {
                return;
            }
            let delta = if scroll.y > 0.0 { 1 } else { -1 };
            let index = motion::cycle_dock_window(&scroll_key, delta, scroll_window_ids.len());
            if let Some(id) = scroll_window_ids.get(index) {
                focus_window(*id);
            }
        })
        .align_child(Center, Center)
        .child(
            Rectangle::new()
                .width(size)
                .height(size)
                .translate(0.0, dock_side * rise + bounce)
                .child(amane::Stack::new(layers).width(size).height(size)),
        )
}

fn action_item(
    key: &'static str,
    glyph: &str,
    scheme: &Scheme,
    slot: f32,
    dock_height: f32,
    proximity: f32,
    magnification: bool,
    transition_scale: f32,
    dock_side: f32,
    action: impl Fn() + 'static,
) -> Rectangle {
    let action_size = ACTION_ICON.min((dock_height - 28.0).max(1.0) * 0.72);
    let proximity = if magnification { proximity } else { 0.0 };
    let scale = motion::number(
        &format!("{key}:scale"),
        1.0 + 0.48 * proximity,
        motion::style_duration(90, 90),
    );
    Rectangle::new()
        .width(slot)
        .height((dock_height - 28.0).max(1.0))
        .cursor(Pointer)
        .on_click(move |button| {
            if button == Button::Left {
                action();
            }
        })
        .on_drop(|paths: Vec<PathBuf>| {
            // GIO moves dropped files to the desktop trash using the
            // freedesktop trash specification (including restore metadata).
            thread::spawn(move || {
                let mut command = Command::new("gio");
                command.arg("trash").args(paths);
                let _ = command.spawn();
            });
        })
        .align_child(Center, Center)
        .child(
            Rectangle::new()
                .width(action_size * scale)
                .height(action_size * scale)
                .translate(0.0, dock_side * 8.0 * proximity * transition_scale)
                .radius(14.0)
                .fill(scheme.surface_container_highest)
                .align_child(Center, Center)
                .child(m3::icon(glyph, action_size * 0.58 * scale, scheme.on_surface)),
        )
}
