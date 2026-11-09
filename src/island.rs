mod compact;
mod dash;
mod peek;
pub(crate) mod widgets;

use std::collections::HashMap;
use std::sync::{Mutex, OnceLock};
use std::time::{Duration, Instant, SystemTime};

use amane::{
    Align, Button, Center, Column, End, Full, Horizontal, InputArea, Key, Keyboard, Layer, LayerWindow,
    Margin, Monitor, Notification, Notifications, Padding, Parent, Pointer, Rectangle,
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
const NOTCH_WIDTH: f32 = 260.0;
const NOTCH_HEIGHT: f32 = 44.0;
const NOTCH_RADIUS: f32 = 22.0;
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

const LEFT_HEIGHT: f32 = 400.0;

const RIGHT_HEIGHT: f32 = 400.0;

const TRAY_RADIUS: f32 = 26.0;
const TRAY_PADDING: f32 = 8.0;

// what the center island is showing, which decides its size and how content fades
#[derive(Clone, Copy, PartialEq, Eq)]
enum IslandMode {
    Compact = 0,       // the top notch and clock
    NoticeCompact = 1, // a notification arriving
    Notice = 2,        // an expanded notification preview
    Dash = 3,          // the full dash
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
    let enabled = settings.flag("show_center_island");
    drop(settings);
    if !enabled {
        return hidden_window("cds-reserve");
    }

    let ui = Ui::read();
    let notice_active = notification_preview_active() && !ui.dash();
    let reserved_height = if notice_active && ui.hover() {
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


// A notch hangs from the top edge: square upper corners, curved lower corners.
fn notch_surface(width: f32, height: f32, radius: f32, fill: amane::Color, scheme: &Scheme) -> Rectangle {
    Rectangle::new()
        .width(width)
        .height(height)
        .fill(fill)
        .radius_bottom_left(radius)
        .radius_bottom_right(radius)
        .clip()
        .shadow(
            Shadow::drop(scheme.shadow)
                .opacity(0.38)
                .blur(24.0)
                .offset(0.0, 8.0),
        )
        .border(1.0, fade(scheme.outline_variant, 80))
}


fn side_surface(width: f32, height: f32, fill: amane::Color, scheme: &Scheme, right_side: bool) -> Rectangle {
    let surface = Rectangle::new()
        .width(width)
        .height(height)
        .fill(fill);
    let surface = if right_side {
        surface.radius_bottom_left(NOTCH_RADIUS)
    } else {
        surface.radius_bottom_right(NOTCH_RADIUS)
    };
    surface
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
    let enabled = Settings::read().flag("show_workspaces");
    if !enabled {
        return hidden_window("cds-left");
    }

    let theme = Theme::read();
    let scheme = theme.scheme();
    let (bar, bar_width) = compact::workspaces(monitor, scheme);
    let side_width = ((monitor.width as f32 - NOTCH_WIDTH) / 2.0).max(0.0);

    let island = side_surface(
        side_width,
        NOTCH_HEIGHT,
        compact::base_fill(scheme),
        scheme,
        false,
    )
    .child(
        Rectangle::new()
            .width(Parent)
            .height(Parent)
            .padding(Padding { top: 0.0, right: 8.0, bottom: 0.0, left: 10.0 })
            .align_child(Align::from(Start), Center)
            .child(Stack::new(vec![bar]).width(Parent).height(Parent)),
    );

    LayerWindow::new()
        .width(Full)
        .height(LEFT_HEIGHT)
        .anchor_vertical(Vertical::Top)
        .layer(Layer::Top)
        .space(Zone::Ignore)
        .namespace("cds-left")
        .input_region(vec![InputArea {
            x: 0,
            y: 0,
            width: (bar_width + 20.0).min(side_width).ceil() as i32,
            height: NOTCH_HEIGHT.ceil() as i32,
        }])
        .child(
            Rectangle::new()
                .width(Parent)
                .height(Parent)
                .align_child(Align::from(Start), Start)
                .child(island),
        )
}

pub fn right(monitor: &Monitor) -> LayerWindow {
    let settings = Settings::read();
    let enabled = settings.flag("show_status");
    drop(settings);
    if !enabled {
        return hidden_window("cds-right");
    }

    let theme = Theme::read();
    let scheme = theme.scheme();
    let tray_open = Tray::read().open();
    let tray_count = Tray::read().items().len();
    let (status, status_width) = compact::status(scheme);
    let side_align = Align::from(End);

    let (content, content_width, height): (Box<dyn Widget>, f32, f32) = if tray_open {
        let (menu, menu_width, menu_height) = tray::menu(scheme);
        let menu_slot = Rectangle::new().width(Parent).height(menu_height);
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
                        right: 10.0,
                        bottom: TRAY_PADDING,
                        left: TRAY_PADDING,
                    })
                    .child(column),
            ),
            menu_width.max(status_width) + TRAY_PADDING * 2.0,
            BAR_HEIGHT + TRAY_PADDING + menu_height + TRAY_PADDING,
        )
    } else {
        (status, status_width, BAR_HEIGHT)
    };

    let side_width = ((monitor.width as f32 - NOTCH_WIDTH) / 2.0).max(0.0);
    let surface_height = height.max(NOTCH_HEIGHT);
    let island = side_surface(
        side_width,
        surface_height,
        compact::status_fill(scheme),
        scheme,
        true,
    )
    .cursor(Pointer)
    .on_hover(|inside| motion::set_hover("status-island", inside))
    .on_click(|button| {
        if button == Button::Left {
            Overlay::toggle(Panel::Control);
        }
    })
    .child(
        Rectangle::new()
            .width(Parent)
            .height(Parent)
            .padding(Padding { top: 0.0, right: 10.0, bottom: 0.0, left: 8.0 })
            .align_child(Align::from(End), Center)
            .child(Stack::new(vec![content]).width(Parent).height(Parent)),
    );

    LayerWindow::new()
        .width(Full)
        .height(RIGHT_HEIGHT)
        .anchor_vertical(Vertical::Top)
        .layer(Layer::Top)
        .space(Zone::Ignore)
        .keyboard(if tray_open { Keyboard::OnDemand } else { Keyboard::None })
        .namespace("cds-right")
        .input_region(vec![InputArea {
            x: (monitor.width as f32 - content_width - 20.0).max(side_width).max(0.0) as i32,
            y: 0,
            width: (content_width + 20.0).min(side_width).ceil() as i32,
            height: surface_height.ceil() as i32,
        }])
        .on_key(|key| {
            if key == Key::Escape {
                Tray::close();
            }
        })
        .child(
            Rectangle::new()
                .width(Parent)
                .height(Parent)
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
                .align_child(Align::from(End), Start)
                .child(island),
        )
}

pub fn view(monitor: &Monitor) -> LayerWindow {
    let enabled = Settings::read().flag("show_center_island");
    if !enabled {
        return hidden_window("cds-island");
    }

    let horizontal = Horizontal::Middle;
    let side_margin = Margin::default();

    let theme = Theme::read();
    let scheme = theme.scheme();
    let notifications = Notifications::read();
    let notice = recent(&notifications);
    let ui = Ui::read();

    let mode = if ui.dash() {
        IslandMode::Dash
    } else if notice.is_some() && ui.hover() {
        IslandMode::Notice
    } else if notice.is_some() {
        IslandMode::NoticeCompact
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
                peek::notice_compact_width(notification),
                peek::NOTICE_COMPACT_HEIGHT,
                peek::NOTICE_COMPACT_HEIGHT / 2.0,
            ),
            None => {
                let (content, _) = compact::clock(scheme, alpha);
                (content, NOTCH_WIDTH, NOTCH_HEIGHT, NOTCH_RADIUS)
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

        IslandMode::Compact => {
            let (content, _) = compact::clock(scheme, alpha);

            (content, NOTCH_WIDTH, NOTCH_HEIGHT, NOTCH_RADIUS)
        }
    };

    let (width, height, radius) = motion::follow(&monitor.name, width, height, radius);
    let (width, height, radius) = (width, height, radius);

    let target = if mode == IslandMode::Dash {
        scheme.surface_container
    } else {
        scheme.surface_container_high
    };

    let fill = motion::color("island-fill", target, motion::SELECT_DURATION);

    let mut island = if mode == IslandMode::Compact {
        notch_surface(width, height, NOTCH_RADIUS, fill, scheme)
    } else {
        surface(width, height, radius, fill, scheme)
    }
    .on_hover(Ui::set_hover);

    if ui.dash() {
        island = island.on_click(|_| {});
    } else {
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
        (((CENTER_WIDTH - width) / 2.0).max(0.0), width)
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
            y: if overlay_open { 0 } else { 0 },
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
                .padding(Padding { top: 0.0, right: 0.0, bottom: 0.0, left: 0.0 })
                .align_child(Center, Start)
                .on_click(move |button| {
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
