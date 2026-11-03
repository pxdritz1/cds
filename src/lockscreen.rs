/*
 * the lock screen: a fullscreen lock with the clock, the date, and one
 * input asking for the pam password. enter submits, nothing else moves.
 */
use amane::{
    Center, Column, Full, Keyboard, Layer, LayerWindow, Monitor, Padding, Parent, Rectangle,
    Service, Stack, TextInput, Widget,
};

use crate::m3::{self, boxed, scale, scale_label, Type};
use crate::clock::Clock;
use crate::theme::{Scheme, Theme};

const PASSWORD: &str = "cds-lockscreen-password";

pub fn view(monitor: &Monitor) -> LayerWindow {
    let theme = Theme::read();
    let scheme = theme.scheme();

    // The lock view is instantiated once per output; scale its composition
    // against that output's logical height to avoid clipping on small screens.
    let ui_scale = (monitor.height as f32 / 900.0).clamp(0.82, 1.12);
    let badge_size = 150.0 * ui_scale;
    let field_width = 280.0 * ui_scale;
    let field_height = 48.0 * ui_scale;

    let clock = Clock::read();

    let state = amane::Lock::read();

    let failed = state.failed();

    let checking = state.checking();

    drop(state);

    let user = std::env::var("USER").unwrap_or_else(|_| String::from("você"));

    // the big cookie badge with a lock glyph, the m3 expressive touch
    let badge = Rectangle::new()
        .width(badge_size)
        .height(badge_size)
        .align_child(Center, Center)
        .child(
            Stack::new(vec![
                Box::new(m3::cookie(7.0, badge_size, scheme.primary_container)) as Box<dyn Widget>,
                Box::new(m3::icon(
                    m3::LOCK,
                    64.0 * ui_scale,
                    scheme.on_primary_container,
                )) as Box<dyn Widget>,
            ]),
        );

    let field = Rectangle::new()
        .width(field_width)
        .height(field_height)
        .fill(scheme.surface_container_highest)
        .radius(16.0 * ui_scale)
        .padding(Padding {
            top: 0.0,
            right: 12.0 * ui_scale,
            bottom: 0.0,
            left: 12.0 * ui_scale,
        })
        .align_child(Center, Center)
        .child(
            TextInput::new(PASSWORD)
                .size(17.0 * ui_scale)
                .color(scheme.on_surface)
                .placeholder("digite a senha…")
                .password()
                .on_submit(move |password| {
                    amane::Lock::unlock(&password);

                    TextInput::set_text(PASSWORD, "");
                })
                .focused(),
        );

    let feedback = if failed {
        scale_label(Type::LabelMedium, "senha incorreta", scheme.error, 1)
            .size(12.0 * ui_scale)
    } else if checking {
        scale_label(Type::LabelMedium, "verificando…", scheme.on_surface_variant, 1)
            .size(12.0 * ui_scale)
    } else {
        scale_label(
            Type::LabelMedium,
            "enter pra desbloquear",
            scheme.on_surface_variant,
            1,
        )
        .size(12.0 * ui_scale)
    };

    let body = Column::new(vec![
        Box::new(badge),
        Box::new(
            scale(Type::DisplayMedium, clock.time(true), scheme.on_surface)
                .size(45.0 * ui_scale),
        ),
        Box::new(scale(
            Type::TitleSmall,
            format!("{} · {}", clock.date(), user),
            scheme.on_surface_variant,
        )
        .size(14.0 * ui_scale)),
        Box::new(field),
        Box::new(feedback),
    ])
    .gap(12.0 * ui_scale)
    .width(Parent)
    .align(Center);

    LayerWindow::new()
        .width(Full)
        .height(Full)
        .layer(Layer::Top)
        .keyboard(Keyboard::Exclusive)
        .namespace("cds-lockscreen")
        .child(
            Rectangle::new()
                .width(Parent)
                .height(Parent)
                .fill(scheme.surface)
                .align_child(Center, Center)
                .child(body),
        )
}
