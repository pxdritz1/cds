mod compact;
mod dash;
mod peek;
mod widgets;

use std::time::Duration;

use amane::{
    Center, Full, InputArea, Key, Keyboard, Layer, LayerWindow, Margin, Media, Monitor,
    Notification, Notifications, Parent, Rectangle, Service, Shadow, Stack, Start, Vertical,
    Widget, Zone, request_frame,
};

use crate::motion;
use crate::theme::{Theme, fade};
use crate::ui::Ui;

const WINDOW_WIDTH: f32 = 720.0;
const WINDOW_HEIGHT: f32 = 412.0;
const MARGIN: i32 = 8;
const RESERVED: f32 = 46.0;
const COMPACT_HEIGHT: f32 = 38.0;
const NOTICE_TIME: Duration = Duration::from_secs(5);

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

    if notice.is_some() {
        request_frame();
    }

    let (content, width, height, radius): (Box<dyn Widget>, f32, f32, f32) = if ui.dash() {
        (
            dash::view(scheme, ui.tab()),
            dash::WIDTH,
            dash::HEIGHT,
            34.0,
        )
    } else if let Some(notification) = notice {
        (
            peek::notice(scheme, notification),
            peek::NOTICE_WIDTH,
            peek::NOTICE_HEIGHT,
            32.0,
        )
    } else if ui.hover() && has_media {
        (
            peek::media(scheme),
            peek::MEDIA_WIDTH,
            peek::MEDIA_HEIGHT,
            29.0,
        )
    } else {
        let (content, width) = compact::view(monitor, scheme);

        (content, width, COMPACT_HEIGHT, COMPACT_HEIGHT / 2.0)
    };

    let (width, height, radius) = motion::follow(&monitor.name, width, height, radius);

    let island = Rectangle::new()
        .width(width)
        .height(height)
        .fill(scheme.surface_container_high)
        .radius(radius)
        .clip()
        .shadow(
            Shadow::drop(scheme.shadow)
                .opacity(0.35)
                .blur(20.0)
                .offset(0.0, 6.0),
        )
        .border(1.0, fade(scheme.outline_variant, 90))
        .on_hover(Ui::set_hover)
        .child(Stack::new(vec![content]).width(Parent).height(Parent));

    let keyboard = if ui.dash() {
        Keyboard::OnDemand
    } else {
        Keyboard::None
    };

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
            x: ((WINDOW_WIDTH - width) / 2.0).max(0.0) as i32,
            y: 0,
            width: width.ceil() as i32,
            height: height.ceil() as i32,
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
