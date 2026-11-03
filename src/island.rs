mod compact;
mod dash;
mod peek;
pub(crate) mod widgets;

use std::time::Duration;

use amane::{
    Button, Center, Column, End, Full, Horizontal, InputArea, Key, Keyboard, Layer, LayerWindow,
    Margin, Media, Monitor, Notification, Notifications, Padding, Parent, Pointer, Rectangle,
    Service, Shadow, Stack, Start, Vertical, Widget, Zone, request_frame,
};

use crate::motion;
use crate::overlay::{Overlay, Panel};
use crate::theme::{Scheme, Theme, fade};
use crate::tray::{self, Tray};
use crate::ui::Ui;

const RESERVED: f32 = 52.0;
const BAR_HEIGHT: f32 = 40.0;
const MARGIN: i32 = 10;
const NOTICE_TIME: Duration = Duration::from_secs(5);

// the tray sheet rounds like a card, not like the pill the bar uses
const CENTER_WIDTH: f32 = 760.0;
const CENTER_HEIGHT: f32 = 480.0;

const LEFT_WIDTH: f32 = 340.0;
const LEFT_HEIGHT: f32 = 400.0;

const RIGHT_WIDTH: f32 = 340.0;
const RIGHT_HEIGHT: f32 = 400.0;

const TRAY_RADIUS: f32 = 26.0;
const TRAY_PADDING: f32 = 8.0;

// what the center island is showing, which decides its size and how content fades
#[derive(Clone, Copy, PartialEq, Eq)]
enum IslandMode {
    Compact = 0, // the clock
    Media = 1,   // the hover peek for the current player
    Notice = 2,  // a notification arriving
    Dash = 3,    // the full dash
}

pub fn reserve(_: &Monitor) -> LayerWindow {
    LayerWindow::new()
        .width(Full)
        .height(RESERVED)
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
        .child(Stack::new(vec![bar_row]).width(Parent).height(Parent));

    LayerWindow::new()
        .width(LEFT_WIDTH)
        .height(LEFT_HEIGHT)
        .anchor_vertical(Vertical::Top)
        .anchor_horizontal(Horizontal::Left)
        .layer(Layer::Top)
        .space(Zone::Ignore)
        .margin(Margin {
            top: MARGIN,
            left: MARGIN,
            ..Margin::default()
        })
        .namespace("cds-left")
        .input_region(vec![InputArea {
            x: 0,
            y: 0,
            width: width.ceil() as i32,
            height: height.ceil() as i32,
        }])
        .child(
            Rectangle::new()
                .width(Parent)
                .height(Parent)
                .align_child(Start, Start)
                .child(island),
        )
}

pub fn right(monitor: &Monitor) -> LayerWindow {
    let theme = Theme::read();
    let scheme = theme.scheme();

    let tray_open = Tray::read().open();
    let (status, status_width) = compact::status(scheme);

    let (content, width, height, radius): (Box<dyn Widget>, f32, f32, f32) = if tray_open {
        let (menu, menu_width, menu_height) = tray::menu(scheme);

        let column = Column::new(vec![Box::new(status), menu])
            .align(End)
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

    LayerWindow::new()
        .width(RIGHT_WIDTH)
        .height(RIGHT_HEIGHT)
        .anchor_vertical(Vertical::Top)
        .anchor_horizontal(Horizontal::Right)
        .layer(Layer::Top)
        .space(Zone::Ignore)
        .margin(Margin {
            top: MARGIN,
            right: MARGIN,
            ..Margin::default()
        })
        .namespace("cds-right")
        .input_region(vec![InputArea {
            x: (RIGHT_WIDTH - width).max(0.0) as i32,
            y: 0,
            width: width.ceil() as i32,
            height: height.ceil() as i32,
        }])
        .child(
            Rectangle::new()
                .width(Parent)
                .height(Parent)
                .align_child(End, Start)
                .child(island),
        )
}

pub fn view(monitor: &Monitor) -> LayerWindow {
    let theme = Theme::read();
    let scheme = theme.scheme();
    let ui = Ui::read();
    let notifications = Notifications::read();
    let has_media = Media::read().active().is_some();

    let notice = recent(&notifications);

    let mode = if ui.dash() {
        IslandMode::Dash
    } else if notice.is_some() {
        IslandMode::Notice
    } else if ui.hover() && has_media {
        IslandMode::Media
    } else {
        IslandMode::Compact
    };

    if mode == IslandMode::Notice {
        request_frame();
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

        IslandMode::Notice => {
            let notification = notice.expect("the notice mode only starts with a notice");

            (
                peek::notice(scheme, notification, alpha),
                peek::NOTICE_WIDTH,
                peek::NOTICE_HEIGHT,
                peek::NOTICE_HEIGHT / 2.0,
            )
        }

        IslandMode::Media => (
            peek::media(scheme, alpha),
            peek::MEDIA_WIDTH,
            peek::MEDIA_HEIGHT,
            peek::MEDIA_HEIGHT / 2.0,
        ),

        IslandMode::Compact => {
            let (content, width) = compact::clock(scheme, alpha);

            (content, width, BAR_HEIGHT, BAR_HEIGHT / 2.0)
        }
    };

    let (width, height, radius) = motion::follow(&monitor.name, width, height, radius);

    let target = if mode == IslandMode::Dash {
        scheme.surface_container
    } else {
        scheme.surface_container_high
    };

    let fill = motion::color("island-fill", target, motion::SELECT_DURATION);

    let mut island = surface(width, height, radius, fill, scheme).on_hover(Ui::set_hover);

    // clicking the island itself: keep the click inside the dash, or open it
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
        .layer(Layer::Top)
        .space(Zone::Ignore)
        .margin(Margin {
            top: MARGIN,
            ..Margin::default()
        })
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
                .align_child(Center, Start)
                .on_click(|button| {
                    if button == Button::Left {
                        Ui::close();
                    }
                })
                .child(island),
        )
}

fn recent(notifications: &Notifications) -> Option<&Notification> {
    notifications.list().iter().rfind(|notification| {
        notification
            .received()
            .elapsed()
            .is_ok_and(|age| age < NOTICE_TIME)
    })
}
