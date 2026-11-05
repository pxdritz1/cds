mod compact;
mod dash;
mod peek;
pub(crate) mod widgets;

use std::collections::HashMap;
use std::sync::{Mutex, OnceLock};
use std::time::{Duration, Instant, SystemTime};

use amane::{
    Align, Button, Center, Column, End, Full, Horizontal, InputArea, Key, Keyboard, Layer, LayerWindow,
    Margin, Media, Monitor, Notification, Notifications, Padding, Parent, Pointer, Rectangle,
    Service, Shadow, Stack, Start, Vertical, Widget, Zone,
};

use crate::motion;
use crate::settings::Settings;
use crate::overlay::{Overlay, Panel};
use crate::theme::{Scheme, Theme, fade};
use crate::tray::{self, Tray};
use crate::ui::Ui;

const RESERVED: f32 = 52.0;
const BAR_HEIGHT: f32 = 40.0;
const MARGIN: i32 = 10;
const NOTICE_TIME: Duration = Duration::from_secs(5);

struct NoticeTimer {
    remaining: Duration,
    last_tick: Instant,
    hovered: bool,
    received: SystemTime,
}

static NOTICE_TIMERS: OnceLock<Mutex<HashMap<u32, NoticeTimer>>> = OnceLock::new();

fn notice_timers() -> &'static Mutex<HashMap<u32, NoticeTimer>> {
    NOTICE_TIMERS.get_or_init(|| Mutex::new(HashMap::new()))
}

pub(crate) fn set_notice_hover(id: u32, inside: bool) {
    let mut timers = notice_timers().lock().unwrap_or_else(|error| error.into_inner());
    if let Some(timer) = timers.get_mut(&id) {
        timer.hovered = inside;
        timer.last_tick = Instant::now();
    }
}

pub(crate) fn notification_preview_active() -> bool {
    let notifications = Notifications::read();
    let timers = notice_timers().lock().unwrap_or_else(|error| error.into_inner());

    timers.iter().any(|(id, timer)| {
        timer.remaining > Duration::ZERO
            && notifications.list().iter().any(|notification| notification.id() == *id)
    })
}

// the tray sheet rounds like a card, not like the pill the bar uses
const CENTER_WIDTH: f32 = 760.0;
const CENTER_HEIGHT: f32 = 480.0;

const LEFT_WIDTH: f32 = 520.0;
const LEFT_HEIGHT: f32 = 400.0;

const RIGHT_WIDTH: f32 = 340.0;
const RIGHT_HEIGHT: f32 = 400.0;

const TRAY_RADIUS: f32 = 26.0;
const TRAY_PADDING: f32 = 8.0;

// what the center island is showing, which decides its size and how content fades
#[derive(Clone, Copy, PartialEq, Eq)]
enum IslandMode {
    Compact = 0,       // the clock
    Media = 1,         // the player-and-clock sheet
    NoticeCompact = 2, // a notification arriving
    Notice = 3,        // an expanded notification preview
    Dash = 4,          // the full dash
}

fn hidden_window(namespace: &'static str) -> LayerWindow {
    LayerWindow::new()
        .width(1.0)
        .height(1.0)
        .visible(false)
        .namespace(namespace)
}

pub fn reserve(monitor: &Monitor) -> LayerWindow {
    let settings = Settings::read();
    let enabled = settings.shell_mode() != 2 && settings.flag("show_center_island");
    drop(settings);
    if !enabled {
        return hidden_window("cds-reserve");
    }

    let ui = Ui::read();
    let media_visible = Media::read().playing()
        && !ui.media_sheet()
        && !ui.dash()
        && !notification_preview_active();
    let notice_active = notification_preview_active() && !ui.dash();
    let media_expanded = motion::hover(&format!("media-island:{}", monitor.name)) > 0.5;
    let reserved_height = if media_visible && media_expanded {
        peek::MEDIA_HEIGHT + 6.0
    } else if media_visible {
        peek::MEDIA_COMPACT_HEIGHT + 6.0
    } else if notice_active && ui.hover() {
        peek::NOTICE_HEIGHT + 6.0
    } else if notice_active {
        peek::NOTICE_COMPACT_HEIGHT + 6.0
    } else {
        RESERVED
    };

    LayerWindow::new()
        .width(Full)
        .height(reserved_height)
        .anchor_vertical(Vertical::Top)
        .layer(Layer::Bottom)
        .space(Zone::Reserve)
        .namespace("cds-reserve")
        .click_through()
        .child(Rectangle::new().width(Parent).height(Parent))
}

// the surface every island shares: fill, soft border and the drop shadow
fn surface(width: f32, height: f32, radius: f32, fill: amane::Color, scheme: &Scheme) -> Rectangle {
    Rectangle::new()
        .width(width)
        .height(height)
        .fill(fill)
        .radius(radius)
        .clip()
        .shadow(
            Shadow::drop(scheme.shadow)
                .opacity(0.38)
                .blur(24.0)
                .offset(0.0, 8.0),
        )
        .border(1.0, fade(scheme.outline_variant, 80))
}

pub fn left(monitor: &Monitor) -> LayerWindow {
    let settings = Settings::read();
    let enabled = settings.shell_mode() != 2 && settings.flag("show_workspaces");
    let alignment = String::from(settings.workspaces_alignment());
    drop(settings);
    if !enabled {
        return hidden_window("cds-left");
    }

    let horizontal = if alignment == "right" {
        Horizontal::Right
    } else {
        Horizontal::Left
    };
    let side_margin = if alignment == "right" {
        Margin { top: MARGIN, right: MARGIN, ..Margin::default() }
    } else {
        Margin { top: MARGIN, left: MARGIN, ..Margin::default() }
    };

    let theme = Theme::read();
    let scheme = theme.scheme();

    let (bar, bar_width) = compact::workspaces(monitor, scheme);

    let bar_row = Rectangle::new()
        .width(Parent)
        .height(BAR_HEIGHT)
        .child(Stack::new(vec![bar]).width(Parent).height(Parent));

    let (width, height, radius) =
        motion::follow(&format!("left:{}", monitor.name), bar_width, BAR_HEIGHT, BAR_HEIGHT / 2.0);

    let island = surface(width, height, radius, compact::base_fill(scheme), scheme)
        .child(Stack::new(vec![Box::new(bar_row)]).width(Parent).height(Parent));

    LayerWindow::new()
        .width(LEFT_WIDTH)
        .height(LEFT_HEIGHT)
        .anchor_vertical(Vertical::Top)
        .anchor_horizontal(horizontal)
        .layer(Layer::Top)
        .space(Zone::Ignore)
        .margin(side_margin)
        .namespace("cds-left")
        .input_region(vec![InputArea {
            x: if alignment == "right" {
                (LEFT_WIDTH - width).max(0.0) as i32
            } else {
                0
            },
            y: 0,
            width: width.ceil() as i32,
            height: height.ceil() as i32,
        }])
        .child(
            Rectangle::new()
                .width(Parent)
                .height(Parent)
                .align_child(if alignment == "right" { Align::from(End) } else { Align::from(Start) }, Start)
                .child(island),
        )
}

pub fn right(monitor: &Monitor) -> LayerWindow {
    let settings = Settings::read();
    let enabled = settings.shell_mode() != 2 && settings.flag("show_status");
    let alignment = String::from(settings.status_alignment());
    drop(settings);
    if !enabled {
        return hidden_window("cds-right");
    }

    let horizontal = if alignment == "left" {
        Horizontal::Left
    } else {
        Horizontal::Right
    };
    let side_margin = if alignment == "left" {
        Margin { top: MARGIN, left: MARGIN, ..Margin::default() }
    } else {
        Margin { top: MARGIN, right: MARGIN, ..Margin::default() }
    };

    let theme = Theme::read();
    let scheme = theme.scheme();

    let tray_open = Tray::read().open();
    let tray_count = Tray::read().items().len();
    let side_align = if alignment == "left" { Align::from(Start) } else { Align::from(End) };
    let (status, status_width) = compact::status(scheme);

    let (content, width, height, radius): (Box<dyn Widget>, f32, f32, f32) = if tray_open {
        let (menu, menu_width, menu_height) = tray::menu(scheme);

        // Center the single icon; keep a multi-icon tray aligned to the right.
        let menu_slot = Rectangle::new()
            .width(Parent)
            .height(menu_height);
        let menu_slot = if tray_count == 1 {
            menu_slot.align_child(Center, Center)
        } else {
            menu_slot.align_child(side_align, Center)
        }
        .child(Stack::new(vec![menu]));

        let column = Column::new(vec![status, Box::new(menu_slot)])
            .align(side_align)
            .gap(TRAY_PADDING)
            .width(Parent)
            .height(Parent);

        (
            Box::new(
                Rectangle::new()
                    .width(Parent)
                    .height(Parent)
                    .padding(Padding {
                        top: 0.0,
                        right: TRAY_PADDING,
                        bottom: TRAY_PADDING,
                        left: TRAY_PADDING,
                    })
                    .child(column),
            ),
            f32::max(status_width, menu_width + TRAY_PADDING * 2.0),
            BAR_HEIGHT + TRAY_PADDING + menu_height + TRAY_PADDING,
            TRAY_RADIUS,
        )
    } else {
        (status, status_width, BAR_HEIGHT, BAR_HEIGHT / 2.0)
    };

    let (width, height, radius) = motion::follow(&format!("right:{}", monitor.name), width, height, radius);

    let island = surface(
        width,
        height,
        radius,
        compact::status_fill(scheme),
        scheme,
    )
    .cursor(Pointer)
    .on_hover(|inside| motion::set_hover("status-island", inside))
    .on_click(|button| {
        if button == Button::Left {
            Overlay::toggle(Panel::Control);
        }
    })
    .child(Stack::new(vec![content]).width(Parent).height(Parent));

    let (region, keyboard) = (
        vec![InputArea {
            x: if alignment == "left" { 0 } else { (RIGHT_WIDTH - width).max(0.0) as i32 },
            y: 0,
            width: width.ceil() as i32,
            height: height.ceil() as i32,
        }],
        if tray_open { Keyboard::OnDemand } else { Keyboard::None },
    );

    LayerWindow::new()
        .width(RIGHT_WIDTH)
        .height(RIGHT_HEIGHT)
        .anchor_vertical(Vertical::Top)
        .anchor_horizontal(horizontal)
        .layer(Layer::Top)
        .space(Zone::Ignore)
        .margin(side_margin)
        .keyboard(keyboard)
        .namespace("cds-right")
        .input_region(region)
        .on_key(|key| {
            if key == Key::Escape {
                Tray::close();
            }
        })
        .child(
            Rectangle::new()
                .width(Parent)
                .height(Parent)
                // Leaving the tray's input area dismisses it. This does not
                // open the tray on hover; it only observes pointer departure.
                .on_hover(move |inside| {
                    if tray_open && !inside {
                        Tray::close();
                    }
                })
                .on_click(move |_| {
                    if tray_open {
                        Tray::close();
                    }
                })
                .align_child(if alignment == "left" { Align::from(Start) } else { Align::from(End) }, Start)
                .child(island),
        )
}

pub fn view(monitor: &Monitor) -> LayerWindow {
    let settings = Settings::read();
    let enabled = settings.shell_mode() != 2 && settings.flag("show_center_island");
    let alignment = String::from(settings.center_alignment());
    drop(settings);
    if !enabled {
        return hidden_window("cds-island");
    }

    let horizontal = match alignment.as_str() {
        "left" => Horizontal::Left,
        "right" => Horizontal::Right,
        _ => Horizontal::Middle,
    };
    let side_margin = match alignment.as_str() {
        "left" => Margin { top: MARGIN, left: MARGIN, ..Margin::default() },
        "right" => Margin { top: MARGIN, right: MARGIN, ..Margin::default() },
        _ => Margin { top: MARGIN, ..Margin::default() },
    };

    let theme = Theme::read();
    let scheme = theme.scheme();
    let notifications = Notifications::read();
    let has_media = Media::read().playing();
    if !has_media {
        Ui::reset_media_sheet();
    }
    let notice = recent(&notifications);

    // Never acquire the Ui write lock while a Ui read guard is alive.
    // The media sheet is click-only, so clear a stale hover before taking
    // the read guard used to build this frame.
    let clear_media_hover = {
        let ui = Ui::read();
        !ui.dash()
            && notice.is_none()
            && has_media
            && !ui.media_sheet()
            && ui.hover()
    };

    if clear_media_hover {
        Ui::set_hover(false);
    }

    let ui = Ui::read();

    let media_hover_key = format!("media-island:{}", monitor.name);
    let media_expanded = motion::hover(&media_hover_key) > 0.5;

    let mode = if ui.dash() {
        IslandMode::Dash
    } else if notice.is_some() && ui.hover() {
        IslandMode::Notice
    } else if notice.is_some() {
        IslandMode::NoticeCompact
    } else if has_media && !ui.media_sheet() {
        IslandMode::Media
    } else {
        IslandMode::Compact
    };

    if matches!(mode, IslandMode::NoticeCompact | IslandMode::Notice) {
        motion::request_frame_at_most(20);
    }

    // content fades in when the island changes what it shows
    let alpha = motion::fade("island", mode as u64);

    let (content, width, height, radius): (Box<dyn Widget>, f32, f32, f32) = match mode {
        IslandMode::Dash => (
            Box::new(dash::view(scheme, ui.tab(), alpha)),
            dash::WIDTH,
            dash::HEIGHT,
            dash::RADIUS,
        ),

        IslandMode::NoticeCompact => match notice {
            Some(notification) => (
                peek::notice_compact(scheme, notification, alpha),
                peek::NOTICE_COMPACT_WIDTH,
                peek::NOTICE_COMPACT_HEIGHT,
                peek::NOTICE_COMPACT_HEIGHT / 2.0,
            ),
            None => {
                let (content, width) = compact::clock(scheme, alpha);
                (content, width, BAR_HEIGHT, crate::m3::shape::ISLAND_COMPACT)
            }
        },

        IslandMode::Notice => match notice {
            Some(notification) => (
                peek::notice(scheme, notification, alpha),
                peek::NOTICE_WIDTH,
                peek::NOTICE_HEIGHT,
                peek::NOTICE_HEIGHT / 2.0,
            ),
            None => {
                let (content, width) = compact::clock(scheme, alpha);
                (content, width, BAR_HEIGHT, crate::m3::shape::ISLAND_COMPACT)
            }
        },

        IslandMode::Media => (
            peek::media(scheme, alpha, media_expanded),
            peek::media_width(),
            if media_expanded {
                peek::MEDIA_HEIGHT
            } else {
                peek::MEDIA_COMPACT_HEIGHT
            },
            if media_expanded {
                crate::m3::shape::ISLAND_EXPANDED
            } else {
                crate::m3::shape::ISLAND_COMPACT
            },
        ),

        IslandMode::Compact => {
            let (content, width) = compact::clock(scheme, alpha);

            (content, width, BAR_HEIGHT, crate::m3::shape::ISLAND_COMPACT)
        }
    };

    let (width, height, radius) = motion::follow(&monitor.name, width, height, radius);

    let target = if mode == IslandMode::Dash {
        scheme.surface_container
    } else {
        scheme.surface_container_high
    };

    let fill = motion::color("island-fill", target, motion::SELECT_DURATION);

    let mut island = surface(width, height, radius, fill, scheme);

    // The media sheet is click-only: hovering it never reveals the dash.
    if mode == IslandMode::Media {
        let hover_key = media_hover_key;
        island = island.on_hover(move |inside| {
            motion::set_hover(hover_key.clone(), inside);
        });
    } else {
        island = island.on_hover(Ui::set_hover);
    }

    // The player sheet has its own clock and transport controls.
    // Do not attach a parent click handler: it would swallow the clock's
    // action for switching back to the compact island.
    if ui.dash() {
        island = island.on_click(|_| {});
    } else if mode != IslandMode::Media {
        island = island.on_click(|_| Ui::toggle());
    }

    let island = island.child(Stack::new(vec![content]).width(Parent).height(Parent));

    /*
     * the dash takes the keyboard outright: a keybind opens it without a
     * click, and on demand focus would leave escape with the window that
     * had it before, so the dash would never see the key.
     */
    let keyboard = if ui.dash() {
        Keyboard::Exclusive
    } else {
        Keyboard::None
    };

    // the dash takes the whole window so a click outside the island closes it
    let overlay_open = ui.dash();

    let (region_x, region_width) = if overlay_open {
        (0.0, CENTER_WIDTH)
    } else {
        let x = match alignment.as_str() {
            "left" => 0.0,
            "right" => (CENTER_WIDTH - width).max(0.0),
            _ => ((CENTER_WIDTH - width) / 2.0).max(0.0),
        };
        (x, width)
    };

    let region_height = if overlay_open { CENTER_HEIGHT } else { height };

    LayerWindow::new()
        .width(CENTER_WIDTH)
        .height(CENTER_HEIGHT)
        .anchor_vertical(Vertical::Top)
        .anchor_horizontal(horizontal)
        .layer(Layer::Top)
        // The separate reserve surface keeps windows below the island.
        // Ignore its exclusive zone here so the island stays on the same
        // top baseline as the left and right islands.
        .space(Zone::Ignore)
        .margin(side_margin)
        .keyboard(keyboard)
        .namespace("cds-island")
        .input_region(vec![InputArea {
            x: region_x as i32,
            y: 0,
            width: region_width.ceil() as i32,
            height: region_height.ceil() as i32,
        }])
        .on_key(|key| {
            if key == Key::Escape {
                Ui::close();
            }
        })
        .child(
            Rectangle::new()
                .width(Parent)
                .height(Parent)
                .align_child(if horizontal == Horizontal::Left { Align::from(Start) } else { Align::from(End) }, Start)
                .on_click(|button| {
                    if button == Button::Left {
                        Ui::close();
                    }
                })
                .child(island),
        )
}

fn recent(notifications: &Notifications) -> Option<&Notification> {
    let now = Instant::now();
    let mut timers = notice_timers().lock().unwrap_or_else(|error| error.into_inner());

    timers.retain(|known_id, _| {
        notifications.list().iter().any(|item| item.id() == *known_id)
    });

    for notification in notifications.list().iter().rev() {
        let id = notification.id();
        let received = notification.received();
        let timer = timers.entry(id).or_insert_with(|| NoticeTimer {
            remaining: NOTICE_TIME.saturating_sub(
                received.elapsed().unwrap_or(Duration::ZERO),
            ),
            last_tick: now,
            hovered: false,
            received,
        });

        if timer.received != received {
            timer.remaining = NOTICE_TIME.saturating_sub(
                received.elapsed().unwrap_or(Duration::ZERO),
            );
            timer.last_tick = now;
            timer.hovered = false;
            timer.received = received;
        }

        let elapsed = now.saturating_duration_since(timer.last_tick);
        if !timer.hovered {
            timer.remaining = timer.remaining.saturating_sub(elapsed);
        }
        timer.last_tick = now;

        if timer.remaining > Duration::ZERO {
            return Some(notification);
        }
    }

    None
}
