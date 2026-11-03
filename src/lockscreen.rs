/*
 * the lock screen: one centered composition on the whole output. the
 * clock and date lead, the avatar follows, then the password field and
 * its feedback. every size derives from the output so the block stays
 * centered and never clips on short screens.
 */
use amane::{
    Center, Column, Full, Horizontal, Keyboard, Layer, LayerWindow, Monitor, Padding, Parent,
    Rectangle, Service, TextInput, Vertical,
};

use crate::clock::Clock;
use crate::m3::{self, Type, scale, scale_label};
use crate::theme::{Theme, fade};

const PASSWORD: &str = "cds-lockscreen-password";

pub fn view(monitor: &Monitor) -> LayerWindow {
    let theme = Theme::read();
    let scheme = theme.scheme();

    // scale the composition against the shortest edge, so 16:9 and 16:10
    // outputs land on the same optical size
    let shortest = monitor.width.min(monitor.height) as f32;
    let ui_scale = (shortest / 900.0).clamp(0.76, 1.12);

    let content_width = (monitor.width as f32 * 0.42).clamp(280.0 * ui_scale, 380.0 * ui_scale);

    let clock = Clock::read();
    let state = amane::Lock::read();

    let failed = state.failed();
    let checking = state.checking();

    drop(state);

    let user = std::env::var("USER").unwrap_or_else(|_| String::from("você"));
    let initial = user
        .chars()
        .next()
        .map(|letter| letter.to_uppercase().to_string())
        .unwrap_or_else(|| String::from("C"));

    let avatar_size = 88.0 * ui_scale;
    let field_height = 52.0 * ui_scale;
    let field_radius = field_height / 2.0;

    // the border carries the authentication state, like a m3 text field
    let (field_border, feedback_color, feedback_text) = if failed {
        (scheme.error, scheme.error, "senha incorreta")
    } else if checking {
        (scheme.primary, scheme.on_surface_variant, "verificando…")
    } else {
        (
            fade(scheme.outline_variant, 180),
            scheme.on_surface_variant,
            "enter pra desbloquear",
        )
    };

    let greeting = scale(
        Type::LabelLarge,
        clock.greeting(),
        scheme.on_surface_variant,
    )
    .size(13.0 * ui_scale);

    let time = scale(Type::DisplayLarge, clock.time(true), scheme.on_surface).size(58.0 * ui_scale);

    let date = scale(
        Type::TitleSmall,
        format!("{} · {}", clock.date(), user),
        scheme.on_surface_variant,
    )
    .size(14.0 * ui_scale);

    let field = Rectangle::new()
        .width(content_width)
        .height(field_height)
        .fill(scheme.surface_container_highest)
        .radius(field_radius)
        .border(1.5, field_border)
        .padding(Padding {
            top: 0.0,
            right: 20.0 * ui_scale,
            bottom: 0.0,
            left: 20.0 * ui_scale,
        })
        .align_child(Center, Center)
        .child(
            TextInput::new(PASSWORD)
                .size(16.0 * ui_scale)
                .color(scheme.on_surface)
                .placeholder("digite a senha…")
                .password()
                .on_submit(move |password| {
                    amane::Lock::unlock(&password);

                    TextInput::set_text(PASSWORD, "");
                })
                .focused(),
        );

    let feedback = Rectangle::new()
        .width(content_width)
        .height(field_height)
        .align_child(Center, Center)
        .child(
            scale_label(Type::LabelMedium, feedback_text, feedback_color, 1).size(12.0 * ui_scale),
        );

    let body = Column::new(vec![
        Box::new(greeting),
        Box::new(time),
        Box::new(date),
        Box::new(
            Rectangle::new()
                .width(content_width)
                .height(10.0 * ui_scale),
        ),
        Box::new(m3::avatar(&initial, avatar_size, scheme)),
        Box::new(Rectangle::new().width(content_width).height(6.0 * ui_scale)),
        Box::new(field),
        Box::new(feedback),
    ])
    .gap(8.0 * ui_scale)
    .width(Parent)
    .align(Center);

    LayerWindow::new()
        .width(Full)
        .height(Full)
        .anchor_vertical(Vertical::Middle)
        .anchor_horizontal(Horizontal::Middle)
        .layer(Layer::Top)
        .keyboard(Keyboard::Exclusive)
        .namespace("cds-lockscreen")
        .child(
            Rectangle::new()
                .width(Parent)
                .height(Parent)
                .fill(scheme.surface)
                .padding(Padding {
                    top: 24.0 * ui_scale,
                    right: 24.0 * ui_scale,
                    bottom: 24.0 * ui_scale,
                    left: 24.0 * ui_scale,
                })
                .align_child(Center, Center)
                .child(
                    Column::new(vec![Box::new(body)])
                        .width(Parent)
                        .align(Center)
                        .justify(Center),
                ),
        )
}
