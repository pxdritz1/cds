/*
 * a layered lock screen: blurred wallpaper, a centered digital clock and
 * password field, with the analogue clock, weather and session controls
 * held to the corners so the composition adapts to each output.
 */
use std::f32::consts::TAU;
use std::sync::atomic::{AtomicBool, AtomicUsize, Ordering};
use std::time::Duration;

use amane::{
    Arc, Canvas, Cap, Center, Column, End, Full, Horizontal, Image, Keyboard, Layer,
    LayerWindow, Line, Lock, Media, Monitor, Notifications, Padding, Parent, Pointer,
    Rectangle, Row, Service, Shape, Stack, TextInput, Vertical, request_frame,
};

use crate::clock::Clock;
use crate::m3::{self, Type, scale, scale_label};
use crate::power::Power;
use crate::settings::Settings;
use crate::theme::{Theme, fade};
use crate::weather::Weather;

const PASSWORD: &str = "cds-lockscreen-password";

static PASSWORD_LENGTH: AtomicUsize = AtomicUsize::new(0);
static POWER_MENU_OPEN: AtomicBool = AtomicBool::new(false);

pub fn view(monitor: &Monitor) -> LayerWindow {
    let theme = Theme::read();
    let scheme = theme.scheme();
    let shortest = monitor.width.min(monitor.height) as f32;
    let ui_scale = (shortest / 900.0).clamp(0.76, 1.12);

    let clock = Clock::read();
    let lock = Lock::read();
    let failed = lock.failed();
    let checking = lock.checking();
    drop(lock);

    let settings = Settings::read();
    let wallpaper = settings.wallpaper().to_owned();
    drop(settings);

    let user = std::env::var("USER").unwrap_or_else(|_| String::from("Usuária"));
    let initial = user
        .chars()
        .next()
        .map(|letter| letter.to_uppercase().to_string())
        .unwrap_or_else(|| String::from("C"));

    let background = Rectangle::new()
        .width(Parent)
        .height(Parent);

    let background = if wallpaper.is_empty() {
        background.fill(scheme.surface)
    } else {
        background.fill(
            Image::cover(wallpaper.as_str())
                .thumbnail(64, 36)
                .blurred(9),
        )
    };

    let scrim = Rectangle::new()
        .width(Parent)
        .height(Parent)
        .fill(fade(scheme.scrim, 112));

    let password_target = (PASSWORD_LENGTH.load(Ordering::Relaxed) as f32 * 13.0 * ui_scale).min(260.0 * ui_scale);
    let password_width = crate::motion::number(
        "lockscreen-password-progress",
        password_target,
        Duration::from_millis(190),
    );
    if (password_width - password_target).abs() > 0.5 {
        request_frame();
    }

    let (field_border, feedback_color, feedback_text) = if failed {
        (scheme.error, scheme.error, "senha incorreta")
    } else if checking {
        (scheme.primary, scheme.on_surface, "verificando…")
    } else {
        (
            fade(scheme.outline_variant, 210),
            scheme.on_surface_variant,
            "enter pra desbloquear",
        )
    };

    let password = Rectangle::new()
        .width(340.0 * ui_scale)
        .height(58.0 * ui_scale)
        .fill(fade(scheme.surface_container_highest, 238))
        .radius(20.0)
        .border(1.5, field_border)
        .padding(Padding {
            top: 0.0,
            right: 18.0 * ui_scale,
            bottom: 0.0,
            left: 18.0 * ui_scale,
        })
        .align_child(Center, Center)
        .child(
            TextInput::new(PASSWORD)
                .size(17.0 * ui_scale)
                .color(scheme.on_surface)
                .placeholder("Digite sua senha")
                .password()
                .focused()
                .on_change(|value| {
                    PASSWORD_LENGTH.store(value.chars().count(), Ordering::Relaxed);
                    request_frame();
                })
                .on_submit(|password| {
                    Lock::unlock(&password);
                    POWER_MENU_OPEN.store(false, Ordering::Relaxed);
                    TextInput::set_text(PASSWORD, "");
                    PASSWORD_LENGTH.store(0, Ordering::Relaxed);
                    request_frame();
                }),
        );

    let password_progress = Rectangle::new()
        .width(340.0 * ui_scale)
        .height(4.0 * ui_scale)
        .fill(fade(scheme.on_surface, 36))
        .radius(2.0)
        .child(
            Rectangle::new()
                .width(password_width)
                .height(4.0)
                .fill(if failed { scheme.error } else { scheme.primary })
                .radius(2.0),
        );

    let notifications = Notifications::read();
    let app_name = notifications
        .list()
        .last()
        .map(|notification| notification.app_name().to_owned())
        .filter(|name| !name.is_empty())
        .unwrap_or_else(|| String::from("Sem notificações"));
    drop(notifications);

    let notification_chip = Rectangle::new()
        .width(340.0)
        .height(54.0 * ui_scale)
        .fill(fade(scheme.surface_container_highest, 220))
        .radius(18.0 * ui_scale)
        .padding(Padding {
            top: 0.0,
            right: 16.0 * ui_scale,
            bottom: 0.0,
            left: 16.0 * ui_scale,
        })
        .child(
            Row::new(vec![
                Box::new(m3::icon(m3::BELL, 21.0 * ui_scale, scheme.primary)),
                Box::new(scale_label(
                    Type::BodyMedium,
                    app_name,
                    scheme.on_surface,
                    1,
                )),
            ])
            .gap(12.0 * ui_scale)
            .align(Center)
            .width(Parent)
            .height(Parent),
        );

    let center_panel = Rectangle::new()
        .width(440.0 * ui_scale)
        .height(390.0 * ui_scale)
        .fill(fade(scheme.surface_container, 224))
        .radius(34.0 * ui_scale)
        .clip()
        .padding(28.0 * ui_scale)
        .child(
            Column::new(vec![
                Box::new(scale(Type::DisplayLarge, clock.time(true), scheme.on_surface).size(82.0 * ui_scale)),
                Box::new(scale_label(
                    Type::TitleMedium,
                    clock.date(),
                    scheme.on_surface_variant,
                    1,
                )),
                Box::new(notification_chip),
                Box::new(password),
                Box::new(password_progress),
                Box::new(
                    Rectangle::new()
                        .width(340.0)
                        .height(22.0)
                        .align_child(Center, Center)
                        .child(scale_label(
                            Type::LabelMedium,
                            feedback_text,
                            feedback_color,
                            1,
                        )),
                ),
            ])
            .gap(12.0 * ui_scale)
            .width(Parent)
            .height(Parent)
            .align(Center)
            .justify(Center),
        );

    let analog = analogue_clock(&clock, scheme, ui_scale);
    let analog_panel = Rectangle::new()
        .width(172.0 * ui_scale)
        .height(172.0 * ui_scale)
        .fill(fade(scheme.surface_container, 224))
        .radius(86.0 * ui_scale)
        .align_child(Center, Center)
        .child(analog);

    let weather = Weather::read();
    let weather_panel = Rectangle::new()
        .width(270.0 * ui_scale)
        .height(104.0 * ui_scale)
        .fill(fade(scheme.surface_container, 224))
        .radius(26.0 * ui_scale)
        .padding(18.0 * ui_scale)
        .child(
            Row::new(vec![
                Box::new(
                    Rectangle::new()
                        .width(54.0 * ui_scale)
                        .height(54.0 * ui_scale)
                        .fill(scheme.tertiary_container)
                        .radius(18.0)
                        .align_child(Center, Center)
                        .child(m3::icon(weather.glyph(), 30.0 * ui_scale, scheme.on_tertiary_container)),
                ),
                Box::new(
                    Column::new(vec![
                        Box::new(scale_label(
                            Type::TitleSmall,
                            weather.condition(),
                            scheme.on_surface,
                            1,
                        )),
                        Box::new(scale(Type::DisplaySmall, weather.temperature(), scheme.on_surface)),
                    ])
                    .gap(2.0)
                    .width(Parent)
                    .justify(Center),
                ),
            ])
            .gap(14.0 * ui_scale)
            .align(Center)
            .width(Parent)
            .height(Parent),
        );

    let media = Media::read();
    let (track, artist) = crate::art::media_labels(&media);
    let media_active = media.active().is_some();
    let track = if media_active { track } else { String::from("Nada tocando") };
    let artist = if media_active { artist } else { String::from("Abra um player para ouvir música") };
    drop(media);

    let power_button = Rectangle::new()
        .width(42.0 * ui_scale)
        .height(42.0 * ui_scale)
        .fill(scheme.primary_container)
        .radius(21.0 * ui_scale)
        .cursor(Pointer)
        .on_click(|_| {
            let open = !POWER_MENU_OPEN.load(Ordering::Relaxed);
            POWER_MENU_OPEN.store(open, Ordering::Relaxed);
            request_frame();
        })
        .align_child(Center, Center)
        .child(m3::icon(m3::POWER, 21.0 * ui_scale, scheme.on_primary_container));

    let user_media_panel = Rectangle::new()
        .width(330.0 * ui_scale)
        .height(154.0 * ui_scale)
        .fill(fade(scheme.surface_container, 224))
        .radius(26.0)
        .padding(18.0)
        .child(
            Column::new(vec![
                Box::new(
                    Row::new(vec![
                        Box::new(m3::avatar(&initial, 42.0 * ui_scale, scheme)),
                        Box::new(
                            Column::new(vec![
                                Box::new(scale_label(
                                    Type::TitleSmall,
                                    user,
                                    scheme.on_surface,
                                    1,
                                )),
                                Box::new(scale_label(
                                    Type::LabelSmall,
                                    "Sessão atual",
                                    scheme.on_surface_variant,
                                    1,
                                )),
                            ])
                            .gap(2.0)
                            .width(Parent)
                            .justify(Center),
                        ),
                        Box::new(power_button),
                    ])
                    .gap(12.0)
                    .align(Center)
                    .width(Parent),
                ),
                Box::new(
                    Row::new(vec![
                        Box::new(m3::icon(m3::MUSIC, 20.0 * ui_scale, scheme.primary)),
                        Box::new(
                            Column::new(vec![
                                Box::new(scale_label(Type::BodyMedium, track, scheme.on_surface, 1)),
                                Box::new(scale_label(Type::LabelSmall, artist, scheme.on_surface_variant, 1)),
                            ])
                            .gap(2.0)
                            .width(Parent),
                        ),
                    ])
                    .gap(12.0)
                    .align(Center)
                    .width(Parent),
                ),
            ])
            .gap(16.0 * ui_scale)
            .width(Parent)
            .height(Parent)
            .justify(Center),
        );

    let power_menu_open = POWER_MENU_OPEN.load(Ordering::Relaxed);
    let power_menu = if power_menu_open {
        Some(power_menu(scheme, ui_scale))
    } else {
        None
    };

    let mut layers: Vec<Box<dyn amane::Widget>> = vec![
        Box::new(background),
        Box::new(scrim),
        Box::new(
            Rectangle::new()
                .width(Parent)
                .height(Parent)
                .align_child(Center, Center)
                .child(center_panel),
        ),
        Box::new(
            Rectangle::new()
                .width(Parent)
                .height(Parent)
                .padding(Padding {
                    top: 32.0 * ui_scale,
                    right: 40.0 * ui_scale,
                    bottom: 0.0,
                    left: 0.0,
                })
                .align_child(End, amane::Start)
                .child(analog_panel),
        ),
        Box::new(
            Rectangle::new()
                .width(Parent)
                .height(Parent)
                .padding(Padding {
                    top: 0.0,
                    right: 0.0,
                    bottom: 36.0 * ui_scale,
                    left: 40.0 * ui_scale,
                })
                .align_child(amane::Start, End)
                .child(weather_panel),
        ),
        Box::new(
            Rectangle::new()
                .width(Parent)
                .height(Parent)
                .padding(Padding {
                    top: 0.0,
                    right: 40.0,
                    bottom: 36.0,
                    left: 0.0,
                })
                .align_child(End, End)
                .child(user_media_panel),
        ),
    ];

    if let Some(menu) = power_menu {
        layers.push(Box::new(
            Rectangle::new()
                .width(Parent)
                .height(Parent)
                .padding(Padding {
                    top: 0.0,
                    right: 40.0,
                    bottom: 210.0 * ui_scale,
                    left: 0.0,
                })
                .align_child(End, End)
                .child(menu),
        ));
    }

    LayerWindow::new()
        .width(Full)
        .height(monitor.height as f32)
        .anchor_vertical(Vertical::Middle)
        .anchor_horizontal(Horizontal::Middle)
        .layer(Layer::Top)
        .keyboard(Keyboard::Exclusive)
        .namespace("cds-lockscreen")
        .child(
            Rectangle::new()
                .width(Parent)
                .height(Parent)
                .child(Stack::new(layers).width(Parent).height(Parent)),
        )
}

fn analogue_clock(clock: &Clock, scheme: &crate::theme::Scheme, ui_scale: f32) -> Canvas {
    let size = 144.0 * ui_scale;
    let center = size / 2.0;
    let radius = 64.0 * ui_scale;

    let mut shapes: Vec<Box<dyn Shape>> = vec![Box::new(
        Arc::new().stroke(2.0 * ui_scale, fade(scheme.on_surface, 170)),
    )];

    for tick in 0..12 {
        let angle = TAU * tick as f32 / 12.0 - std::f32::consts::FRAC_PI_2;
        let outer_x = center + radius * angle.cos();
        let outer_y = center + radius * angle.sin();
        let inner_radius = if tick % 3 == 0 { radius - 11.0 * ui_scale } else { radius - 6.0 * ui_scale };
        let inner_x = center + inner_radius * angle.cos();
        let inner_y = center + inner_radius * angle.sin();

        shapes.push(Box::new(
            Line::new()
                .from(inner_x, inner_y)
                .to(outer_x, outer_y)
                .stroke(if tick % 3 == 0 { 2.6 * ui_scale } else { 1.4 * ui_scale }, scheme.on_surface_variant)
                .cap(Cap::Round),
        ));
    }

    let (hour, minute) = clock
        .time(true)
        .split_once(':')
        .and_then(|(hour, minute)| Some((hour.parse::<u32>().ok()?, minute.parse::<u32>().ok()?)))
        .unwrap_or((0, 0));

    let hour_angle = TAU * ((hour % 12) as f32 + minute as f32 / 60.0) / 12.0
        - std::f32::consts::FRAC_PI_2;
    let minute_angle = TAU * minute as f32 / 60.0 - std::f32::consts::FRAC_PI_2;

    shapes.push(Box::new(
        Line::new()
            .from(center, center)
            .to(center + 31.0 * ui_scale * hour_angle.cos(), center + 31.0 * hour_angle.sin())
            .stroke(5.0 * ui_scale, scheme.on_surface)
            .cap(Cap::Round),
    ));
    shapes.push(Box::new(
        Line::new()
            .from(center, center)
            .to(center + 49.0 * ui_scale * minute_angle.cos(), center + 49.0 * minute_angle.sin())
            .stroke(2.8 * ui_scale, scheme.primary)
            .cap(Cap::Round),
    ));

    Canvas::new().width(size).height(size).shapes(shapes)
}

fn power_menu(scheme: &crate::theme::Scheme, ui_scale: f32) -> Rectangle {
    let actions = [
        (1, m3::LOGOUT, "Sair da sessão"),
        (2, m3::POWER, "Suspender"),
        (3, m3::RESTART, "Reiniciar"),
        (4, m3::POWER, "Desligar"),
    ];

    let rows: Vec<Box<dyn amane::Widget>> = actions
        .iter()
        .map(|(index, glyph, label)| {
            let index = *index;
            Box::new(
                Rectangle::new()
                    .width(Parent)
                    .height(44.0 * ui_scale)
                    .fill(fade(scheme.surface_container_highest, 210))
                    .radius(14.0)
                    .cursor(Pointer)
                    .on_click(move |_| {
                        POWER_MENU_OPEN.store(false, Ordering::Relaxed);
                        Power::activate(index);
                    })
                    .padding(Padding {
                        top: 0.0,
                        right: 12.0 * ui_scale,
                        bottom: 0.0,
                        left: 12.0 * ui_scale,
                    })
                    .child(
                        Row::new(vec![
                            Box::new(m3::icon(glyph, 19.0 * ui_scale, scheme.on_surface_variant)),
                            Box::new(scale_label(
                                Type::BodyMedium,
                                *label,
                                scheme.on_surface,
                                1,
                            )),
                        ])
                        .gap(12.0)
                        .align(Center)
                        .width(Parent)
                        .height(Parent),
                    ),
            ) as Box<dyn amane::Widget>
        })
        .collect();

    Rectangle::new()
        .width(260.0 * ui_scale)
        .height(224.0 * ui_scale)
        .fill(fade(scheme.surface_container, 238))
        .radius(24.0)
        .clip()
        .padding(14.0 * ui_scale)
        .child(
            Column::new(rows)
                .gap(6.0 * ui_scale)
                .width(Parent)
                .height(Parent)
                .justify(Center),
        )
}
