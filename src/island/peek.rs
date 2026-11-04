use std::path::Path as FsPath;

use amane::{
    Apps, Center, Column, Image, Media, Notification, Notifications, Padding, Parent,
    Rectangle, Row, Service, Widget,
};

use crate::m3::{self, Type};

use crate::art::{artwork, media_labels};
use crate::clock::Clock;
use crate::theme::Scheme;
use crate::settings::Settings;
use super::widgets::{
    BELL, CLOSE, boxed, icon, icon_button, label,
};

pub const MEDIA_WIDTH: f32 = 300.0;
pub const MEDIA_HEIGHT: f32 = 71.0;
pub const NOTICE_COMPACT_WIDTH: f32 = 260.0;
pub const NOTICE_COMPACT_HEIGHT: f32 = 52.0;
pub const NOTICE_WIDTH: f32 = 364.0;
pub const NOTICE_HEIGHT: f32 = 72.0;

/// Calculate the media island width from visible content and the configured title limit.
pub fn media_width() -> f32 {
    let media = Media::read();
    let (title, _) = media_labels(&media);
    let settings = Settings::read();
    let show_title = settings.flag("media_show_title");
    let show_artist = settings.flag("media_show_artist");
    let show_clock = settings.flag("media_show_clock");
    let show_artwork = settings.flag("media_show_artwork");
    let max_title_width = settings.media_title_max_width() as f32;
    drop(settings);

    let info_width = media_info_width(&title, show_title, show_artist, max_title_width);
    let mut count = 2usize;
    let mut content = 64.0;
    if show_artwork { count += 1; content += 38.0; }
    if info_width > 0.0 { count += 1; content += info_width; }
    if show_clock { count += 1; content += 42.0; }
    (content + count.saturating_sub(1) as f32 * 5.0 + 20.0).max(MEDIA_WIDTH).min(740.0)
}

fn media_info_width(title: &str, show_title: bool, show_artist: bool, max_width: f32) -> f32 {
    if !show_title && !show_artist { return 0.0; }
    if !show_title { return 104.0; }
    let estimated = title.chars().map(|c| {
        if c.is_whitespace() || "ilI.,'!:;|".contains(c) { 3.8 }
        else if "MW@#%&".contains(c) { 10.0 }
        else if c.is_ascii_uppercase() { 8.2 }
        else if c.is_ascii() { 7.0 }
        else { 8.5 }
    }).sum::<f32>() + 4.0;
    estimated.max(104.0).min(max_width.max(104.0))
}

fn media_title_label(content: String, size: f32, color: amane::Color, weight: amane::Weight, width: f32) -> Rectangle {
    Rectangle::new().width(width).height(18.0).clip()
        .align_child(amane::Start, Center)
        .child(m3::text(content, size, color, weight))
}

pub fn media(scheme: &Scheme, alpha: f32) -> Box<dyn Widget> {
    let media = Media::read();
    let playing = media.playing();
    let glyph = if playing { m3::PAUSE } else { m3::PLAY };
    let (title, artist) = media_labels(&media);

    let settings = Settings::read();
    let show_title = settings.flag("media_show_title");
    let show_artist = settings.flag("media_show_artist");
    let show_clock = settings.flag("media_show_clock");
    let show_artwork = settings.flag("media_show_artwork");
    let title_max_width = settings.media_title_max_width() as f32;
    drop(settings);

    let info_width = media_info_width(&title, show_title, show_artist, title_max_width);
    let mut info_lines: Vec<Box<dyn Widget>> = Vec::new();
    if show_title {
        info_lines.push(boxed(media_title_label(title, 13.0, scheme.on_surface, amane::Weight::SemiBold, info_width)));
    }
    if show_artist {
        info_lines.push(boxed(m3::scale_label(Type::LabelSmall, artist, scheme.on_surface_variant, 1)));
    }
    let info = if info_lines.is_empty() {
        None
    } else {
        Some(boxed(Column::new(info_lines).gap(1.0).width(info_width)))
    };

    let clock_view = if show_clock {
        let clock = Clock::read();
        let clock_column = Column::new(vec![
            boxed(m3::scale_label(Type::LabelMedium, clock.time(true), scheme.on_surface, 1)),
            boxed(m3::scale_label(Type::LabelSmall, clock.short_date(), scheme.on_surface_variant, 1)),
        ]).gap(1.0).width(42.0).align(Center);
        Some(boxed(Rectangle::new().width(42.0).height(46.0).align_child(Center, Center).child(clock_column)))
    } else { None };

    let mut row_children: Vec<Box<dyn Widget>> = Vec::new();
    if show_artwork { row_children.push(boxed(artwork(media.art_url(), 38.0, 12.0, scheme))); }
    if let Some(info) = info { row_children.push(info); }
    row_children.push(boxed(m3::round_button("peek-play", glyph, 34.0, scheme.primary, scheme.on_primary, Media::play_pause)));
    row_children.push(boxed(m3::round_button("peek-next", m3::NEXT, 30.0, scheme.secondary_container, scheme.on_secondary_container, Media::next)));
    if let Some(clock_view) = clock_view { row_children.push(clock_view); }

    let row = Row::new(row_children).gap(5.0).align(Center).width(Parent).height(46.0);

    let position = media.position().as_secs();
    let length = media.length().as_secs();
    let fraction = if length > 0 {
        position as f32 / length as f32
    } else {
        0.0
    };

    let timeline = Row::new(vec![
        boxed(m3::scale_label(
            Type::LabelSmall,
            format_time(position),
            scheme.on_surface_variant,
            1,
        )),
        boxed(m3::progress_line(
            fraction,
            154.0,
            scheme.primary,
            scheme.surface_container_highest,
        )),
        boxed(m3::scale_label(
            Type::LabelSmall,
            format_time(length),
            scheme.on_surface_variant,
            1,
        )),
    ])
    .gap(6.0)
    .align(Center)
    .justify(Center)
    .width(Parent)
    .height(18.0);

    let content = Column::new(vec![boxed(row), boxed(timeline)])
        .gap(1.0)
        .width(Parent)
        .height(Parent);

    boxed(
        Rectangle::new()
            .width(Parent)
            .height(Parent)
            .opacity(alpha)
            .padding(Padding {
                top: 3.0,
                right: 10.0,
                bottom: 3.0,
                left: 10.0,
            })
            .child(content),
    )
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




fn marquee_label(
    content: String,
    size: f32,
    color: amane::Color,
    weight: amane::Weight,
) -> Rectangle {
    const VIEWPORT: f32 = 104.0;

    Rectangle::new()
        .width(VIEWPORT)
        .height(18.0)
        .clip()
        .align_child(amane::Start, Center)
        .child(m3::text(content, size, color, weight))
}

fn format_time(seconds: u64) -> String {
    format!("{:02}:{:02}", seconds / 60, seconds % 60)
}
