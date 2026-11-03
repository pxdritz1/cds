/*
 * the control center: the right side island of the dank bar, loosely.
 * tiles for wifi, bluetooth, Não perturbe and the dark mode, sliders
 * for volume and brightness, the battery face and the recent
 * notifications with a clear all.
 */
use amane::{
    Battery, Bluetooth, Center, Column, Key, LayerWindow, Monitor, Padding, Parent, Rectangle, Row,
    ScrollArea, Service, Widget,
};

use crate::m3::{self, Tile, Type, boxed, icon, round_button, scale, scale_label, slider, tile};
use crate::overlay::{self, Frame, Overlay, Panel, Place};
use crate::settings::Settings;
use crate::theme::{Scheme, Theme, fade};

const WIDTH: f32 = 400.0;
const HEIGHT: f32 = 620.0;
const PADDING: f32 = 16.0;

// inner width the tiles and sliders share
const BODY: f32 = WIDTH - PADDING * 2.0;
const HALF: f32 = (BODY - 8.0) / 2.0;

pub fn view(monitor: &Monitor) -> LayerWindow {
    let theme = Theme::read();
    let scheme = theme.scheme();

    let open = Overlay::read().panel() == Panel::Control;

    // ---- the tiles, each with its decision taken while the data is read ---

    let wifi_enabled = amane::Network::read().wifi_enabled();

    let wifi_subtitle = {
        let network = amane::Network::read();

        if !network.wifi_enabled() {
            String::from("desligado")
        } else if network.connecting() {
            String::from("conectando…")
        } else if network.connected() {
            let ssid = String::from(network.ssid());

            if ssid.is_empty() {
                format!("força {}", network.strength())
            } else {
                format!("{ssid} · força {}", network.strength())
            }
        } else {
            String::from("sem rede")
        }
    };

    let bt_available = Bluetooth::read().available();

    let bt_powered = Bluetooth::read().powered();

    let bt_subtitle = {
        let bluetooth = Bluetooth::read();

        if !bt_available {
            String::from("sem adaptador")
        } else if !bt_powered {
            String::from("desligado")
        } else {
            bluetooth
                .devices()
                .iter()
                .find(|device| device.connected())
                .map_or(String::from("nenhum pareado"), |device| {
                    format!("{}", device.name())
                })
        }
    };

    let dark = Settings::read().dark();

    let dnd_on = Settings::read().flag("dnd");

    let top_row = Row::new(vec![
        boxed(tile(
            Tile {
                key: "ctl:wifi",
                glyph: if wifi_enabled { m3::WIFI } else { m3::WIFI_OFF },
                title: "Wi-Fi",
                subtitle: &wifi_subtitle,
                active: wifi_enabled,
            },
            HALF,
            scheme,
            move || {
                amane::Network::set_wifi(!wifi_enabled);
            },
            None,
        )),
        boxed(tile(
            Tile {
                key: "ctl:bt",
                glyph: m3::BLUETOOTH,
                title: "Bluetooth",
                subtitle: &bt_subtitle,
                active: bt_powered,
            },
            HALF,
            scheme,
            move || {
                Bluetooth::set_powered(!bt_powered);
            },
            None,
        )),
    ])
    .gap(8.0)
    .width(Parent);

    let bottom_row = Row::new(vec![
        boxed(tile(
            Tile {
                key: "ctl:dnd",
                glyph: m3::DND,
                title: "Não perturbe",
                subtitle: if dnd_on { "silêncio total" } else { "normal" },
                active: dnd_on,
            },
            HALF,
            scheme,
            || Settings::toggle_flag("dnd"),
            None,
        )),
        boxed(tile(
            Tile {
                key: "ctl:dark",
                glyph: if dark { m3::LIGHT_MODE } else { m3::DARK_MODE },
                title: if dark {
                    "Vou pro claro"
                } else {
                    "Vou pro escuro"
                },
                subtitle: "tema do sistema",
                active: !dark,
            },
            HALF,
            scheme,
            move || Settings::set_dark_live(!dark),
            None,
        )),
    ])
    .gap(8.0)
    .width(Parent);

    // ---- sliders -----------------------------------------------------------

    let volume = amane::Audio::read().volume();

    let volume_glyph = if volume == 0 {
        m3::MUTED
    } else if volume < 48 {
        m3::VOLUME_LOW
    } else {
        m3::VOLUME
    };

    let volume_slider = {
        slider(
            "ctl:volume",
            volume_glyph,
            volume as f32 / 100.0,
            BODY,
            scheme,
            move |fraction| {
                amane::Audio::set_volume((fraction * 100.0).round() as u8);
            },
        )
    };

    let brightness = amane::Brightness::read().percent();

    let brightness_slider = {
        slider(
            "ctl:brightness",
            m3::BRIGHTNESS,
            brightness as f32 / 100.0,
            BODY,
            scheme,
            move |fraction| {
                amane::Brightness::set((fraction * 100.0).round() as u8);
            },
        )
    };

    // ---- battery -----------------------------------------------------------

    let battery = Battery::read();

    let (level, charging, present) = (battery.percent(), battery.charging(), battery.present());

    drop(battery);

    let battery_line = Row::new(vec![
        boxed(icon(m3::BATTERY, 18.0, scheme.primary)),
        boxed(scale_label(
            Type::LabelMedium,
            if present {
                format!(
                    "bateria {level}%{}",
                    if charging { " · carregando" } else { "" }
                )
            } else {
                String::from("sem bateria interna")
            },
            scheme.on_surface_variant,
            1,
        )),
    ])
    .gap(6.0)
    .align(Center);

    // ---- the notifications list -------------------------------------------

    let list = amane::Notifications::read().list().to_vec();

    let notice_rows: Vec<Box<dyn Widget>> = list
        .iter()
        .rev()
        .take(4)
        .map(|notification| boxed(notice_row(notification, scheme)))
        .collect();

    let notice_body: Box<dyn Widget> = if notice_rows.is_empty() {
        boxed(scale_label(
            Type::LabelMedium,
            "Sem notificações por enquanto",
            scheme.on_surface_variant,
            1,
        ))
    } else {
        boxed(
            ScrollArea::new(
                "ctl-notices",
                Column::new(notice_rows).gap(4.0).width(Parent),
            )
            .width(Parent)
            .height(150.0),
        )
    };

    let notices = Column::new(vec![
        boxed(
            Row::new(vec![
                boxed(scale(Type::TitleSmall, "Notificações", scheme.on_surface)),
                boxed(Rectangle::new().width(Parent).height(1.0)),
                boxed(m3::icon_button(
                    "ctl:clear",
                    m3::CLEAR_ALL,
                    32.0,
                    scheme,
                    || {
                        amane::Notifications::clear();
                    },
                )),
            ])
            .gap(6.0)
            .align(Center)
            .width(Parent),
        ),
        notice_body,
    ])
    .gap(4.0)
    .width(Parent);

    // ---- the header --------------------------------------------------------

    let header = Row::new(vec![
        boxed(scale(Type::TitleLarge, "Central", scheme.on_surface)),
        boxed(Rectangle::new().width(Parent).height(1.0)),
        boxed(round_button(
            "ctl:config",
            m3::SETTINGS,
            36.0,
            scheme.secondary_container,
            scheme.on_secondary_container,
            || Overlay::open(Panel::Config),
        )),
        boxed(round_button(
            "ctl:close",
            m3::CLOSE,
            36.0,
            scheme.surface_container_highest,
            scheme.on_surface,
            || Overlay::close(),
        )),
    ])
    .gap(6.0)
    .align(Center)
    .width(Parent);

    let content = Rectangle::new()
        .width(Parent)
        .height(Parent)
        .padding(Padding {
            top: PADDING + 14.0,
            right: PADDING,
            bottom: PADDING,
            left: PADDING,
        })
        .child(
            Column::new(vec![
                Box::new(header),
                Box::new(divider(scheme)),
                Box::new(top_row),
                Box::new(bottom_row),
                Box::new(volume_slider),
                Box::new(brightness_slider),
                Box::new(battery_line),
                Box::new(divider(scheme)),
                Box::new(notices),
            ])
            .width(Parent)
            .height(Parent)
            .gap(10.0),
        );

    overlay::layer(
        "control",
        "cds-control",
        &monitor.name,
        open,
        scheme,
        Frame {
            width: WIDTH,
            height: HEIGHT,
            radius: m3::shape::XL,
            place: Place::Right,
        },
        content,
        |key| {
            if key == Key::Escape {
                Overlay::close();
            }
        },
    )
}

// little helper keeping the section borders drawn the same way
fn divider(scheme: &Scheme) -> Rectangle {
    Rectangle::new()
        .width(Parent)
        .height(1.0)
        .fill(scheme.outline_variant)
}

fn notice_row(notification: &amane::Notification, scheme: &Scheme) -> amane::Column {
    let summary = String::from(if notification.summary().is_empty() {
        notification.app_name()
    } else {
        notification.summary()
    });

    let body = String::from(notification.body());

    Column::new(vec![
        Box::new(scale_label(
            Type::LabelMedium,
            summary,
            scheme.on_surface,
            1,
        )),
        Box::new(scale_label(
            Type::LabelSmall,
            body,
            fade(scheme.on_surface_variant, 210),
            1,
        )),
    ])
    .gap(1.0)
    .width(Parent)
    .height(44.0)
    .justify(Center)
}
