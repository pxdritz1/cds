use std::path::Path as FsPath;

use amane::{
    Apps, Center, Column, Image, Notification, Notifications, Padding, Parent,
    Rectangle, Row, Service, Widget,
};

use crate::m3;

use crate::theme::Scheme;
use super::widgets::{
    BELL, CLOSE, boxed, icon, icon_button, label,
};

pub const NOTICE_COMPACT_MIN_WIDTH: f32 = 104.0;
pub const NOTICE_COMPACT_MAX_WIDTH: f32 = 700.0;
pub const NOTICE_COMPACT_HEIGHT: f32 = 40.0;
pub const NOTICE_WIDTH: f32 = 364.0;
pub const NOTICE_HEIGHT: f32 = 72.0;

/// Fit the compact notification island to its title while keeping room for the app icon and padding.
pub fn notice_compact_width(notification: &Notification) -> f32 {
    let title = notification.summary();
    let measured_text = m3::text(title.to_string(), 13.0, amane::Color::TRANSPARENT, amane::Weight::SemiBold);
    let measured = m3::measured(&measured_text);
    let title_width = if measured > 0.0 {
        measured + 4.0
    } else {
        title.chars().map(|c| {
            if c.is_whitespace() || "ilI.,'!:;|".contains(c) { 3.8 }
            else if "MW@#%&".contains(c) { 10.0 }
            else if c.is_ascii_uppercase() { 8.2 }
            else if c.is_ascii() { 7.0 }
            else { 8.5 }
        }).sum::<f32>() + 4.0
    };

    (title_width + 32.0 + 10.0 + 28.0)
        .max(NOTICE_COMPACT_MIN_WIDTH)
        .min(NOTICE_COMPACT_MAX_WIDTH)
}

pub fn notice_compact(
    scheme: &Scheme,
    notification: &Notification,
    alpha: f32,
) -> Box<dyn Widget> {
    let id = notification.id();
    let avatar = notification_avatar(scheme, notification, 32.0);
    let title = label(
        notification.summary(),
        13.0,
        scheme.on_surface,
        amane::Weight::SemiBold,
        1,
    );

    let row = Row::new(vec![boxed(avatar), boxed(title)])
        .gap(10.0)
        .align(Center)
        .justify(Center)
        .width(Parent)
        .height(Parent);

    boxed(
        Rectangle::new()
            .width(Parent)
            .height(Parent)
            .opacity(alpha)
            .padding(Padding {
                top: 0.0,
                right: 14.0,
                bottom: 0.0,
                left: 14.0,
            })
            .cursor(amane::Pointer)
            .on_hover(move |inside| crate::island::set_notice_hover(id, inside))
            .on_click(move |_| Notifications::click(id))
            .child(row),
    )
}

fn notification_avatar(
    scheme: &Scheme,
    notification: &Notification,
    size: f32,
) -> Rectangle {
    let image_size = (size * 1.6) as u32;

    if let Some(path) = notification_art(notification).filter(|path| Image::loaded(path)) {
        Rectangle::new()
            .width(size)
            .height(size)
            .radius(size / 2.0)
            .clip()
            .fill(Image::cover(path).thumbnail(image_size, image_size))
    } else {
        Rectangle::new()
            .width(size)
            .height(size)
            .fill(scheme.primary_container)
            .radius(size / 2.0)
            .align_child(Center, Center)
            .child(icon(BELL, size * 0.5, scheme.on_primary_container))
    }
}

pub fn notice(scheme: &Scheme, notification: &Notification, alpha: f32) -> Box<dyn Widget> {
    let id = notification.id();

    let avatar = notification_avatar(scheme, notification, 40.0);

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
        boxed(avatar),
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
            .on_hover(move |inside| crate::island::set_notice_hover(id, inside))
            .on_click(move |_| Notifications::click(id))
            .child(row),
    )
}

fn notification_art(notification: &Notification) -> Option<String> {
    if let Some(path) = local_image_path(notification.image()) {
        return Some(path);
    }

    if let Some(path) = local_image_path(notification.icon()) {
        return Some(path);
    }

    let icon_name = notification.icon();
    let app_name = notification.app_name();

    if icon_name.is_empty() && app_name.is_empty() {
        return None;
    }

    let apps = Apps::read();
    let app = apps.list().iter().find(|app| {
        let name = app.name();
        let icon = app.icon().unwrap_or("");

        (!app_name.is_empty() && name.eq_ignore_ascii_case(app_name))
            || (!icon_name.is_empty() && icon.eq_ignore_ascii_case(icon_name))
    })?;

    app.icon_path()
        .map(|path| path.to_string_lossy().into_owned())
}

fn local_image_path(value: &str) -> Option<String> {
    if value.is_empty() {
        return None;
    }

    let path = if let Some(encoded) = value.strip_prefix("file://") {
        decode_file_url_path(encoded)?
    } else {
        value.to_string()
    };

    let path = FsPath::new(&path);

    if path.is_file() {
        Some(path.to_string_lossy().into_owned())
    } else {
        None
    }
}

fn decode_file_url_path(value: &str) -> Option<String> {
    let bytes = value.as_bytes();
    let mut decoded = Vec::with_capacity(bytes.len());
    let mut index = 0;

    while index < bytes.len() {
        if bytes[index] == b'%' {
            let high = *bytes.get(index + 1)?;
            let low = *bytes.get(index + 2)?;
            let high = (high as char).to_digit(16)?;
            let low = (low as char).to_digit(16)?;
            decoded.push((high * 16 + low) as u8);
            index += 3;
        } else {
            decoded.push(bytes[index]);
            index += 1;
        }
    }

    String::from_utf8(decoded).ok()
}


