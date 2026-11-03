use amane::{
    Audio, Battery, Bluetooth, Brightness, Center, Column, Media, Network, Notification,
    Notifications, Padding, Parent, Rectangle, Row, Service, Widget,
};

use crate::art::{artwork, media_labels};
use crate::m3::{self, Type, scale};
use crate::theme::Scheme;
use crate::ui::{Tab, Ui};

use super::widgets::{
    BELL, CLOSE, NEXT, PAUSE, PLAY, boxed, icon, icon_button, label, round_button,
};

pub const MEDIA_WIDTH: f32 = 344.0;
pub const MEDIA_HEIGHT: f32 = 64.0;
pub const NOTICE_WIDTH: f32 = 364.0;
pub const NOTICE_HEIGHT: f32 = 72.0;
pub const QUICK_WIDTH: f32 = 424.0;
pub const QUICK_HEIGHT: f32 = 232.0;

pub fn quick_controls(scheme: &Scheme, alpha: f32) -> Box<dyn Widget> {
    let audio = Audio::read();
    let brightness = Brightness::read();
    let network = Network::read();
    let bluetooth = Bluetooth::read();
    let battery = Battery::read();

    let wifi_active = network.connected() || network.wifi_enabled();
    let wifi_glyph = if wifi_active { m3::WIFI } else { m3::WIFI_OFF };

    let bluetooth_active = bluetooth.powered();
    let bluetooth_glyph = if bluetooth_active {
        m3::BLUETOOTH
    } else {
        m3::BLUETOOTH_OFF
    };

    let mic_active = !audio.microphone_muted();
    let mic_glyph = if mic_active { m3::MIC } else { m3::MIC_OFF };

    let header = Row::new(vec![
        boxed(scale(
            Type::TitleMedium,
            "Controles rápidos",
            scheme.on_surface,
        )),
        boxed(icon_button(
            "quick-close",
            CLOSE,
            32.0,
            scheme,
            Ui::close_panel,
        )),
    ])
    .gap(8.0)
    .align(Center)
    .justify(amane::SpaceBetween)
    .width(Parent);

    let wifi = m3::toggle_icon_button(
        "quick-wifi",
        wifi_glyph,
        46.0,
        wifi_active,
        scheme,
        move || Network::set_wifi(!wifi_active),
    );

    let blue = m3::toggle_icon_button(
        "quick-bluetooth",
        bluetooth_glyph,
        46.0,
        bluetooth_active,
        scheme,
        move || Bluetooth::set_powered(!bluetooth_active),
    );

    let mic = m3::toggle_icon_button(
        "quick-mic",
        mic_glyph,
        46.0,
        mic_active,
        scheme,
        move || Audio::toggle_microphone_mute(),
    );

    let battery_chip = if battery.present() {
        let percent = battery.percent();
        let color = if percent <= 15 {
            scheme.error
        } else {
            scheme.on_surface_variant
        };

        let glyph = if battery.charging() {
            m3::BATTERY_CHARGING
        } else {
            m3::BATTERY
        };

        Rectangle::new()
            .width(84.0)
            .height(46.0)
            .fill(scheme.surface_container_highest)
            .radius(23.0)
            .align_child(Center, Center)
            .child(
                Row::new(vec![
                    boxed(icon(glyph, 18.0, color)),
                    boxed(scale(Type::LabelMedium, format!("{percent}%"), color)),
                ])
                .gap(6.0)
                .align(Center),
            )
    } else {
        Rectangle::new().width(0.0).height(1.0)
    };

    let switches = Row::new(vec![boxed(wifi), boxed(blue), boxed(mic)])
        .gap(10.0)
        .align(Center);

    let mut toggle_items: Vec<Box<dyn Widget>> = vec![boxed(switches)];

    if battery.present() {
        toggle_items.push(boxed(battery_chip));
    }

    let toggles = Row::new(toggle_items)
        .gap(10.0)
        .align(Center)
        .justify(amane::SpaceBetween)
        .width(Parent);

    let volume_glyph = if audio.muted() || audio.volume() == 0 {
        m3::MUTED
    } else {
        m3::VOLUME
    };

    let volume = m3::slider(
        "quick:volume",
        volume_glyph,
        f32::from(audio.volume()) / 100.0,
        QUICK_WIDTH - 32.0,
        scheme,
        |fraction| Audio::set_volume((fraction * 100.0).round() as u8),
    );

    let mut sliders: Vec<Box<dyn Widget>> = vec![boxed(volume)];

    if brightness.present() {
        sliders.push(boxed(m3::slider(
            "quick:brightness",
            m3::BRIGHTNESS,
            f32::from(brightness.percent()) / 100.0,
            QUICK_WIDTH - 32.0,
            scheme,
            |fraction| Brightness::set((fraction * 100.0).round() as u8),
        )));
    }

    boxed(
        Rectangle::new()
            .width(Parent)
            .height(Parent)
            .opacity(alpha)
            .padding(Padding {
                top: 16.0,
                right: 16.0,
                bottom: 16.0,
                left: 16.0,
            })
            .child(
                Column::new(vec![
                    boxed(header),
                    boxed(toggles),
                    boxed(Column::new(sliders).gap(8.0).width(Parent)),
                ])
                .gap(14.0)
                .width(Parent)
                .height(Parent),
            ),
    )
}

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
