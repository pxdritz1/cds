use amane::{
    Button, Center, End, Full, Key, Keyboard, Layer, LayerWindow, Padding, Parent, Rectangle,
    Service, Shadow, Stack, Start, Widget, Workspaces, Zone,
};

use crate::m3::{self, boxed, icon, state_layer};
use crate::motion;
use crate::theme::{Scheme, fade, mix};
use crate::ui::Ui;

#[derive(Clone, Copy, PartialEq, Eq, Default)]
pub enum Panel {
    #[default]
    None,
    Launcher,
    Spotlight,
    Apps,
    Clipboard,
    Power,
    Control,
    Config,
    Color,
    Emoji,
}

impl Panel {
    pub fn name(self) -> &'static str {
        match self {
            Panel::None => "none",
            Panel::Launcher => "launcher",
            Panel::Spotlight => "spotlight",
            Panel::Apps => "apps",
            Panel::Clipboard => "clipboard",
            Panel::Power => "power",
            Panel::Control => "control",
            Panel::Config => "config",
            Panel::Color => "color",
            Panel::Emoji => "emoji",
        }
    }
}

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum Place {
    Under,
    Notch,
    Spot,
    Right,
    Middle,
}

#[derive(Clone, Copy)]
pub struct Frame {
    pub width: f32,
    pub height: f32,
    pub radius: f32,
    pub place: Place,
    pub full_screen: bool,
}

#[derive(Default)]
pub struct Overlay {
    panel: Panel,
}

impl Service for Overlay {
    fn new() -> Self {
        Self::default()
    }

    fn listen() {}
}

impl Overlay {
    pub fn panel(&self) -> Panel {
        self.panel
    }

    pub fn open(panel: Panel) {
        Ui::close();
        crate::tray::Tray::close();

        if Self::read().panel == Panel::Launcher && panel != Panel::Launcher {
            crate::launcher::Launcher::reset();
        }

        Self::write().panel = panel;
    }

    pub fn close() {
        let panel = Self::read().panel;

        if panel == Panel::None {
            return;
        }

        if panel == Panel::Launcher {
            crate::launcher::Launcher::reset();
        }

        Self::write().panel = Panel::None;
    }

    /// Close the active overlay without playing its exit animation.
    pub fn close_immediately() {
        let panel = Self::read().panel;

        if panel == Panel::None {
            return;
        }

        if panel == Panel::Launcher {
            crate::launcher::Launcher::reset();
        }

        Self::write().panel = Panel::None;
        motion::presence_immediately(panel.name(), false);
        Ui::refresh();
        amane::request_frame();
    }

    pub fn toggle(panel: Panel) {
        if Self::read().panel == panel {
            Self::close();

            return;
        }

        Self::open(panel);
    }

    pub fn on_focused_monitor(monitor: &str) -> bool {
        let workspaces = Workspaces::read();

        workspaces
            .list()
            .iter()
            .find(|workspace| workspace.focused())
            .and_then(|workspace| workspace.output())
            .is_none_or(|output| output == monitor)
    }
}

fn bubble(progress: f32, size: f32, dx: f32, dy: f32, color: amane::Color) -> Rectangle {
    let scale = 0.28 + progress * 1.55;
    let diameter = size * scale;

    Rectangle::new()
        .width(Parent)
        .height(Parent)
        .align_child(Center, Center)
        .child(
            Rectangle::new()
                .width(diameter)
                .height(diameter)
                .fill(color)
                .radius(Full)
                .opacity((1.0 - progress) * 0.16)
                .translate(dx * progress, dy * progress),
        )
}

pub fn layer(
    name: &'static str,
    namespace: &'static str,
    monitor: &str,
    open: bool,
    scheme: &Scheme,
    frame: Frame,
    content: impl Widget + 'static,
    on_key: impl Fn(Key) + 'static,
) -> LayerWindow {
    Ui::watch();

    let presence = motion::presence(name, open);
    let burst = motion::fade(&format!("{name}:bubbles"), if open { 1 } else { 0 });
    let focused = Overlay::on_focused_monitor(monitor);
    let visible = focused && (open || presence > 0.001);

    if visible && (presence > 0.001 || burst < 0.999) {
        motion::request_frame_at_most(30);
    }

    let dim = match frame.place {
        Place::Right => 0.16,
        Place::Under => 0.40,
        Place::Notch => 0.0,
        Place::Spot => 0.45,
        Place::Middle => 0.55,
    };

    let panel = if frame.full_screen {
        Rectangle::new()
            .width(Parent)
            .height(Parent)
            .fill(scheme.surface_container)
            .radius(0.0)
            .clip()
            .border(1.0, fade(scheme.outline_variant, 70))
            .shadow(
                Shadow::drop(scheme.shadow)
                    .opacity(0.40)
                    .blur(36.0)
                    .offset(0.0, 14.0),
            )
            .opacity(presence)
            .scale(0.96 + 0.04 * presence)
            .translate(0.0, -14.0 * (1.0 - presence))
            .on_click(|_| {})
            .child(content)
    } else {
        Rectangle::new()
            .width(frame.width)
            .height(frame.height)
            .fill(scheme.surface_container)
            .radius(frame.radius)
            .clip()
            .border(1.0, fade(scheme.outline_variant, 70))
            .shadow(
                Shadow::drop(scheme.shadow)
                    .opacity(0.40)
                    .blur(36.0)
                    .offset(0.0, 14.0),
            )
            .opacity(presence)
            .scale(0.96 + 0.04 * presence)
            .translate(0.0, -14.0 * (1.0 - presence))
            .on_click(|_| {})
            .child(content)
    };

    let padding = match frame.place {
        Place::Under => Padding {
            top: 58.0,
            ..Padding::default()
        },
        Place::Notch => Padding {
            top: 44.0,
            ..Padding::default()
        },
        Place::Spot => Padding {
            top: 130.0,
            ..Padding::default()
        },
        Place::Right => Padding {
            top: 58.0,
            right: 10.0,
            ..Padding::default()
        },
        Place::Middle => Padding::default(),
    };

    // The scrim handles outside clicks; a separate layer keeps the
    // bubble burst behind the panel without intercepting its input.
    let root = Rectangle::new()
        .width(Parent)
        .height(Parent)
        .fill(fade(scheme.scrim, (presence * 255.0 * dim) as u8))
        .on_click(|button| {
            if button == Button::Left {
                Overlay::close();
            }
        });

    let panel_layer = Rectangle::new()
        .width(Parent)
        .height(Parent)
        .padding(padding);

    let panel_layer = match frame.place {
        Place::Under | Place::Notch => panel_layer.align_child(Center, Start),
        Place::Spot => panel_layer.align_child(Center, Start),
        Place::Right => panel_layer.align_child(End, Start),
        Place::Middle => panel_layer.align_child(Center, Center),
    }
    .child(panel);

    let layers: Vec<Box<dyn Widget>> = vec![
        boxed(bubble(burst, 22.0, -170.0, -90.0, scheme.primary_container)),
        boxed(bubble(burst, 16.0, 170.0, -60.0, scheme.tertiary_container)),
        boxed(bubble(burst, 12.0, -95.0, 150.0, scheme.secondary_container)),
        boxed(bubble(burst, 18.0, 110.0, 135.0, scheme.primary)),
        boxed(panel_layer),
    ];

    let content = Stack::new(layers).width(Parent).height(Parent);

    LayerWindow::new()
        .width(Full)
        .height(Full)
        .layer(Layer::Overlay)
        .space(Zone::Ignore)
        .keyboard(if open {
            Keyboard::Exclusive
        } else {
            Keyboard::None
        })
        .visible(visible)
        .namespace(namespace)
        .on_key(move |key| {
            if key == Key::Escape {
                Overlay::close();

                return;
            }

            on_key(key);
        })
        .child(root.child(content))
}

pub fn search(
    key: &str,
    input: &'static str,
    glyph: &str,
    placeholder: &str,
    open: bool,
    scheme: &Scheme,
    on_change: impl Fn(String) + 'static,
    on_submit: impl Fn(String) + 'static,
) -> Rectangle {
    let state = motion::hover(&format!("{key}:search"));
    let owned = String::from(key);

    let fill = state_layer(
        scheme.surface_container_high,
        scheme.on_surface,
        0.05 * state,
    );

    let field = amane::TextInput::new(input)
        .size(16.0)
        .color(scheme.on_surface)
        .placeholder(placeholder)
        .on_change(on_change)
        .on_submit(on_submit);

    let field = if open { field.focused() } else { field };

    Rectangle::new()
        .width(Parent)
        .height(56.0)
        .fill(fill)
        .radius(Full)
        .cursor(amane::Pointer)
        .on_hover(move |inside| motion::set_hover(format!("{owned}:search"), inside))
        .padding(Padding {
            top: 0.0,
            right: 20.0,
            bottom: 0.0,
            left: 20.0,
        })
        .child(
            amane::Row::new(vec![
                boxed(icon(glyph, 22.0, scheme.on_surface_variant)),
                boxed(field),
            ])
            .gap(14.0)
            .align(Center)
            .width(Parent),
        )
}

pub fn row_fill(key: &str, selected: bool, scheme: &Scheme) -> (amane::Color, amane::Color) {
    let state = motion::hover(key);

    let target = if selected {
        scheme.secondary_container
    } else {
        amane::Color::TRANSPARENT
    };

    let fill = motion::color(&format!("{key}:fill"), target, motion::SELECT_DURATION);
    let fill = state_layer(fill, scheme.on_surface, 0.06 * state);

    let color = if selected {
        scheme.on_secondary_container
    } else {
        scheme.on_surface
    };

    (fill, color)
}

pub fn hint_bar(left: &str, right: &str, scheme: &Scheme) -> amane::Row {
    amane::Row::new(vec![
        boxed(m3::scale(
            m3::Type::LabelMedium,
            left,
            scheme.on_surface_variant,
        )),
        boxed(m3::scale(
            m3::Type::LabelMedium,
            right,
            fade(scheme.on_surface_variant, 170),
        )),
    ])
    .width(Parent)
    .justify(amane::SpaceBetween)
}

pub fn dimmed(scheme: &Scheme) -> amane::Color {
    mix(scheme.on_surface_variant, scheme.surface, 0.2)
}
