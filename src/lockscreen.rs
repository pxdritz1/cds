/*
 * the lock screen: a fullscreen lock with the clock, the date, and one
 * input asking for the pam password. enter submits, nothing else moves.
 */
use amane::{
    Center, Column, Full, Keyboard, Layer, LayerWindow, Monitor, Padding, Parent, Rectangle, Row,
    Service, Stack, TextInput, Widget,
};

use crate::m3::{self, boxed, scale, scale_label, Type};
use crate::clock::Clock;
use crate::theme::{Scheme, Theme};

const PASSWORD: &str = "cds-lockscreen-password";

pub fn view(monitor: &Monitor) -> LayerWindow {
    let theme = Theme::read();
    let scheme = theme.scheme();

    let clock = Clock::read();

    let state = amane::Lock::read();

    let failed = state.failed();

    let checking = state.checking();

    drop(state);

    let user = std::env::var("USER").unwrap_or_else(|_| String::from("você"));

    // the big cookie badge with a lock glyph, the m3 expressive touch
    let badge = Rectangle::new()
        .width(150.0)
        .height(150.0)
        .align_child(Center, Center)
        .child(
            Stack::new(vec![
                Box::new(m3::cookie(7.0, 150.0, scheme.primary_container)) as Box<dyn Widget>,
                Box::new(m3::icon(m3::LOCK, 64.0, scheme.on_primary_container)) as Box<dyn Widget>,
            ]),
        );

    let field = Rectangle::new()
        .width(280.0)
        .height(48.0)
        .fill(scheme.surface_container_highest)
        .radius(16.0)
        .padding(Padding {
            top: 0.0,
            right: 12.0,
            bottom: 0.0,
            left: 12.0,
        })
        .align_child(Center, Center)
        .child(
            TextInput::new(PASSWORD)
                .size(17.0)
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
    } else if checking {
        scale_label(Type::LabelMedium, "verificando…", scheme.on_surface_variant, 1)
    } else {
        scale_label(
            Type::LabelMedium,
            "enter pra desbloquear",
            scheme.on_surface_variant,
            1,
        )
    };

    let body = Column::new(vec![
        Box::new(badge),
        Box::new(scale(Type::DisplayMedium, clock.time(true), scheme.on_surface)),
        Box::new(scale(
            Type::TitleSmall,
            format!("{} · {}", clock.date(), user),
            scheme.on_surface_variant,
        )),
        Box::new(field),
        Box::new(feedback),
    ])
    .gap(12.0)
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
