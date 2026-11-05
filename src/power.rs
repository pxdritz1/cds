use amane::{
    Button, Center, Column, Key, LayerWindow, Monitor, Padding, Parent, Pointer, Rectangle, Row,
    Service, SpaceBetween, Widget,
};

use crate::island::widgets::{CHECK, LOCK, LOGOUT, POWER, RESTART, boxed, icon, label, text};
use crate::m3;
use crate::overlay::{self, Overlay, Panel, row_fill};
use crate::theme::{Scheme, Theme, fade};

const WIDTH: f32 = 360.0;
const HEIGHT: f32 = 420.0;
const PADDING: f32 = 16.0;

#[derive(Clone, Copy, PartialEq, Eq)]
enum Action {
    Lock,
    Logout,
    Suspend,
    Reboot,
    Shutdown,
}

const ACTIONS: [Action; 5] = [
    Action::Lock,
    Action::Logout,
    Action::Suspend,
    Action::Reboot,
    Action::Shutdown,
];

impl Action {
    fn glyph(self) -> &'static str {
        match self {
            Action::Lock => LOCK,
            Action::Logout => LOGOUT,
            Action::Suspend => POWER,
            Action::Reboot => RESTART,
            Action::Shutdown => POWER,
        }
    }

    fn label(self) -> &'static str {
        match self {
            Action::Lock => crate::lang::text("Bloquear tela", "Lock screen"),
            Action::Logout => crate::lang::text("Sair da sessão", "Log out"),
            Action::Suspend => crate::lang::text("Suspender", "Suspend"),
            Action::Reboot => crate::lang::text("Reiniciar", "Restart"),
            Action::Shutdown => crate::lang::text("Desligar", "Shut down"),
        }
    }

    fn hint(self) -> &'static str {
        match self {
            Action::Lock => crate::lang::text("Manter a sessão ativa", "Keep the session running"),
            Action::Logout => crate::lang::text("Encerrar a sessão do niri", "Close the niri session"),
            Action::Suspend => crate::lang::text("Suspender até ser reativado", "Sleep until woken"),
            Action::Reboot => crate::lang::text("Reiniciar o computador", "Restart the machine"),
            Action::Shutdown => crate::lang::text("Desligar o computador", "Power the machine off"),
        }
    }

    // the destructive actions carry the error role, as material suggests
    fn danger(self) -> bool {
        matches!(self, Action::Shutdown | Action::Reboot)
    }

    fn run(self) {
        let command = match self {
            Action::Lock => "loginctl lock-session",
            Action::Logout => "niri msg action quit",
            Action::Suspend => "systemctl suspend",
            Action::Reboot => "systemctl reboot",
            Action::Shutdown => "systemctl poweroff",
        };

        amane::spawn(command);
    }
}

#[derive(Default)]
pub struct Power {
    selected: usize,
}

impl Service for Power {
    fn new() -> Self {
        Self::default()
    }

    // only input and ipc change the selection
    fn listen() {}
}

impl Power {
    pub fn selected(&self) -> usize {
        self.selected
    }

    pub fn move_by(delta: isize) {
        let mut power = Self::write();

        let last = ACTIONS.len() as isize - 1;
        let current = power.selected.min(last as usize) as isize;

        power.selected = (current + delta).clamp(0, last) as usize;
    }

    pub fn toggle() {
        let showing = Overlay::read().panel() == Panel::Power;

        if showing {
            Overlay::close();

            return;
        }

        Self::open();
    }

    pub fn open() {
        let mut power = Self::write();

        power.selected = 0;

        drop(power);

        Overlay::open(Panel::Power);
    }

    pub fn activate(index: usize) {
        let Some(action) = ACTIONS.get(index).copied() else {
            return;
        };

        Overlay::close();
        action.run();
    }

    pub fn activate_selected() {
        let selected = Self::read().selected;

        Self::activate(selected);
    }
}

pub fn view(monitor: &Monitor) -> LayerWindow {
    let theme = Theme::read();
    let scheme = theme.scheme();

    let state = Overlay::read();
    let open = state.panel() == Panel::Power;

    let power = Power::read();
    let selected = power.selected().min(ACTIONS.len() - 1);

    let header = Row::new(vec![
        boxed(
            Rectangle::new()
                .width(32.0)
                .height(32.0)
                .fill(scheme.primary_container)
                .radius(10.0)
                .align_child(Center, Center)
                .child(icon(POWER, 18.0, scheme.on_primary_container)),
        ),
        boxed(text(
            crate::lang::text("Energia", "Power"),
            18.0,
            scheme.on_surface,
            amane::Weight::SemiBold,
        )),
    ])
    .gap(10.0)
    .align(Center)
    .width(Parent);

    let rows: Vec<Box<dyn Widget>> = ACTIONS
        .iter()
        .enumerate()
        .map(|(index, &action)| boxed(action_row(scheme, action, index, index == selected)))
        .collect();

    let footer = Row::new(vec![
        boxed(text(
            crate::lang::text("↑↓ selecionar  ·  Enter executar", "↑↓ choose  ·  Enter runs"),
            11.5,
            scheme.on_surface_variant,
            amane::Weight::Medium,
        )),
        boxed(text(
            crate::lang::text("Esc fecha", "Esc closes"),
            11.5,
            fade(scheme.on_surface_variant, 170),
            amane::Weight::Regular,
        )),
    ])
    .width(Parent)
    .justify(SpaceBetween);

    let content = Rectangle::new()
        .width(Parent)
        .height(Parent)
        .padding(Padding {
            top: PADDING,
            right: PADDING,
            bottom: PADDING,
            left: PADDING,
        })
        .child(
            Column::new(vec![
                boxed(header),
                boxed(Column::new(rows).gap(4.0).width(Parent).height(Parent)),
                boxed(footer),
            ])
            .width(Parent)
            .height(Parent)
            .gap(14.0),
        );

    overlay::layer(
        "power",
        "cds-power",
        &monitor.name,
        open,
        scheme,
        overlay::Frame {
            width: WIDTH,
            height: HEIGHT,
            radius: m3::shape::XL,
            place: overlay::Place::Under,
            full_screen: false,
        },
        content,
        |key| match key {
            Key::Down | Key::Tab => Power::move_by(1),
            Key::Up => Power::move_by(-1),
            Key::Enter => Power::activate_selected(),
            _ => {}
        },
    )
}

fn action_row(scheme: &Scheme, action: Action, index: usize, selected: bool) -> Rectangle {
    let key = format!("power:{index}");
    let (fill, color) = row_fill(&key, selected, scheme);

    let (container, glyph_color) = if action.danger() {
        (scheme.error_container, scheme.on_error_container)
    } else {
        (scheme.primary_container, scheme.on_primary_container)
    };

    let badge = Rectangle::new()
        .width(36.0)
        .height(36.0)
        .fill(container)
        .radius(10.0)
        .align_child(Center, Center)
        .child(icon(action.glyph(), 20.0, glyph_color));

    let info = Column::new(vec![
        boxed(label(action.label(), 13.5, color, amane::Weight::Medium, 1)),
        boxed(label(
            action.hint(),
            11.0,
            if selected {
                fade(color, 200)
            } else {
                scheme.on_surface_variant
            },
            amane::Weight::Regular,
            1,
        )),
    ])
    .gap(1.0)
    .width(Parent);

    let mut children: Vec<Box<dyn Widget>> = vec![boxed(badge), boxed(info)];

    if selected {
        children.push(boxed(icon(CHECK, 18.0, color)));
    }

    let owned = key.clone();

    Rectangle::new()
        .width(Parent)
        .height(54.0)
        .fill(fill)
        .radius(16.0)
        .padding(8.0)
        .cursor(Pointer)
        .on_hover(move |inside| crate::motion::set_hover(owned.clone(), inside))
        .on_click(move |button| {
            if button == Button::Left {
                Power::activate(index);
            }
        })
        .child(
            Row::new(children)
                .gap(12.0)
                .align(Center)
                .width(Parent)
                .height(Parent),
        )
}
