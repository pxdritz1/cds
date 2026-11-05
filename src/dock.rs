use std::path::PathBuf;
use std::process::{Command, Stdio};
use std::thread;
use std::time::{Duration, Instant};

use amane::{
    Align, Apps, Button, Canvas, Center, Column, DesktopApp, Horizontal, Image, InputArea, Keyboard, Key, Layer, LayerWindow, Monitor,
    End, Parent, Path, Pointer, Rectangle, Row, Scroll, ScrollArea, Service, Shadow, Shape, Start, Text, TextInput, Vertical, Widget,
    Zone,
};

use crate::m3;
use crate::motion;
use crate::overlay::Overlay;
use crate::settings::Settings;
use crate::theme::{Scheme, Theme, fade};

const ACTION_ICON: f32 = 42.0;
const MAGNIFICATION: f32 = 1.32;
const DRAWER_SEARCH_ID: &str = "dock-apps-search";

#[derive(Clone, PartialEq, Eq)]
struct RunningWindow {
    id: u64,
    app_id: String,
    title: String,
}

#[derive(Default, PartialEq, Eq)]
struct RunningApps {
    windows: Vec<RunningWindow>,
}

struct DockUi {
    picker: Option<String>,
    drawer_open: bool,
    query: String,
    hovered: bool,
    last_activity: Instant,
    auto_hidden: bool,
}

impl Default for DockUi {
    fn default() -> Self {
        Self {
            picker: None,
            drawer_open: false,
            query: String::new(),
            hovered: false,
            last_activity: Instant::now(),
            auto_hidden: false,
        }
    }
}

impl Service for DockUi {
    fn new() -> Self { Self::default() }

    fn listen() {
        loop {
            let auto_hide = Settings::read().flag("dock_auto_hide");
            let changed = {
                let mut state = Self::write();
                let protected = state.hovered || state.drawer_open || state.picker.is_some();
                let should_hide = auto_hide
                    && !protected
                    && state.last_activity.elapsed() >= Duration::from_millis(1600);
                if state.auto_hidden != should_hide {
                    state.auto_hidden = should_hide;
                    true
                } else {
                    false
                }
            };
            if changed { crate::ui::Ui::refresh(); }
            thread::sleep(Duration::from_millis(if auto_hide { 80 } else { 250 }));
        }
    }
}

impl DockUi {
    fn snapshot(&self) -> (Option<String>, bool, String, bool) {
        (
            if self.drawer_open { None } else { self.picker.clone() },
            self.drawer_open,
            self.query.clone(),
            self.auto_hidden,
        )
    }

    fn set_hovered(inside: bool) {
        let changed = {
            let mut state = Self::write();
            let changed = state.hovered != inside || (inside && state.auto_hidden);
            if changed {
                state.hovered = inside;
                state.last_activity = Instant::now();
            }
            if inside { state.auto_hidden = false; }
            changed
        };
        if changed { crate::ui::Ui::refresh(); }
    }

    fn toggle_picker(exec: &str) {
        let mut state = Self::write();
        state.picker = if state.picker.as_deref() == Some(exec) {
            None
        } else {
            Some(String::from(exec))
        };
        state.drawer_open = false;
        state.query.clear();
        state.auto_hidden = false;
        state.last_activity = Instant::now();
        drop(state);
        crate::ui::Ui::refresh();
    }

    fn close_picker() {
        let changed = {
            let mut state = Self::write();
            let changed = state.picker.is_some();
            state.picker = None;
            state.last_activity = Instant::now();
            changed
        };
        if changed { crate::ui::Ui::refresh(); }
    }

    fn toggle_drawer() {
        let open = {
            let mut state = Self::write();
            state.drawer_open = !state.drawer_open;
            state.picker = None;
            state.query.clear();
            state.auto_hidden = false;
            state.last_activity = Instant::now();
            state.drawer_open
        };
        if open { TextInput::set_text(DRAWER_SEARCH_ID, ""); }
        crate::ui::Ui::refresh();
    }

    fn set_query(query: String) {
        Self::write().query = query;
        crate::ui::Ui::refresh();
    }

    fn close_drawer() {
        let changed = {
            let mut state = Self::write();
            let changed = state.drawer_open;
            state.drawer_open = false;
            state.query.clear();
            state.auto_hidden = false;
            state.last_activity = Instant::now();
            changed
        };
        if changed {
            TextInput::set_text(DRAWER_SEARCH_ID, "");
            crate::ui::Ui::refresh();
        }
    }
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
        let title = json_string_values(object, "title").into_iter().next().unwrap_or_default();
        let id = ids[0];
        if !app_id.is_empty() && !windows.iter().any(|window: &RunningWindow| window.id == id) {
            windows.push(RunningWindow { id, app_id, title });
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


pub fn reserve(monitor: &Monitor) -> LayerWindow {
    let settings = Settings::read();
    let mode = settings.shell_mode();
    let enabled = settings.flag("show_dock");
    let icon_size = settings.dock_icon_size() as f32;
    let position = String::from(settings.dock_position());
    drop(settings);
    let scale = motion::mode_scale(mode, 2, motion::mode_transition());
    if !enabled || scale <= 0.001 || position != "bottom" {
        return LayerWindow::new().width(1.0).height(1.0).visible(false).namespace("cds-dock-reserve");
    }
    let dock_height = (icon_size * MAGNIFICATION + 32.0).max(96.0);
    LayerWindow::new().width(monitor.width as f32).height(dock_height * scale)
        .anchor_vertical(Vertical::Bottom).anchor_horizontal(Horizontal::Left)
        .layer(Layer::Bottom).space(Zone::Reserve).namespace("cds-dock-reserve")
        .click_through().child(Rectangle::new().width(Parent).height(Parent))
}

pub fn view(monitor: &Monitor) -> LayerWindow {
    crate::ui::Ui::watch();

    let settings = Settings::read();
    let mode = settings.shell_mode();
    let position = String::from(settings.dock_position());
    let alignment = String::from(settings.dock_alignment());
    let enabled = settings.flag("show_dock");
    drop(settings);

    let transition = motion::mode_transition();
    let dock_scale = motion::mode_scale(mode, 2, transition);
    let pointer_key = format!("dock:pointer:{}", monitor.name);
    if !enabled || dock_scale <= 0.001 {
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
    let auto_hide = settings.flag("dock_auto_hide");
    drop(settings);

    let running_windows = RunningApps::read().windows.clone();
    let (picker_exec, drawer_open, drawer_query, auto_hidden) = DockUi::read().snapshot();
    let hide_target = if auto_hide && auto_hidden && !drawer_open && picker_exec.is_none() && transition.is_none() {
        0.0
    } else {
        1.0
    };
    let reveal_progress = motion::number(
        &format!("dock:auto-hide:{}", monitor.name),
        hide_target,
        Duration::from_millis(360),
    );
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
    let drawer_apps = apps.list().to_vec();
    drop(apps);

    // Keep enough headroom for icons to grow upward without resizing the layer.
    let base_slot = (icon_size + 6.0).max(54.0);
    // The mapped surface reserves headroom for the enlarged icons and badges.
    // The visible capsule remains compact and is anchored to the selected edge.
    let base_body_height = (icon_size * MAGNIFICATION + 32.0).max(96.0);
    let base_height = (base_body_height + 64.0).max(160.0);
    let has_separator = pinned_count > 0 && pinned_count < items.len();
    let base_width = (items.len() as f32 + 2.0) * (base_slot + 4.0)
        + 16.0 + if has_separator { 6.0 } else { 0.0 };
    let dock_x = match alignment.as_str() {
        "left" => 0.0,
        "right" => (monitor.width as f32 - base_width).max(0.0),
        _ => ((monitor.width as f32 - base_width) * 0.5).max(0.0),
    };
    let dock_y = if position == "top" {
        12.0
    } else {
        (monitor.height as f32 - base_body_height).max(0.0)
    };
    let body_y = if position == "top" {
        12.0
    } else {
        (monitor.height as f32 - base_body_height).max(0.0)
    };
    let hide_offset = hidden_offset(&position, base_body_height) * (1.0 - reveal_progress);

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
    let mut picker_anchor = None;

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
        if picker_exec.as_deref() == Some(app.exec()) && window_ids.len() > 1 {
            picker_anchor = Some(center);
        }
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
        || DockUi::toggle_drawer(),
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

    let mut body_layers: Vec<Box<dyn Widget>> = vec![
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
    ];

    let mut picker_region = None;
    if let (Some(exec), Some(anchor)) = (picker_exec.as_deref(), picker_anchor) {
        if let Some((app, _, _)) = items.iter().find(|(app, _, _)| app.exec() == exec) {
            let windows: Vec<RunningWindow> = running_windows.iter()
                .filter(|window| app_matches_id(app, &window.app_id))
                .cloned()
                .collect();
            if windows.len() > 1 {
                let panel_width = 280.0_f32.min((monitor.width as f32 - 24.0).max(180.0));
                let panel_height = 44.0 + windows.len().min(5) as f32 * 42.0;
                let panel_x = (anchor - panel_width * 0.5)
                    .clamp(8.0, (base_width - panel_width - 8.0).max(8.0));
                let body_y = if position == "top" {
                    12.0
                } else {
                    (monitor.height as f32 - base_body_height - 14.0).max(0.0)
                };
                let desired_y = if dock_side < 0.0 {
                    body_y - panel_height - 12.0
                } else {
                    body_y + base_body_height + 12.0
                };
                let panel_screen_y = desired_y.clamp(
                    4.0,
                    (monitor.height as f32 - panel_height - 4.0).max(4.0),
                );
                let panel_y = panel_screen_y - body_y;
                picker_region = Some(InputArea {
                    x: panel_x.floor() as i32,
                    y: panel_screen_y.floor() as i32,
                    width: panel_width.ceil() as i32,
                    height: panel_height.ceil() as i32,
                });
                body_layers.push(Box::new(window_picker(
                    &windows, scheme, panel_width, panel_height, panel_x, panel_y,
                )));
            }
        }
    }

    let mut drawer_region = None;
    if drawer_open {
        let drawer_width = (monitor.width as f32 - 32.0).max(240.0).min(720.0);
        let drawer_height = (monitor.height as f32 - 32.0).max(220.0).min(420.0);
        let drawer_screen_x = (dock_x + base_width * 0.5 - drawer_width * 0.5)
            .clamp(8.0, (monitor.width as f32 - drawer_width - 8.0).max(8.0));
        let desired_y = if dock_side < 0.0 {
            body_y - drawer_height - 12.0
        } else {
            body_y + base_body_height + 12.0
        };
        let drawer_screen_y = desired_y.clamp(
            8.0,
            (monitor.height as f32 - drawer_height - 8.0).max(8.0),
        );
        drawer_region = Some(InputArea {
            x: drawer_screen_x.floor() as i32,
            y: drawer_screen_y.floor() as i32,
            width: drawer_width.ceil() as i32,
            height: drawer_height.ceil() as i32,
        });
        body_layers.push(Box::new(app_drawer(
            &drawer_apps,
            &drawer_query,
            scheme,
            drawer_width,
            drawer_height,
            drawer_screen_x - dock_x,
            drawer_screen_y - body_y,
        )));
    }

    let body = amane::Stack::new(body_layers)
        .width(base_width)
        .height(base_body_height);

    // Keep the Wayland surface at monitor size. Only the dock's inner
    // geometry changes; resizing a layer surface during magnification or a
    // mode transition makes the compositor reconfigure it and causes jitter.
    // The narrow input region means the transparent part of this surface does
    // not steal clicks from applications.
    let input_region = if transition.is_some() {
        Vec::new()
    } else if auto_hide && auto_hidden && reveal_progress <= 0.001 {
        vec![InputArea {
            x: 0,
            y: if position == "top" { 0 } else { (monitor.height as i32 - 4).max(0) },
            width: monitor.width as i32,
            height: 4,
        }]
    } else {
        let mut regions = vec![InputArea {
            x: dock_x.floor() as i32,
            y: (dock_y + hide_offset).floor().max(0.0) as i32,
            width: base_width.ceil().min(monitor.width as f32) as i32,
            height: base_body_height.ceil().min(monitor.height as f32) as i32,
        }];
        if let Some(mut picker) = picker_region {
            picker.x += dock_x.floor() as i32;
            regions.push(picker);
        }
        if let Some(drawer) = drawer_region {
            regions.push(drawer);
        }
        regions
    };

    let keyboard = if drawer_open { Keyboard::OnDemand } else { Keyboard::None };
    let window = LayerWindow::new()
        .width(monitor.width as f32)
        .height(monitor.height as f32)
        .anchor_vertical(Vertical::Top)
        .anchor_horizontal(Horizontal::Left)
        .layer(Layer::Top)
        .space(Zone::Ignore)
        .keyboard(keyboard)
        .namespace("cds-dock")
        .visible(true)
        .input_region(input_region);
    let window = window.on_key(|key| {
        if key == Key::Escape { DockUi::close_drawer(); }
    });
    let window = if transition.is_some() { window.click_through() } else { window };
    let pointer_move_key = pointer_key.clone();
    let pointer_leave_key = pointer_key;
    let horizontal_align = match alignment.as_str() {
        "left" => Align::Start,
        "right" => Align::End,
        _ => Align::Center,
    };
    window.child(
        Rectangle::new()
            .width(Parent)
            .height(Parent)
            .on_move(move |point| {
                motion::set_pointer_x(pointer_move_key.clone(), Some(point.x - dock_x));
            })
            .on_hover(move |inside| {
                DockUi::set_hovered(inside);
                if !inside {
                    motion::set_pointer_x(pointer_leave_key.clone(), None);
                }
            })
            .align_child(
                horizontal_align,
                if position == "top" { Align::Start } else { Align::End },
            )
            .child(
                Rectangle::new()
                    .width(base_width)
                    .height(base_body_height)
                    .translate(
                        0.0,
                        (if position == "top" { 12.0 } else { 0.0 }) + hide_offset,
                    )
                    .child(body),
            ),
    )
}

fn app_drawer(
    apps: &[DesktopApp],
    query: &str,
    scheme: &Scheme,
    width: f32,
    height: f32,
    x: f32,
    y: f32,
) -> Rectangle {
    let needle = query.trim().to_lowercase();
    let mut results: Vec<&DesktopApp> = apps.iter().filter(|app| {
        needle.is_empty()
            || app.name().to_lowercase().contains(&needle)
            || app.description().is_some_and(|description| description.to_lowercase().contains(&needle))
    }).collect();
    let settings = Settings::read();
    results.sort_by_key(|app| !settings.is_pinned(app.exec()));
    drop(settings);

    let mut grid_rows: Vec<Box<dyn Widget>> = Vec::new();
    for group in results.chunks(5) {
        let tiles = group.iter().map(|app| {
            Box::new(drawer_tile(app, scheme)) as Box<dyn Widget>
        }).collect();
        grid_rows.push(Box::new(Row::new(tiles).gap(8.0).width(Parent)));
    }

    let grid: Box<dyn Widget> = if grid_rows.is_empty() {
        Box::new(Rectangle::new()
            .width(Parent)
            .height(180.0)
            .align_child(Center, Center)
            .child(Text::new(crate::lang::text("Nenhum aplicativo encontrado", "No applications found"))
                .size(13.0).color(scheme.on_surface_variant)))
    } else {
        Box::new(ScrollArea::new(
            "dock-app-grid",
            Column::new(grid_rows).gap(8.0).width(Parent),
        ).width(Parent).height(Parent))
    };

    let title = Rectangle::new()
        .width(Parent)
        .height(24.0)
        .align_child(Start, Center)
        .child(Text::new(crate::lang::text("Aplicativos", "Applications"))
            .size(15.0).color(scheme.on_surface).weight(amane::Weight::SemiBold));

    let search = Rectangle::new()
        .width(Parent)
        .height(38.0)
        .radius(12.0)
        .fill(scheme.surface_container_highest)
        .padding(10.0)
        .align_child(Start, Center)
        .child(TextInput::new(DRAWER_SEARCH_ID)
            .width(Parent)
            .size(13.0)
            .color(scheme.on_surface)
            .placeholder(crate::lang::text("Buscar aplicativos...", "Search applications..."))
            .focused()
            .on_change(DockUi::set_query));

    Rectangle::new()
        .width(width)
        .height(height)
        .translate(x, y)
        .padding(16.0)
        .radius(24.0)
        .fill(scheme.surface_container)
        .border(1.0, fade(scheme.outline_variant, 120))
        .shadow(Shadow::drop(scheme.shadow).opacity(0.38).blur(28.0).offset(0.0, 10.0))
        .child(Column::new(vec![Box::new(title), Box::new(search), grid])
            .gap(10.0).width(Parent).height(Parent))
}

fn drawer_tile(app: &DesktopApp, scheme: &Scheme) -> Rectangle {
    let app_to_launch = app.clone();
    let exec = String::from(app.exec());
    let icon: Box<dyn Widget> = match app.icon_path() {
        Some(path) if Image::loaded(path) => Box::new(
            Rectangle::new().width(36.0).height(36.0).radius(11.0).clip()
                .fill(Image::contain(path).thumbnail(96, 96)),
        ),
        _ => Box::new(
            Rectangle::new().width(36.0).height(36.0).radius(11.0)
                .fill(scheme.primary_container)
                .align_child(Center, Center)
                .child(m3::icon(crate::glyphs::application(app.name(), app.exec()), 21.0, scheme.on_primary_container)),
        ),
    };
    let name = Text::new(app.name()).size(10.5).color(scheme.on_surface).elide();
    Rectangle::new()
        .width(108.0)
        .height(84.0)
        .radius(14.0)
        .fill(scheme.surface_container_high)
        .cursor(Pointer)
        .on_click(move |button| match button {
            Button::Left => {
                app_to_launch.launch();
                DockUi::close_drawer();
                Overlay::close();
            }
            Button::Right => Settings::toggle_pinned(&exec),
            _ => {}
        })
        .child(Column::new(vec![icon, Box::new(name)])
            .gap(6.0).align(Center).justify(Center).width(Parent).height(Parent))
}

fn window_picker(
    windows: &[RunningWindow],
    scheme: &Scheme,
    width: f32,
    height: f32,
    x: f32,
    y: f32,
) -> Rectangle {
    let mut rows: Vec<Box<dyn Widget>> = Vec::new();
    rows.push(Box::new(
        Rectangle::new()
            .width(Parent)
            .height(28.0)
            .align_child(Start, Center)
            .child(Text::new(crate::lang::text("Janelas abertas", "Open windows"))
                .size(11.0)
                .color(scheme.on_surface_variant)
                .weight(amane::Weight::SemiBold)),
    ));

    for window in windows.iter().take(5) {
        let id = window.id;
        let title = if window.title.trim().is_empty() {
            crate::lang::text("Janela sem título", "Untitled window")
        } else {
            window.title.as_str()
        };
        rows.push(Box::new(
            Rectangle::new()
                .width(Parent)
                .height(38.0)
                .radius(10.0)
                .fill(scheme.surface_container_highest)
                .cursor(Pointer)
                .on_click(move |button| {
                    if button == Button::Left {
                        focus_window(id);
                        DockUi::close_picker();
                    }
                })
                .padding(10.0)
                .align_child(Start, Center)
                .child(Text::new(title).size(12.0).color(scheme.on_surface).elide()),
        ));
    }

    Rectangle::new()
        .width(width)
        .height(height)
        .translate(x, y)
        .padding(8.0)
        .radius(18.0)
        .fill(scheme.surface_container)
        .border(1.0, fade(scheme.outline_variant, 120))
        .shadow(Shadow::drop(scheme.shadow).opacity(0.38).blur(24.0).offset(0.0, 8.0))
        .child(Column::new(rows).gap(4.0).width(Parent).height(Parent))
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

fn hidden_offset(position: &str, body_height: f32) -> f32 {
    if position == "top" { -(body_height + 8.0) } else { body_height + 10.0 }
}

fn gaussian(distance: f32) -> f32 {
    (-distance * distance).exp()
}

#[cfg(test)]
mod tests {
    use super::{gaussian, hidden_offset};

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

    #[test]
    fn auto_hide_leaves_four_pixels_at_both_edges() {
        let height = 120.0;
        assert_eq!(12.0 + hidden_offset("top", height) + height, 4.0);
        assert_eq!(-14.0 + hidden_offset("bottom", height), height - 4.0);
    }

    #[test]
    fn neighboring_icon_keeps_a_subtle_magnification() {
        let neighbor = gaussian(58.0 / 46.0);
        assert!(neighbor > 0.15 && neighbor < 0.25);
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
                if click_window_ids.len() > 1 {
                    DockUi::toggle_picker(&launch);
                } else {
                    DockUi::close_picker();
                    if let Some(id) = click_window_ids.first() {
                        focus_window(*id);
                    } else {
                        amane::spawn(&launch);
                    }
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
        .align_child(Center, if dock_side < 0.0 { Align::End } else { Align::Start })
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
        .align_child(Center, if dock_side < 0.0 { Align::End } else { Align::Start })
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
