use amane::{Center, Image, Rectangle};

use crate::island::widgets::{MUSIC, icon};
use crate::theme::Scheme;

// an album cover, or the music note placeholder when there is no image
pub fn artwork(path: &str, size: f32, radius: f32, scheme: &Scheme) -> Rectangle {
    let file = path.strip_prefix("file://").unwrap_or(path);

    let base = Rectangle::new()
        .width(size)
        .height(size)
        .radius(radius)
        .clip();

    if !file.is_empty() && Image::loaded(file) {
        let pixels = (size * 2.0).round() as u32;

        base.fill(Image::cover(file).thumbnail(pixels, pixels))
    } else {
        base.fill(scheme.secondary_container)
            .align_child(Center, Center)
            .child(icon(MUSIC, size * 0.42, scheme.on_secondary_container))
    }
}

// metadata can be empty even while a player is open, so labels get a fallback
pub fn media_labels(media: &amane::Media) -> (String, String) {
    let active = media.active().is_some();

    let title = media.title();
    let artist = media.artist();

    let title = if !title.is_empty() {
        String::from(title)
    } else if active {
        String::from("Título desconhecido")
    } else {
        String::from("Nada tocando")
    };

    let artist = if !artist.is_empty() {
        String::from(artist)
    } else if active {
        String::from("Artista desconhecido")
    } else {
        String::from("Abra um player pra começar")
    };

    (title, artist)
}
