use amane::{
    Center, Column, Media, Notification, Notifications, Padding, Parent, Rectangle, Row, Service,
    Widget,
};

use crate::art::{artwork, media_labels};
use crate::theme::Scheme;
use crate::ui::{Tab, Ui};

use super::widgets::{
    BELL, CLOSE, NEXT, PAUSE, PLAY, boxed, icon, icon_button, label, round_button,
};

pub const MEDIA_WIDTH: f32 = 344.0;
pub const MEDIA_HEIGHT: f32 = 64.0;
pub const NOTICE_WIDTH: f32 = 364.0;
pub const NOTICE_HEIGHT: f32 = 72.0;

pub fn media(scheme: &Scheme, alpha: f32) -> Box<dyn Widget> {
    let media = Media::read();

    let playing = media.playing();

    let glyph = if playing { PAUSE } else { PLAY };

    let (title, artist) = media_labels(&media);

    let info = Column::new(vec![
        boxed(label(
            title,
            13.5,
            scheme.on_surface,
            amane::Weight::SemiBold,
            1,
        )),
        boxed(label(
            artist,
            11.5,
            scheme.on_surface_variant,
            amane::Weight::Regular,
            1,
        )),
    ])
    .gap(1.0)
    .width(172.0);

    let row = Row::new(vec![
        boxed(artwork(media.art_url(), 44.0, 14.0, scheme)),
        boxed(info),
        boxed(round_button(
            "peek-play",
            glyph,
            40.0,
            scheme.primary,
            scheme.on_primary,
            Media::play_pause,
        )),
        boxed(round_button(
            "peek-next",
            NEXT,
            34.0,
            scheme.secondary_container,
            scheme.on_secondary_container,
            Media::next,
        )),
    ])
    .gap(10.0)
    .align(Center)
    .width(Parent)
    .height(Parent);

    boxed(
        Rectangle::new()
            .width(Parent)
            .height(Parent)
            .opacity(alpha)
            .padding(Padding {
                top: 0.0,
                right: 12.0,
                bottom: 0.0,
                left: 10.0,
            })
            .child(row),
    )
}

pub fn notice(scheme: &Scheme, notification: &Notification, alpha: f32) -> Box<dyn Widget> {
    let id = notification.id();

    let bell = Rectangle::new()
        .width(40.0)
        .height(40.0)
        .fill(scheme.primary_container)
        .radius(20.0)
        .align_child(Center, Center)
        .child(icon(BELL, 20.0, scheme.on_primary_container));

    let info = Column::new(vec![
        boxed(label(
            notification.summary(),
            13.5,
            scheme.on_surface,
            amane::Weight::SemiBold,
            1,
        )),
        boxed(label(
            notification.body(),
            11.5,
            scheme.on_surface_variant,
            amane::Weight::Regular,
            2,
        )),
    ])
    .gap(1.0)
    .width(240.0);

    let row = Row::new(vec![
        boxed(bell),
        boxed(info),
        boxed(icon_button("peek-close", CLOSE, 32.0, scheme, move || {
            Notifications::dismiss(id)
        })),
    ])
    .gap(12.0)
    .align(Center)
    .width(Parent)
    .height(Parent);

    boxed(
        Rectangle::new()
            .width(Parent)
            .height(Parent)
            .opacity(alpha)
            .padding(Padding {
                top: 0.0,
                right: 12.0,
                bottom: 0.0,
                left: 12.0,
            })
            .cursor(amane::Pointer)
            .on_click(|_| Ui::open(Tab::Alerts))
            .child(row),
    )
}
