use amane::{Full, Image, Layer, LayerWindow, Monitor, Parent, Rectangle, Service, Zone};

use crate::settings::Settings;
use crate::theme::Theme;

pub fn view(monitor: &Monitor) -> LayerWindow {
    let settings = Settings::read();
    let theme = Theme::read();

    let path = settings.wallpaper();

    let base = Rectangle::new().width(Parent).height(Parent);

    let filled = if path.is_empty() {
        base.fill(theme.scheme().surface)
    } else {
        base.fill(Image::cover(path).thumbnail(monitor.width, monitor.height))
    };

    LayerWindow::new()
        .width(Full)
        .height(monitor.height as f32)
        .layer(Layer::Background)
        .space(Zone::Ignore)
        .namespace("cds-wallpaper")
        .click_through()
        .child(filled)
}
