/*
 * the control center: the island that drops from the right of the bar.
 * the main page has the toggles (each with its own detail page), the
 * sliders, the player and the notifications; wifi and bluetooth open full
 * pages with the networks and devices around.
 */
use std::cell::{Cell, RefCell};

use amane::{
    Bluetooth, Brightness, Button, Center, Column, Key, LayerWindow, Media, Monitor,
    Network, Notification, Notifications, Padding, Parent, Rectangle, Row, ScrollArea, Service,
    Widget,
};

use crate::art::{artwork, media_labels};
use crate::glyphs;
use crate::m3::{
    self, Item, Kind, Tile, Type, boxed, icon, icon_button, leading_icon, list_item, round_button,
    scale, scale_label, slider, switch, tile,
};
use crate::motion;
use crate::overlay::{self, Frame, Overlay, Panel, Place};
use crate::settings::Settings;
use crate::theme::{Scheme, Theme, fade};
use crate::ui::Ui;

const WIDTH: f32 = 400.0;
const HEIGHT: f32 = 620.0;
const PADDING: f32 = 16.0;
const TOP: f32 = 24.0;

// inner width the tiles and sliders share
const BODY: f32 = WIDTH - PADDING * 2.0;
const HALF: f32 = (BODY - 8.0) / 2.0;
const INNER_HEIGHT: f32 = HEIGHT - TOP - PADDING;

const PASSWORD_FIELD: &str = "ctl-wifi-password";

#[derive(Clone, Copy, PartialEq, Eq)]
enum Page {
    Main,
    Wifi,
    Bluetooth,
    Output,
    Input,
}

impl Page {
    fn index(self) -> u64 {
        match self {
            Page::Main => 0,
            Page::Wifi => 1,
            Page::Bluetooth => 2,
            Page::Output => 3,
            Page::Input => 4,
        }
    }
}

thread_local! {
    static PAGE: Cell<Page> = const { Cell::new(Page::Main) };
    static PENDING: RefCell<Option<String>> = const { RefCell::new(None) };
    static PASSWORD: RefCell<String> = const { RefCell::new(String::new()) };
}

fn page() -> Page {
    PAGE.with(Cell::get)
}

fn go(next: Page) {
    let before = page();

    if before == next {
        return;
    }

    PAGE.with(|page| page.set(next));
    PENDING.with(|pending| *pending.borrow_mut() = None);
    PASSWORD.with(|password| password.borrow_mut().clear());

    if before == Page::Bluetooth {
        Bluetooth::stop_scan();
    }

    match next {
        Page::Wifi => Network::scan(),
        Page::Bluetooth => Bluetooth::start_scan(),
        Page::Output | Page::Input | Page::Main => {}
    }

    Ui::refresh();
}

pub fn view(monitor: &Monitor) -> LayerWindow {
    let theme = Theme::read();
    let scheme = theme.scheme();

    let open = Overlay::read().panel() == Panel::Control;

    // the next opening starts on the main page again
    if !open && page() != Page::Main {
        go(Page::Main);
    }

    let body: Box<dyn Widget> = match page() {
        Page::Main => main_page(scheme),
        Page::Wifi => wifi_page(scheme),
        Page::Bluetooth => bluetooth_page(scheme),
        Page::Output => audio_devices_page(scheme, true),
        Page::Input => audio_devices_page(scheme, false),
    };

    let alpha = motion::fade("control-page", page().index());

    let content = Rectangle::new()
        .width(Parent)
        .height(Parent)
        .opacity(alpha)
        .padding(Padding {
            top: TOP,
            right: PADDING,
            bottom: PADDING,
            left: PADDING,
        })
        .child(body_stack(body));

    overlay::layer(
        "control",
        "cds-control",
        &monitor.name,
        open,
        scheme,
        Frame {
            width: WIDTH,
            height: HEIGHT,
            radius: m3::shape::EXTRA_LARGE,
            place: Place::Right,
        },
        content,
        |key| {
            if key == Key::Escape {
                if page() == Page::Main {
                    Overlay::close();
                } else {
                    go(Page::Main);
                }
            }
        },
    )
}

fn body_stack(body: Box<dyn Widget>) -> amane::Stack {
    amane::Stack::new(vec![body]).width(Parent).height(Parent)
}

fn spacer() -> Rectangle {
    Rectangle::new().width(Parent).height(1.0)
}

// ---- main page ------------------------------------------------------------

fn main_page(scheme: &Scheme) -> Box<dyn Widget> {
    let mut used = 0.0;
    let mut sections: Vec<Box<dyn Widget>> = Vec::new();
    let gap = 12.0;

    // header
    sections.push(boxed(
        Row::new(vec![
            boxed(scale(Type::TitleLarge, "Central", scheme.on_surface)),
            boxed(spacer()),
            boxed(round_button(
                "ctl:config",
                m3::SETTINGS,
                40.0,
                scheme.surface_container_highest,
                scheme.on_surface,
                || Overlay::open(Panel::Config),
            )),
            boxed(round_button(
                "ctl:power",
                m3::POWER,
                40.0,
                scheme.error_container,
                scheme.on_error_container,
                || Overlay::open(Panel::Power),
            )),
        ])
        .gap(8.0)
        .align(Center)
        .width(Parent),
    ));
    used += 40.0 + gap;

    // toggles
    sections.push(boxed(toggles(scheme)));
    used += 64.0 * 3.0 + 8.0 * 2.0 + gap;

    // sliders
    let (sliders, slider_height) = sliders(scheme);
    sections.push(boxed(sliders));
    used += slider_height + gap;

    // the player, only when something is open
    if Media::read().active().is_some() {
        sections.push(boxed(player(scheme)));
        used += 72.0 + gap;
    }

    // notifications fill what is left
    let list_height = (INNER_HEIGHT - used - 32.0).max(80.0);
    sections.push(boxed(notices(scheme, list_height)));

    boxed(Column::new(sections).gap(gap).width(Parent).height(Parent))
}

fn toggles(scheme: &Scheme) -> Column {
    let (wifi_on, wifi_subtitle, wifi_glyph) = {
        let network = Network::read();

        let subtitle = if !network.wifi_enabled() {
            String::from("Desligado")
        } else if network.connecting() {
            String::from("Conectando…")
        } else if network.connected() {
            if network.ssid().is_empty() {
                String::from("Conectado")
            } else {
                String::from(network.ssid())
            }
        } else {
            String::from("Sem rede")
        };

        (network.wifi_enabled(), subtitle, glyphs::wifi(&network))
    };

    let (bt_available, bt_on, bt_subtitle, bt_glyph) = {
        let bluetooth = Bluetooth::read();

        let connected = bluetooth
            .devices()
            .iter()
            .find(|device| device.connected())
            .map(|device| String::from(device.name()));

        let subtitle = if !bluetooth.available() {
            String::from("Sem adaptador")
        } else if !bluetooth.powered() {
            String::from("Desligado")
        } else {
            connected.clone().unwrap_or_else(|| String::from("Ligado"))
        };

        (
            bluetooth.available(),
            bluetooth.powered(),
            subtitle,
            glyphs::bluetooth(
                bluetooth.available(),
                bluetooth.powered(),
                connected.is_some(),
                false,
            ),
        )
    };

    let (output_muted, output_volume, mic_muted, mic_volume) = {
        let audio = amane::Audio::read();

        (audio.muted(), audio.volume(), audio.microphone_muted(), audio.microphone_volume())
    };

    let dark = Settings::read().dark();
    let dnd = Settings::read().flag("dnd");

    let wifi = tile(
        Tile {
            key: "ctl:wifi",
            glyph: wifi_glyph,
            title: "Wi-Fi",
            subtitle: &wifi_subtitle,
            active: wifi_on,
        },
        HALF,
        scheme,
        move || Network::set_wifi(!wifi_on),
        Some(Box::new(|| go(Page::Wifi))),
    );

    let bluetooth = tile(
        Tile {
            key: "ctl:bt",
            glyph: bt_glyph,
            title: "Bluetooth",
            subtitle: &bt_subtitle,
            active: bt_on,
        },
        HALF,
        scheme,
        move || {
            if bt_available {
                Bluetooth::set_powered(!bt_on);
            }
        },
        Some(Box::new(|| go(Page::Bluetooth))),
    );

    let sound = tile(
        Tile {
            key: "ctl:sound",
            glyph: glyphs::volume(output_volume, output_muted),
            title: "Som",
            subtitle: &if output_muted { String::from("Silenciado") } else { format!("{output_volume}%") },
            active: !output_muted,
        },
        HALF,
        scheme,
        amane::Audio::toggle_mute,
        Some(Box::new(|| go(Page::Output))),
    )
    .on_scroll(|scroll| {
        let volume = i32::from(amane::Audio::read().volume());
        let step = if scroll.y < 0.0 { 5 } else { -5 };
        amane::Audio::set_volume((volume + step).clamp(0, 100) as u8);
    });

    let mic = tile(
        Tile {
            key: "ctl:mic",
            glyph: glyphs::microphone(mic_muted),
            title: "Microfone",
            subtitle: &if mic_muted {
                String::from("Desligado")
            } else {
                format!("{mic_volume}%")
            },
            active: !mic_muted,
        },
        HALF,
        scheme,
        amane::Audio::toggle_microphone_mute,
        Some(Box::new(|| go(Page::Input))),
    )
    .on_scroll(|scroll| {
        let volume = i32::from(amane::Audio::read().microphone_volume());
        let step = if scroll.y < 0.0 { 5 } else { -5 };
        amane::Audio::set_microphone_volume((volume + step).clamp(0, 100) as u8);
    });

    let silence = tile(
        Tile {
            key: "ctl:dnd",
            glyph: if dnd { m3::BELL_OFF } else { m3::BELL },
            title: "Não perturbe",
            subtitle: if dnd { "Ligado" } else { "Desligado" },
            active: dnd,
        },
        HALF,
        scheme,
        || Settings::toggle_flag("dnd"),
        None,
    );

    let theme = tile(
        Tile {
            key: "ctl:dark",
            glyph: if dark { m3::DARK_MODE } else { m3::LIGHT_MODE },
            title: "Tema",
            subtitle: if dark { "Escuro" } else { "Claro" },
            active: dark,
        },
        HALF,
        scheme,
        move || Settings::set_dark_live(!dark),
        None,
    );

    Column::new(vec![
        boxed(Row::new(vec![boxed(wifi), boxed(bluetooth)]).gap(8.0)),
        boxed(Row::new(vec![boxed(sound), boxed(mic)]).gap(8.0)),
        boxed(Row::new(vec![boxed(theme), boxed(silence)]).gap(8.0)),
    ])
    .gap(8.0)
    .width(Parent)
}

fn audio_devices_page(scheme: &Scheme, output: bool) -> Box<dyn Widget> {
    use std::process::Command;

    let title = if output { "Saídas de som" } else { "Entradas de som" };
    let list_arg = if output { "sinks" } else { "sources" };
    let default_arg = if output { "get-default-sink" } else { "get-default-source" };
    let set_arg = if output { "set-default-sink" } else { "set-default-source" };

    let default = Command::new("pactl")
        .arg(default_arg)
        .output()
        .ok()
        .filter(|result| result.status.success())
        .map(|result| String::from_utf8_lossy(&result.stdout).trim().to_string())
        .unwrap_or_default();

    let devices = Command::new("pactl")
        .args(["list", "short", list_arg])
        .output()
        .ok()
        .filter(|result| result.status.success())
        .map(|result| {
            String::from_utf8_lossy(&result.stdout)
                .lines()
                .filter_map(|line| {
                    let mut fields = line.split_whitespace();
                    let _index = fields.next()?;
                    let name = fields.next()?.to_string();
                    if !output && name.ends_with(".monitor") {
                        return None;
                    }
                    Some(name)
                })
                .collect::<Vec<_>>()
        })
        .unwrap_or_default();

    let mut rows: Vec<Box<dyn Widget>> = vec![boxed(
        Row::new(vec![
            boxed(icon_button("ctl:audio-back", m3::BACK, 36.0, scheme, || go(Page::Main))),
            boxed(scale(Type::TitleMedium, title, scheme.on_surface)),
        ])
        .gap(10.0)
        .align(Center),
    )];

    for device in &devices {
        let selected = *device == default;
        let label = if selected { format!("✓  {device}") } else { device.clone() };
        let device_name = device.clone();
        let command = set_arg;
        rows.push(boxed(
            Rectangle::new()
                .width(Parent)
                .height(48.0)
                .fill(if selected { scheme.secondary_container } else { scheme.surface_container_high })
                .radius(14.0)
                .cursor(amane::Pointer)
                .on_click(move |_| {
                    let _ = Command::new("pactl").args([command, device_name.as_str()]).spawn();
                })
                .padding(12.0)
                .align_child(amane::Start, Center)
                .child(scale(Type::LabelMedium, label, if selected { scheme.on_secondary_container } else { scheme.on_surface })),
        ));
    }

    if devices.is_empty() {
        rows.push(boxed(scale(Type::BodyMedium, "Nenhum dispositivo encontrado pelo pactl", scheme.on_surface_variant)));
    }

    boxed(ScrollArea::new(
        if output { "ctl-audio-outputs" } else { "ctl-audio-inputs" },
        Column::new(rows).gap(8.0).width(Parent),
    ).width(Parent).height(Parent))
}

fn sliders(scheme: &Scheme) -> (Column, f32) {
    let (volume, muted) = {
        let audio = amane::Audio::read();

        (audio.volume(), audio.muted())
    };

    let mut rows: Vec<Box<dyn Widget>> = vec![boxed(slider(
        "ctl:volume",
        glyphs::volume(volume, muted),
        f32::from(volume) / 100.0,
        BODY,
        scheme,
        |fraction| amane::Audio::set_volume((fraction * 100.0).round() as u8),
    ))];

    let mut height = 44.0;

    let brightness = Brightness::read();

    if brightness.present() {
        let percent = brightness.percent();

        rows.push(boxed(slider(
            "ctl:brightness",
            glyphs::brightness(percent),
            f32::from(percent) / 100.0,
            BODY,
            scheme,
            |fraction| Brightness::set((fraction * 100.0).round() as u8),
        )));

        height += 44.0 + 6.0;
    }

    (Column::new(rows).gap(6.0).width(Parent), height)
}

fn player(scheme: &Scheme) -> Rectangle {
    let media = Media::read();

    let (title, artist) = media_labels(&media);
    let playing = media.playing();
    let art = artwork(media.art_url(), 48.0, 14.0, scheme);

    drop(media);

    Rectangle::new()
        .width(Parent)
        .height(72.0)
        .fill(scheme.surface_container_high)
        .radius(m3::shape::LARGE_PLUS)
        .padding(Padding {
            top: 0.0,
            right: 12.0,
            bottom: 0.0,
            left: 12.0,
        })
        .child(
            Row::new(vec![
                boxed(art),
                boxed(
                    Column::new(vec![
                        boxed(scale_label(Type::TitleSmall, title, scheme.on_surface, 1)),
                        boxed(scale_label(
                            Type::LabelSmall,
                            artist,
                            scheme.on_surface_variant,
                            1,
                        )),
                    ])
                    .width(Parent),
                ),
                boxed(icon_button(
                    "ctl:previous",
                    m3::PREVIOUS,
                    36.0,
                    scheme,
                    Media::previous,
                )),
                boxed(round_button(
                    "ctl:play",
                    if playing { m3::PAUSE } else { m3::PLAY },
                    44.0,
                    scheme.primary,
                    scheme.on_primary,
                    Media::play_pause,
                )),
                boxed(icon_button("ctl:next", m3::NEXT, 36.0, scheme, Media::next)),
            ])
            .gap(8.0)
            .align(Center)
            .width(Parent)
            .height(Parent),
        )
}

fn notices(scheme: &Scheme, height: f32) -> Column {
    let list: Vec<Notification> = Notifications::read().list().to_vec();

    let header = Row::new(vec![
        boxed(scale(Type::TitleSmall, "Notificações", scheme.on_surface)),
        boxed(spacer()),
        boxed(icon_button(
            "ctl:clear",
            m3::CLEAR_ALL,
            32.0,
            scheme,
            || {
                Notifications::clear();
            },
        )),
    ])
    .align(Center)
    .width(Parent);

    let body: Box<dyn Widget> = if list.is_empty() {
        boxed(
            Rectangle::new()
                .width(Parent)
                .height(height)
                .fill(scheme.surface_container_low)
                .radius(m3::shape::LARGE_PLUS)
                .align_child(Center, Center)
                .child(scale(
                    Type::LabelLarge,
                    "Nada por aqui",
                    scheme.on_surface_variant,
                )),
        )
    } else {
        let rows: Vec<Box<dyn Widget>> = list
            .iter()
            .rev()
            .map(|notification| boxed(notice_row(notification, scheme)))
            .collect();

        boxed(
            ScrollArea::new("ctl-notices", Column::new(rows).gap(6.0).width(BODY))
                .width(BODY)
                .height(height),
        )
    };

    Column::new(vec![boxed(header), body])
        .gap(4.0)
        .width(Parent)
}

fn notice_row(notification: &Notification, scheme: &Scheme) -> Rectangle {
    let id = notification.id();

    let summary = String::from(if notification.summary().is_empty() {
        notification.app_name()
    } else {
        notification.summary()
    });

    Rectangle::new()
        .width(BODY)
        .height(58.0)
        .fill(scheme.surface_container_high)
        .radius(m3::shape::LARGE)
        .padding(Padding {
            top: 0.0,
            right: 6.0,
            bottom: 0.0,
            left: 14.0,
        })
        .child(
            Row::new(vec![
                boxed(
                    Column::new(vec![
                        boxed(scale_label(Type::LabelLarge, summary, scheme.on_surface, 1)),
                        boxed(scale_label(
                            Type::BodySmall,
                            String::from(notification.body()),
                            fade(scheme.on_surface_variant, 220),
                            1,
                        )),
                    ])
                    .width(Parent),
                ),
                boxed(icon_button(
                    &format!("ctl:dismiss:{id}"),
                    m3::CLOSE,
                    32.0,
                    scheme,
                    move || Notifications::dismiss(id),
                )),
            ])
            .gap(4.0)
            .align(Center)
            .width(Parent)
            .height(Parent),
        )
}

// ---- shared bits of the detail pages --------------------------------------

fn subheader(title: &str, trailing: Vec<Box<dyn Widget>>, scheme: &Scheme) -> Row {
    let mut children: Vec<Box<dyn Widget>> = vec![
        boxed(icon_button("ctl:back", m3::BACK, 40.0, scheme, || {
            go(Page::Main)
        })),
        boxed(scale(Type::TitleLarge, title, scheme.on_surface)),
        boxed(spacer()),
    ];

    children.extend(trailing);

    Row::new(children).gap(8.0).align(Center).width(Parent)
}

fn empty_note(text: &str, height: f32, scheme: &Scheme) -> Rectangle {
    Rectangle::new()
        .width(Parent)
        .height(height)
        .fill(scheme.surface_container_low)
        .radius(m3::shape::LARGE_PLUS)
        .align_child(Center, Center)
        .child(scale(Type::LabelLarge, text, scheme.on_surface_variant))
}

// ---- wifi page ------------------------------------------------------------

struct Access {
    ssid: String,
    strength: u8,
    secured: bool,
    active: bool,
    saved: bool,
}

fn access_points() -> Vec<Access> {
    let network = Network::read();

    let mut list: Vec<Access> = Vec::new();

    for point in network.access_points() {
        if point.ssid().is_empty() {
            continue;
        }

        if let Some(existing) = list.iter_mut().find(|item| item.ssid == point.ssid()) {
            if point.strength() > existing.strength || point.active() {
                existing.strength = point.strength();
            }

            existing.active |= point.active();
            existing.saved |= point.saved();

            continue;
        }

        list.push(Access {
            ssid: String::from(point.ssid()),
            strength: point.strength(),
            secured: point.secured(),
            active: point.active(),
            saved: point.saved(),
        });
    }

    list.sort_by(|a, b| {
        b.active
            .cmp(&a.active)
            .then(b.saved.cmp(&a.saved))
            .then(b.strength.cmp(&a.strength))
    });

    list
}

fn wifi_page(scheme: &Scheme) -> Box<dyn Widget> {
    let enabled = Network::read().wifi_enabled();
    let connecting = Network::read().connecting();

    let header = subheader(
        "Wi-Fi",
        vec![
            boxed(icon_button(
                "ctl:wifi-scan",
                m3::REFRESH,
                40.0,
                scheme,
                Network::scan,
            )),
            boxed(switch(
                "ctl:wifi-switch",
                enabled,
                scheme,
                Network::set_wifi,
            )),
        ],
        scheme,
    );

    let pending = PENDING.with(|pending| pending.borrow().clone());

    let mut rows: Vec<Box<dyn Widget>> = Vec::new();

    if let Some(ssid) = pending.clone() {
        rows.push(boxed(password_card(&ssid, scheme)));
    }

    let points = access_points();

    for point in &points {
        let key = format!("ctl:ap:{}", point.ssid);

        let supporting = if point.active {
            if connecting {
                "Conectando…"
            } else {
                "Conectado"
            }
        } else if point.saved {
            "Salva"
        } else if point.secured {
            "Protegida"
        } else {
            "Aberta"
        };

        let leading = leading_icon(
            glyphs::wifi_level(point.strength),
            if point.active {
                scheme.primary
            } else {
                scheme.surface_container_highest
            },
            if point.active {
                scheme.on_primary
            } else {
                scheme.on_surface_variant
            },
        );

        let trailing: Option<Box<dyn Widget>> = if point.secured && !point.active {
            Some(boxed(icon(m3::LOCK, 18.0, scheme.on_surface_variant)))
        } else if point.active {
            Some(boxed(icon(m3::CHECK, 20.0, scheme.primary)))
        } else {
            None
        };

        let ssid = point.ssid.clone();
        let (active, saved, secured) = (point.active, point.saved, point.secured);

        rows.push(boxed(list_item(
            Item {
                key: &key,
                headline: &point.ssid,
                supporting: Some(supporting),
                selected: point.active,
                height: 60.0,
            },
            Some(boxed(leading)),
            trailing,
            BODY,
            scheme,
            move |button| {
                if button != Button::Left {
                    return;
                }

                if active {
                    Network::disconnect();
                } else if saved || !secured {
                    Network::connect(&ssid, None);
                } else {
                    PENDING.with(|pending| *pending.borrow_mut() = Some(ssid.clone()));
                    PASSWORD.with(|password| password.borrow_mut().clear());
                    amane::TextInput::set_text(PASSWORD_FIELD, "");
                    Ui::refresh();
                }
            },
        )));
    }

    let list_height = INNER_HEIGHT - 40.0 - 12.0;

    let body: Box<dyn Widget> = if !enabled {
        boxed(empty_note("Wi-Fi desligado", list_height, scheme))
    } else if rows.is_empty() {
        boxed(empty_note("Procurando redes…", list_height, scheme))
    } else {
        boxed(
            ScrollArea::new("ctl-wifi", Column::new(rows).gap(4.0).width(BODY))
                .width(BODY)
                .height(list_height),
        )
    };

    boxed(
        Column::new(vec![boxed(header), body])
            .gap(12.0)
            .width(Parent),
    )
}

fn password_card(ssid: &str, scheme: &Scheme) -> Rectangle {
    let target = String::from(ssid);

    let field = amane::TextInput::new(PASSWORD_FIELD)
        .size(15.0)
        .color(scheme.on_surface)
        .placeholder("Senha")
        .password()
        .focused()
        .on_change(|text| PASSWORD.with(|password| *password.borrow_mut() = text))
        .on_submit({
            let target = target.clone();

            move |text| join(&target, &text)
        });

    let input = Rectangle::new()
        .width(Parent)
        .height(48.0)
        .fill(scheme.surface_container_highest)
        .radius(m3::shape::LARGE)
        .padding(Padding {
            top: 0.0,
            right: 14.0,
            bottom: 0.0,
            left: 14.0,
        })
        .align_child(amane::Start, Center)
        .child(field);

    let actions = Row::new(vec![
        boxed(spacer()),
        boxed(m3::button(
            "ctl:wifi-cancel",
            "Cancelar",
            None,
            Kind::Text,
            scheme,
            || {
                PENDING.with(|pending| *pending.borrow_mut() = None);
                Ui::refresh();
            },
        )),
        boxed(m3::button(
            "ctl:wifi-join",
            "Conectar",
            None,
            Kind::Filled,
            scheme,
            move || {
                let text = PASSWORD.with(|password| password.borrow().clone());

                join(&target, &text);
            },
        )),
    ])
    .gap(8.0)
    .align(Center)
    .width(Parent);

    Rectangle::new()
        .width(BODY)
        .height(128.0)
        .fill(scheme.secondary_container)
        .radius(m3::shape::LARGE_PLUS)
        .padding(14.0)
        .child(
            Column::new(vec![
                boxed(scale(
                    Type::TitleSmall,
                    format!("Entrar em {ssid}"),
                    scheme.on_secondary_container,
                )),
                boxed(input),
                boxed(actions),
            ])
            .gap(8.0)
            .width(Parent),
        )
}

fn join(ssid: &str, password: &str) {
    if password.is_empty() {
        return;
    }

    Network::connect(ssid, Some(password));

    PENDING.with(|pending| *pending.borrow_mut() = None);
    PASSWORD.with(|password| password.borrow_mut().clear());

    Ui::refresh();
}

// ---- bluetooth page -------------------------------------------------------

struct Device {
    path: String,
    name: String,
    icon: String,
    paired: bool,
    connected: bool,
    battery: Option<u8>,
}

fn device_glyph(name: &str) -> &'static str {
    if name.contains("headset") || name.contains("headphone") {
        "\u{e310}"
    } else if name.contains("phone") {
        "\u{e32c}"
    } else if name.contains("mouse") {
        "\u{e323}"
    } else if name.contains("keyboard") {
        m3::KEYBOARD
    } else if name.contains("speaker") || name.contains("audio") {
        "\u{e32d}"
    } else {
        m3::BLUETOOTH
    }
}

fn bluetooth_page(scheme: &Scheme) -> Box<dyn Widget> {
    let (available, powered, scanning, mut devices) = {
        let bluetooth = Bluetooth::read();

        let devices: Vec<Device> = bluetooth
            .devices()
            .iter()
            .map(|device| Device {
                path: String::from(device.path()),
                name: String::from(device.name()),
                icon: String::from(device.icon()),
                paired: device.paired(),
                connected: device.connected(),
                battery: device.battery(),
            })
            .collect();

        (
            bluetooth.available(),
            bluetooth.powered(),
            bluetooth.scanning(),
            devices,
        )
    };

    devices.sort_by(|a, b| {
        b.connected
            .cmp(&a.connected)
            .then(b.paired.cmp(&a.paired))
            .then(a.name.cmp(&b.name))
    });

    let header = subheader(
        "Bluetooth",
        vec![
            boxed(icon_button(
                "ctl:bt-scan",
                if scanning { m3::CLOSE } else { m3::REFRESH },
                40.0,
                scheme,
                move || {
                    if scanning {
                        Bluetooth::stop_scan();
                    } else {
                        Bluetooth::start_scan();
                    }
                },
            )),
            boxed(switch("ctl:bt-switch", powered, scheme, move |on| {
                if available {
                    Bluetooth::set_powered(on);

                    if on {
                        Bluetooth::start_scan();
                    }
                }
            })),
        ],
        scheme,
    );

    let list_height = INNER_HEIGHT - 40.0 - 12.0;

    let body: Box<dyn Widget> = if !available {
        boxed(empty_note("Sem adaptador bluetooth", list_height, scheme))
    } else if !powered {
        boxed(empty_note("Bluetooth desligado", list_height, scheme))
    } else if devices.is_empty() {
        boxed(empty_note("Procurando dispositivos…", list_height, scheme))
    } else {
        let rows: Vec<Box<dyn Widget>> = devices
            .iter()
            .map(|device| boxed(device_row(device, scheme)))
            .collect();

        boxed(
            ScrollArea::new("ctl-bt", Column::new(rows).gap(4.0).width(BODY))
                .width(BODY)
                .height(list_height),
        )
    };

    boxed(
        Column::new(vec![boxed(header), body])
            .gap(12.0)
            .width(Parent),
    )
}

fn device_row(device: &Device, scheme: &Scheme) -> Rectangle {
    let key = format!("ctl:dev:{}", device.path);

    let supporting = match (device.connected, device.paired, device.battery) {
        (true, _, Some(level)) => format!("Conectado · {level}%"),
        (true, _, None) => String::from("Conectado"),
        (false, true, _) => String::from("Pareado"),
        _ => String::from("Disponível"),
    };

    let leading = leading_icon(
        device_glyph(&device.icon),
        if device.connected {
            scheme.primary
        } else {
            scheme.surface_container_highest
        },
        if device.connected {
            scheme.on_primary
        } else {
            scheme.on_surface_variant
        },
    );

    let trailing: Option<Box<dyn Widget>> = if device.paired {
        let path = device.path.clone();

        Some(boxed(icon_button(
            &format!("ctl:forget:{}", device.path),
            m3::DELETE,
            32.0,
            scheme,
            move || Bluetooth::forget(&path),
        )))
    } else {
        None
    };

    let path = device.path.clone();
    let (connected, paired) = (device.connected, device.paired);

    list_item(
        Item {
            key: &key,
            headline: &device.name,
            supporting: Some(&supporting),
            selected: device.connected,
            height: 60.0,
        },
        Some(boxed(leading)),
        trailing,
        BODY,
        scheme,
        move |button| {
            if button != Button::Left {
                return;
            }

            if connected {
                Bluetooth::disconnect(&path);
            } else if paired {
                Bluetooth::connect(&path);
            } else {
                Bluetooth::pair(&path);
            }
        },
    )
}
