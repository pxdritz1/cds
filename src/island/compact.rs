/*
 * the three islands of the bar. each one is its own surface: workspaces
 * and tray on the left, the clock in the middle (it grows into the dash)
 * and the system status on the right (it opens the control center).
 */
use std::time::Duration;

use amane::{
    Audio, Battery, Bluetooth, Button, Center, Color, Image, Media, Monitor, Network, Parent, Pointer, Rectangle,
    Row, Service, Stack, Widget, Workspace, Workspaces,
};

use crate::clock::Clock;
use crate::focused::FocusedWindow;
use crate::glyphs;
use crate::m3::{self, Type};
use crate::motion;
use crate::overlay::{Overlay, Panel};
use crate::theme::{Scheme, fade, mix};
use crate::tray::Tray;
use crate::weather::Weather;

use super::widgets::{EXPAND_MORE, MUSIC, boxed, icon, text};

const DOT: f32 = 8.0;
const FOCUSED: f32 = 28.0;
const FOCUSED_HEIGHT: f32 = 10.0;
const DOT_GAP: f32 = 7.0;
const SIDE: f32 = 14.0;
const ICON: f32 = 18.0;
const ITEM_GAP: f32 = 10.0;
const EDGE: f32 = 8.0;
const TRAY_BUTTON: f32 = 28.0;
const CLOCK_WIDTH: f32 = 260.0;
const CLOCK_CENTER_WIDTH: f32 = 166.0;


pub fn clock(scheme: &Scheme, alpha: f32) -> (Box<dyn Widget>, f32) {
    let clock = Clock::read();

    let time = m3::scale(Type::TitleSmall, clock.time(true), scheme.on_surface);
    let date = m3::scale(
        Type::LabelMedium,
        clock.short_date(),
        scheme.on_surface_variant,
    );

    let search = Rectangle::new()
        .width(26.0)
        .height(36.0)
        .cursor(Pointer)
        .on_click(|button| {
            if button == Button::Left {
                crate::spotlight::Spotlight::toggle();
            }
        })
        .align_child(Center, Center)
        .child(icon(crate::m3::SEARCH, ICON, scheme.on_surface_variant));

    let media_playing = Media::read().playing();
    let weather = Weather::read();
    let weather_icon: Box<dyn Widget> = if media_playing {
        boxed(
            Rectangle::new()
                .width(26.0)
                .height(36.0)
                .cursor(Pointer)
                .on_click(|button| match button {
                    Button::Left | Button::Right => crate::ui::Ui::open(crate::ui::Tab::Media),
                    _ => {}
                })
                .align_child(Center, Center)
                .child(
                    Stack::new(vec![
                        boxed(m3::cookie(12.0, 26.0, scheme.primary_container)),
                        boxed(
                            Rectangle::new()
                                .width(26.0)
                                .height(26.0)
                                .align_child(Center, Center)
                                .child(icon(MUSIC, 13.0, scheme.on_primary_container)),
                        ),
                    ])
                    .width(26.0)
                    .height(26.0),
                ),
        )
    } else {
        boxed(
            Rectangle::new()
                .width(26.0)
                .height(36.0)
                .cursor(Pointer)
                .on_click(|button| {
                    if button == Button::Left {
                        crate::ui::Ui::open(crate::ui::Tab::Weather);
                    }
                })
                .align_child(Center, Center)
                .child(icon(weather.glyph(), ICON, scheme.on_surface_variant)),
        )
    };

    let clock_content = Row::new(vec![
        boxed(time),
        boxed(icon("·", 10.0, fade(scheme.on_surface_variant, 140))),
        boxed(date),
    ])
    .gap(6.0)
    .align(Center);

    let time_date = Rectangle::new()
        .width(CLOCK_CENTER_WIDTH)
        .height(Parent)
        .cursor(Pointer)
        .align_child(Center, Center)
        .child(clock_content);

    let row = Row::new(vec![
        boxed(search),
        boxed(time_date),
        boxed(Stack::new(vec![weather_icon]).width(26.0).height(36.0)),
    ])
    .gap(3.0)
    .align(Center)
    .width(Parent)
    .height(Parent)
    .justify(Center);

    (
        boxed(
            Rectangle::new()
                .width(Parent)
                .height(Parent)
                .opacity(alpha)
                .padding(amane::Padding {
                    top: 2.0,
                    right: 3.0,
                    bottom: 2.0,
                    left: 3.0,
                })
                .child(row),
        ),
        CLOCK_WIDTH,
    )
}

pub fn workspaces(monitor: &Monitor, scheme: &Scheme) -> (Box<dyn Widget>, f32) {
    let workspaces = Workspaces::read();

    let mut dots: Vec<Box<dyn Widget>> = Vec::new();
    let mut dots_width = 0.0;

    for workspace in workspaces.list() {
        if workspace.output() != Some(monitor.name.as_str()) {
            continue;
        }

        if !dots.is_empty() {
            dots_width += DOT_GAP;
        }

        let width = dot_width(workspace);
        let height = dot_height(workspace);
        dots_width += width;
        dots.push(boxed(dot(workspace, width, height, scheme)));
    }

    drop(workspaces);

    let (focused_title, focused_app, focused_icon) = {
        let focused = FocusedWindow::read();
        (
            String::from(focused.title()),
            String::from(focused.app_id()),
            focused.icon_path().map(String::from),
        )
    };

    let mut children: Vec<Box<dyn Widget>> = Vec::new();
    let mut width = SIDE * 2.0 - 6.0;

    if !dots.is_empty() {
        children.push(boxed(Row::new(dots).gap(DOT_GAP).align(Center)));
        width += dots_width + ITEM_GAP;
    }

    if !focused_title.is_empty() {
        let icon_widget: Box<dyn Widget> = match focused_icon.as_deref() {
            Some(path) if Image::loaded(path) => boxed(
                Rectangle::new()
                    .width(24.0)
                    .height(24.0)
                    .radius(7.0)
                    .clip()
                    .fill(Image::contain(path).thumbnail(48, 48)),
            ),
            _ => boxed(m3::icon(
                glyphs::application(&focused_title, &focused_app),
                20.0,
                scheme.on_surface_variant,
            )),
        };

        let title: String = focused_title.chars().take(24).collect();
        let title = if focused_title.chars().count() > 24 {
            format!("{title}…")
        } else {
            title
        };

        let title_widget = Rectangle::new()
            .width(170.0)
            .height(30.0)
            .clip()
            .align_child(Center, Center)
            .child(m3::scale(Type::LabelSmall, title, scheme.on_surface_variant));

        children.push(boxed(
            Row::new(vec![icon_widget, boxed(title_widget)])
                .gap(8.0)
                .align(Center),
        ));
        width += ITEM_GAP + 202.0;
    }

    let row = Row::new(children)
        .gap(ITEM_GAP)
        .align(Center)
        .width(Parent)
        .height(Parent)
        .justify(Center);

    (boxed(row), width)
}

pub fn status(scheme: &Scheme) -> (Box<dyn Widget>, f32) {
    let network = Network::read();
    let bluetooth = Bluetooth::read();
    let audio = Audio::read();
    let battery = Battery::read();

    let settings = crate::settings::Settings::read();
    let positions = [
        String::from(settings.status_position("network")),
        String::from(settings.status_position("bluetooth")),
        String::from(settings.status_position("volume")),
        String::from(settings.status_position("microphone")),
        String::from(settings.status_position("battery")),
    ];
    drop(settings);

    let dim = fade(scheme.on_surface_variant, 130);
    let color = scheme.on_surface_variant;
    let mut groups: [Vec<Box<dyn Widget>>; 3] = [Vec::new(), Vec::new(), Vec::new()];
    let mut widths = [0.0_f32; 3];

    groups[0].push(boxed(tray_button(scheme)));
    widths[0] = TRAY_BUTTON;

    let mut push = |index: usize, widget: Box<dyn Widget>, natural: f32| {
        let group = match positions[index].as_str() {
            "center" => 1,
            "right" => 2,
            _ => 0,
        };

        if !groups[group].is_empty() {
            widths[group] += ITEM_GAP;
        }

        groups[group].push(widget);
        widths[group] += natural;
    };

    let wifi = icon(
        glyphs::wifi(&network),
        ICON,
        if network.connected() { color } else { dim },
    );
    let natural = m3::measured(&wifi);
    push(0, boxed(wifi), natural);

    if bluetooth.available() {
        let connected = bluetooth.devices().iter().any(|device| device.connected());
        let glyph = icon(
            glyphs::bluetooth(true, bluetooth.powered(), connected, false),
            ICON,
            if bluetooth.powered() { color } else { dim },
        );
        let natural = m3::measured(&glyph);
        push(1, boxed(glyph), natural);
    }

    let silent = audio.muted() || audio.volume() == 0;
    let volume = icon(
        glyphs::volume(audio.volume(), audio.muted()),
        ICON,
        if silent { dim } else { color },
    );
    let natural = m3::measured(&volume);
    push(2, boxed(volume), natural);

    if audio.microphone_muted() {
        let mic = icon(glyphs::MIC_OFF, ICON, scheme.error);
        let natural = m3::measured(&mic);
        push(3, boxed(mic), natural);
    }

    if battery.present() {
        let percent = battery.percent();
        let charging = battery.charging();
        let tint = if charging {
            scheme.primary
        } else if percent <= 15 {
            scheme.error
        } else if percent <= 30 {
            scheme.tertiary
        } else {
            color
        };

        let glyph = icon(glyphs::battery(percent, charging), ICON, tint);
        let label = text(format!("{percent}%"), 12.0, tint, amane::Weight::Medium);
        let natural = m3::measured(&glyph) + 4.0 + m3::measured(&label);

        push(
            4,
            boxed(
                Row::new(vec![boxed(glyph), boxed(label)])
                    .gap(4.0)
                    .align(Center),
            ),
            natural,
        );
    }

    // only groups that hold something take part, so no empty slot adds space
    let visible: Vec<usize> = (0..3).filter(|&index| !groups[index].is_empty()).collect();

    let content = visible.iter().map(|&index| widths[index]).sum::<f32>()
        + ITEM_GAP * visible.len().saturating_sub(1) as f32;

    let rows: Vec<Box<dyn Widget>> = visible
        .iter()
        .map(|&index| {
            boxed(
                Row::new(groups[index].drain(..).collect())
                    .gap(ITEM_GAP)
                    .align(Center),
            )
        })
        .collect();

    let row = Row::new(rows)
        .gap(ITEM_GAP)
        .align(Center)
        .width(Parent)
        .height(Parent);

    let row = if visible == [1] {
        row.justify(Center)
    } else {
        row.justify(amane::SpaceBetween)
    };

    let padded = Rectangle::new()
        .width(Parent)
        .height(Parent)
        .padding(amane::Padding {
            top: 0.0,
            right: EDGE,
            bottom: 0.0,
            left: EDGE,
        })
        .child(row);

    (boxed(padded), content + EDGE * 2.0)
}

// the right island lights up while the control center is open
pub fn status_fill(scheme: &Scheme) -> Color {
    let open = Overlay::read().panel() == Panel::Control;

    let progress = motion::number(
        "status-open",
        if open { 1.0 } else { 0.0 },
        motion::SELECT_DURATION,
    );

    let state = motion::hover("status-island");

    let base = mix(
        scheme.surface_container_high,
        scheme.primary_container,
        progress,
    );

    mix(base, scheme.on_surface, 0.08 * state)
}

pub fn base_fill(scheme: &Scheme) -> Color {
    scheme.surface_container_high
}

fn tray_button(scheme: &Scheme) -> Rectangle {
    let open = Tray::read().open();

    let state = motion::hover("tray-chevron");

    let fill = mix(Color::TRANSPARENT, scheme.on_surface_variant, 0.14 * state);

    let degrees = motion::number(
        "tray-chevron",
        if open { 180.0 } else { 0.0 },
        motion::SELECT_DURATION,
    );

    Rectangle::new()
        .width(28.0)
        .height(28.0)
        .fill(fill)
        .radius(14.0)
        .cursor(Pointer)
        .on_hover(|inside| motion::set_hover("tray-chevron", inside))
        .on_click(|button| {
            if button == Button::Left {
                Tray::toggle();
            }
        })
        .align_child(Center, Center)
        .child(
            Rectangle::new()
                .width(22.0)
                .height(22.0)
                .rotate(degrees)
                .align_child(Center, Center)
                .child(icon(EXPAND_MORE, 20.0, scheme.on_surface_variant)),
        )
}

// the focused workspace widens into a pill while the others stay dots
fn dot_width(workspace: &Workspace) -> f32 {
    let target = if workspace.focused() { FOCUSED } else { DOT };

    motion::number(
        &format!("workspace:{}", workspace.id()),
        target,
        motion::WORKSPACE_DURATION,
    )
}

fn dot_height(workspace: &Workspace) -> f32 {
    motion::number(
        &format!("workspace-height:{}", workspace.id()),
        if workspace.focused() { FOCUSED_HEIGHT } else { DOT },
        motion::WORKSPACE_DURATION,
    )
}

fn dot(workspace: &Workspace, width: f32, height: f32, scheme: &Scheme) -> Rectangle {
    let target = if workspace.focused() {
        scheme.primary
    } else if workspace.urgent() {
        scheme.error
    } else {
        scheme.outline
    };

    let fill = motion::color(
        &format!("workspace-color:{}", workspace.id()),
        target,
        Duration::from_millis(280),
    );

    let id = workspace.id();

    Rectangle::new()
        .width(width)
        .height(height)
        .fill(fill)
        .radius(height / 2.0)
        .cursor(Pointer)
        .on_click(move |_| Workspaces::focus(id))
}
