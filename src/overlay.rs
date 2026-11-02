use amane::{
    Center, Full, Key, Keyboard, Layer, LayerWindow, Parent, Rectangle, Service, Shadow, Widget,
    Zone, Workspaces,
};

use crate::motion;
use crate::theme::{Scheme, fade, mix};
use crate::ui::Ui;

#[derive(Clone, Copy, PartialEq, Eq, Default)]
pub enum Panel {
    #[default]
    None,
    Launcher,
    Clipboard,
    Power,
}

impl Panel {
    fn name(self) -> &'static str {
        match self {
            Panel::None => "none",
            Panel::Launcher => "launcher",
            Panel::Clipboard => "clipboard",
            Panel::Power => "power",
        }
    }
}

// which full screen surface is open over the desks, at most one at a time
#[derive(Default)]
pub struct Overlay {
    panel: Panel,
}

impl Service for Overlay {
    fn new() -> Self {
        Self::default()
    }

    // input and ipc change it, there is nothing to poll
    fn listen() {}
}

impl Overlay {
    pub fn panel(&self) -> Panel {
        self.panel
    }

    pub fn open(panel: Panel) {
        // the dash and an overlay would both want the keyboard
        let dash = Ui::read().dash();

        if dash {
            Ui::close();
        }

        Self::write().panel = panel;
    }

    pub fn close() {
        let open = Self::read().panel != Panel::None;

        if !open {
            return;
        }

        Self::write().panel = Panel::None;
    }

    pub fn toggle(panel: Panel) {
        let showing = Self::read().panel == panel;

        if showing {
            Self::close();

            return;
        }

        Self::open(panel);
    }

    // only the monitor with the focused workspace shows the overlay
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

/*
 * the dimmed backdrop and the panel both fade in, while the panel also
 * settles down into place; the caller keeps the window mapped until the
 * presence reaches zero, so closing is as soft as opening
 */
pub fn layer(
    name: &'static str,
    namespace: &'static str,
    monitor: &str,
    open: bool,
    scheme: &Scheme,
    panel_width: f32,
    panel_height: f32,
    content: impl Widget + 'static,
    on_key: impl Fn(Key) + 'static,
) -> LayerWindow {
    let presence = motion::presence(name, open);
    let focused = Overlay::on_focused_monitor(monitor);

    let visible = focused && (open || presence > 0.001);

    let alpha = (presence * 255.0).round() as u8;

    let panel = Rectangle::new()
        .width(panel_width)
        .height(panel_height)
        .fill(scheme.surface_container_low)
        .radius(30.0)
        .clip()
        .border(1.0, fade(scheme.outline_variant, 90))
        .shadow(
            Shadow::drop(scheme.shadow)
                .opacity(0.45)
                .blur(32.0)
                .offset(0.0, 14.0),
        )
        .opacity(presence)
        .scale(0.94 + 0.06 * presence)
        .translate(0.0, 18.0 * (1.0 - presence))
        .on_click(|_| {})
        .child(content);

    let root = Rectangle::new()
        .width(Parent)
        .height(Parent)
        .fill(fade(scheme.scrim, (alpha as f32 * 0.55) as u8))
        .on_click(|_| Overlay::close())
        .align_child(Center, Center)
        .child(panel);

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
        .child(root)
}

// a search bar shared by the launcher and the clipboard
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

    let fill = mix(
        scheme.surface_container_high,
        scheme.on_surface,
        0.05 * state,
    );

    let text_input = amane::TextInput::new(input)
        .size(16.0)
        .color(scheme.on_surface)
        .placeholder(placeholder)
        .on_change(on_change)
        .on_submit(on_submit);

    let text_input = if open { text_input.focused() } else { text_input };

    Rectangle::new()
        .width(Parent)
        .height(56.0)
        .fill(fill)
        .radius(28.0)
        .cursor(amane::Pointer)
        .on_hover(move |inside| motion::set_hover(owned.clone(), inside))
        .padding(amane::Padding {
            top: 0.0,
            right: 18.0,
            bottom: 0.0,
            left: 18.0,
        })
        .child(
            amane::Row::new(vec![
                Box::new(crate::island::widgets::icon(glyph, 22.0, scheme.on_surface_variant)),
                Box::new(text_input),
            ])
            .gap(12.0)
            .align(Center)
            .width(Parent),
        )
}

// the row highlight used by the launcher and the clipboard
pub fn row_fill(key: &str, selected: bool, scheme: &Scheme) -> (amane::Color, amane::Color) {
    let state = motion::hover(key);

    let target = if selected {
        scheme.secondary_container
    } else {
        amane::Color::TRANSPARENT
    };

    let fill = motion::color(
        &format!("{key}:fill"),
        target,
        motion::SELECT_DURATION,
    );

    let fill = mix(fill, scheme.on_surface, 0.06 * state);

    let color = if selected {
        scheme.on_secondary_container
    } else {
        scheme.on_surface
    };

    (fill, color)
}

pub fn command(arguments: &[String]) -> String {
    let [action, rest @ ..] = arguments else {
        return usage();
    };

    match action.as_str() {
        "launcher" => crate::launcher::Launcher::toggle(),
        "clipboard" => crate::clipboard::Clipboard::toggle(),
        "power" => crate::power::Power::toggle(),

        "toggle" => match rest.first().map(String::as_str) {
            Some("launcher") => crate::launcher::Launcher::toggle(),
            Some("clipboard") => crate::clipboard::Clipboard::toggle(),
            Some("power") => crate::power::Power::toggle(),
            _ => return usage(),
        },

        "close" => Overlay::close(),

        "panel" => return String::from(Overlay::read().panel.name()),

        _ => return usage(),
    }

    String::from("ok")
}

fn usage() -> String {
    String::from("usage: overlay launcher|clipboard|power | toggle <panel> | close | panel")
}
