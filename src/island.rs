mod compact;
mod dash;
mod peek;
pub(crate) mod widgets;

use std::time::Duration;

use amane::{
    Center, Full, InputArea, Key, Keyboard, Layer, LayerWindow, Margin, Media, Monitor,
    Notification, Notifications, Parent, Rectangle, Service, Shadow, Stack, Start, Vertical,
    Widget, Zone, request_frame,
};

use crate::motion;
use crate::theme::{Theme, fade};
use crate::ui::Ui;

const WINDOW_WIDTH: f32 = 760.0;
const WINDOW_HEIGHT: f32 = 480.0;
const MARGIN: i32 = 10;
const RESERVED: f32 = 44.0;
const COMPACT_HEIGHT: f32 = 40.0;
const NOTICE_TIME: Duration = Duration::from_secs(5);

// what the island is showing, which decides its size and how content fades
#[derive(Clone, Copy, PartialEq, Eq)]
enum Mode {
    Compact = 0,
    Media = 1,
    Notice = 2,
    Dash = 3,
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

pub fn view(monitor: &Monitor) -> LayerWindow {
    let theme = Theme::read();
    let scheme = theme.scheme();
    let ui = Ui::read();
    let notifications = Notifications::read();
    let has_media = Media::read().active().is_some();

    let notice = recent(&notifications);

    let mode = if ui.dash() {
        Mode::Dash
    } else if notice.is_some() {
        Mode::Notice
    } else if ui.hover() && has_media {
        Mode::Media
    } else {
        Mode::Compact
    };

    if mode == Mode::Notice {
        request_frame();
    }

    // content fades in when the island changes what it shows
    let alpha = motion::fade("island", mode as u64);

    let (content, width, height, radius): (Box<dyn Widget>, f32, f32, f32) = match mode {
        Mode::Dash => (
            Box::new(dash::view(scheme, ui.tab(), alpha)),
            dash::WIDTH,
            dash::HEIGHT,
            dash::RADIUS,
        ),

        Mode::Notice => {
            let notification = notice.expect("the notice mode only starts with a notice");

            (
                peek::notice(scheme, notification, alpha),
                peek::NOTICE_WIDTH,
                peek::NOTICE_HEIGHT,
                peek::NOTICE_HEIGHT / 2.0,
            )
        }

        Mode::Media => (
            peek::media(scheme, alpha),
            peek::MEDIA_WIDTH,
            peek::MEDIA_HEIGHT,
            peek::MEDIA_HEIGHT / 2.0,
        ),

        Mode::Compact => {
            let (content, width) = compact::view(monitor, scheme, alpha);

            (content, width, COMPACT_HEIGHT, COMPACT_HEIGHT / 2.0)
        }
    };

    let (width, height, radius) = motion::follow(&monitor.name, width, height, radius);

    let mut island = Rectangle::new()
        .width(width)
        .height(height)
        .fill(scheme.surface_container_high)
        .radius(radius)
        .clip()
        .shadow(
            Shadow::drop(scheme.shadow)
                .opacity(0.38)
                .blur(24.0)
                .offset(0.0, 8.0),
        )
        .border(1.0, fade(scheme.outline_variant, 80))
        .on_hover(Ui::set_hover);

    if ui.dash() {
        // keeps a click inside the surface from reaching the backdrop below
        island = island.on_click(|_| {});
    } else {
        island = island.on_click(|_| Ui::toggle());
    }

    let island = island.child(
        Stack::new(vec![content])
            .width(Parent)
            .height(Parent),
    );

    let keyboard = if ui.dash() {
        Keyboard::OnDemand
    } else {
        Keyboard::None
    };

    // the dash takes the whole window so a click outside the island closes it
    let (region_x, region_width) = if ui.dash() {
        (0.0, WINDOW_WIDTH)
    } else {
        (((WINDOW_WIDTH - width) / 2.0).max(0.0), width)
    };

    let region_height = if ui.dash() { WINDOW_HEIGHT } else { height };

    LayerWindow::new()
        .width(WINDOW_WIDTH)
        .height(WINDOW_HEIGHT)
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
                .on_click(|_| Ui::close())
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
