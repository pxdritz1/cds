use amane::{Rectangle, Size};
use crate::theme::Scheme;
pub fn cover(size: f32, radius: f32, scheme: &Scheme) -> Rectangle {
    Rectangle::new()
        .width(Size::Fixed(size))
        .height(Size::Fixed(size))
        .fill(scheme.surface_container_highest)
        .radius(radius)
}