use amane::{
    Center, Column, Media, Notification, Padding, Parent, Rectangle, Row, Service, Weight, Widget,
};

use crate::theme::Scheme;
use crate::ui::{Tab, Ui};

use crate::art::cover;
use super::widgets::{BELL, NEXT, PAUSE, PLAY, boxed, icon, label, round_button};

pub const MEDIA_WIDTH: f32 = 340.0;
pub const MEDIA_HEIGHT: f32 = 58.0;
pub const NOTICE_WIDTH: f32 = 372.0;
pub const NOTICE_HEIGHT: f32 = 64.0;

pub fn media(scheme: &Scheme) -> Box<dyn Widget> {
    let media = Media::read();

    let glyph = if media.playing() { PAUSE } else { PLAY };

    let info = Column::new(vec![
        boxed(label(
            media.title(),
            14.0,
            scheme.on_surface,
            Weight::SemiBold,
            186.0,
        )),
        boxed(label(
            media.artist(),
            12.0,
            scheme.on_surface_variant,
            Weight::Regular,
            186.0,
        )),
    ]);

    let row = Row::new(vec![
        boxed(cover(40.0, 14.0, scheme)),
        boxed(info),
        boxed(round_button(
            glyph,
            38.0,
            scheme.primary,
            scheme.on_primary,
            Media::play_pause,
        )),
        boxed(round_button(
            NEXT,
            38.0,
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
            .padding(Padding {
                top: 0.0,
                right: 10.0,
                bottom: 0.0,
                left: 10.0,
            })
            .child(row),
    )
}

pub fn notice(scheme: &Scheme, notification: &Notification) -> Box<dyn Widget> {
    let bell = Rectangle::new()
        .width(40.0)
        .height(40.0)
        .fill(scheme.primary_container)
        .radius(20.0)
        .align_child(Center, Center)
        .child(icon(BELL, 18.0, scheme.on_primary_container));

    let info = Column::new(vec![
        boxed(label(
            notification.summary(),
            14.0,
            scheme.on_surface,
            Weight::SemiBold,
            280.0,
        )),
        boxed(label(
            notification.body(),
            12.0,
            scheme.on_surface_variant,
            Weight::Regular,
            280.0,
        )),
    ]);

    let row = Row::new(vec![boxed(bell), boxed(info)])
        .gap(12.0)
        .align(Center)
        .width(Parent)
        .height(Parent);

    boxed(
        Rectangle::new()
            .width(Parent)
            .height(Parent)
            .padding(Padding {
                top: 0.0,
                right: 16.0,
                bottom: 0.0,
                left: 12.0,
            })
            .cursor(amane::Pointer)
            .on_click(|_| Ui::open(Tab::Alerts))
            .child(row),
    )
}
