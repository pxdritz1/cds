use amane::{Button, Color, Full, Layer, LayerWindow, Parent, Rectangle, Service, Vertical};

use super::state::ShellState;

pub fn view() -> LayerWindow {
    let visible = ShellState::read().overlay.is_some();

    LayerWindow::new()
        .width(Full)
        .height(Full)
        .anchor_vertical(Vertical::Middle)
        .layer(Layer::Top)
        .namespace("cds-dismiss")
        .visible(visible)
        .child(
            Rectangle::new()
                .width(Parent)
                .height(Parent)
                .fill(Color::rgba(0, 0, 0, 96))
                .on_click(dismiss_overlay),
        )
}

fn dismiss_overlay(_: Button) {
    ShellState::write().overlay = None;
}
