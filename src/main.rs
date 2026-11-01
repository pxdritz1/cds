mod theme;

use amane::{App, Layer, LayerWindow, Parent, Rectangle, Text, Vertical};

fn main() {
    App::new().window(view).run();
}

fn view() -> LayerWindow {
    LayerWindow::new()
        .width(460.0)
        .height(64.0)
        .anchor_vertical(Vertical::Top)
        .layer(Layer::Top)
        .child(
            Rectangle::new()
                .width(Parent)
                .height(Parent)
                .fill(theme::SURFACE)
                .child(
                    Text::new("chloe dynamic shell")
                        .size(16.0)
                        .color(theme::ON_SURFACE),
                ),
        )
}
