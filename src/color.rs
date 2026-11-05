use std::process::Command;
use std::thread;
use std::time::Duration;

use amane::{
    Center, Column, Key, LayerWindow, Monitor, Padding, Parent, Rectangle, Row, Service,
    TextInput, Widget,
};

use crate::m3::{self, Type, boxed};
use crate::overlay::{self, Overlay, Panel};
use crate::theme::Theme;

const INPUT: &str = "cds-color-hex";
const WIDTH: f32 = 520.0;
const HEIGHT: f32 = 300.0;

#[derive(Default)]
pub struct ColorPicker {
    hex: String,
}

impl Service for ColorPicker {
    fn new() -> Self { Self { hex: String::from("#6750A4") } }
    fn listen() {}
}

impl ColorPicker {
    pub fn hex(&self) -> &str { &self.hex }

    pub fn set_hex(value: String) {
        Self::write().hex = value;
    }

    pub fn reset() {
        let value = Self::read().hex.clone();
        TextInput::set_text(INPUT, &value);
    }

    pub fn open() {
        Self::reset();
        Overlay::open(Panel::Color);
    }

    /// Hide the color overlay before starting a screen capture.
    pub fn pick_from_screen() {
        Overlay::close_immediately();

        thread::spawn(move || {
            // Allow the compositor to commit the frame with the overlay hidden.
            thread::sleep(Duration::from_millis(50));

            let output = Command::new("hyprpicker")
                .args(["--format", "hex", "--no-fancy"])
                .output();

            let picked = output
                .ok()
                .filter(|output| output.status.success())
                .and_then(|output| {
                    let value = String::from_utf8_lossy(&output.stdout);
                    normalize(value.trim())
                });

            if let Some(hex) = picked {
                ColorPicker::set_hex(hex.clone());
                TextInput::set_text(INPUT, &hex);
            }

            // Reopen even when the picker is cancelled or unavailable, keeping the
            // previous HEX value visible instead of leaving the user without a panel.
            Overlay::open(Panel::Color);
            crate::ui::Ui::refresh();
        });
    }

    pub fn copy_hex() {
        let value = Self::read().hex.clone();
        let Some(hex) = normalize(&value) else { return; };
        thread::spawn(move || {
            if let Ok(mut child) = Command::new("wl-copy").arg("--").arg(hex).spawn() {
                let _ = child.wait();
            }
        });
        Overlay::close();
    }
}

fn normalize(value: &str) -> Option<String> {
    let value = value.trim().trim_start_matches('#');
    if value.len() != 6 || !value.bytes().all(|byte| byte.is_ascii_hexdigit()) {
        return None;
    }
    Some(format!("#{value}").to_uppercase())
}

fn color(value: &str, fallback: amane::Color) -> amane::Color {
    let Some(hex) = normalize(value) else { return fallback; };
    let raw = &hex[1..];
    let Ok(red) = u8::from_str_radix(&raw[0..2], 16) else { return fallback; };
    let Ok(green) = u8::from_str_radix(&raw[2..4], 16) else { return fallback; };
    let Ok(blue) = u8::from_str_radix(&raw[4..6], 16) else { return fallback; };
    amane::Color::rgba(red, green, blue, 255)
}

pub fn view(monitor: &Monitor) -> LayerWindow {
    let theme = Theme::read();
    let scheme = theme.scheme();
    let open = Overlay::read().panel() == Panel::Color;
    let value = ColorPicker::read().hex().to_string();
    let normalized = normalize(&value);
    let preview = normalized.as_deref().map(|hex| color(hex, scheme.primary)).unwrap_or(scheme.error);

    let field = TextInput::new(INPUT)
        .size(18.0)
        .color(scheme.on_surface)
        .placeholder("#RRGGBB")
        .on_change(ColorPicker::set_hex)
        .on_submit(|_| ColorPicker::copy_hex());
    let field = if open { field.focused() } else { field };

    let preview_box = Rectangle::new().width(Parent).height(86.0)
        .fill(preview).radius(20.0)
        .child(Rectangle::new().width(Parent).height(Parent)
            .padding(16.0)
            .align_child(amane::End, Center)
            .child(m3::text(normalized.clone().unwrap_or_else(|| String::from(crate::lang::text("HEX inválido", "Invalid HEX"))), 13.0,
                if normalized.is_some() { amane::Color::WHITE } else { scheme.on_error }, amane::Weight::SemiBold)));

    let input_box = Rectangle::new().width(Parent).height(54.0)
        .fill(scheme.surface_container_highest).radius(14.0)
        .padding(Padding { top: 0.0, right: 14.0, bottom: 0.0, left: 14.0 })
        .align_child(Center, Center).child(field);

    let copy = m3::button("color-copy", crate::lang::text("Copiar HEX", "Copy HEX"),
        Some(crate::island::widgets::COPY), m3::Kind::Filled, &scheme, ColorPicker::copy_hex);
    let eyedropper = m3::button("color-eyedropper",
        crate::lang::text("Conta-gotas", "Eyedropper"),
        None, m3::Kind::Tonal, &scheme, ColorPicker::pick_from_screen);
    let hint = if normalized.is_some() {
        crate::lang::text("Digite um valor HEX de 6 dígitos ou pressione Enter para copiar.", "Enter a 6-digit HEX value or press Enter to copy.")
    } else {
        crate::lang::text("Formato inválido. Use #RRGGBB.", "Invalid format. Use #RRGGBB.")
    };

    let content = Rectangle::new().width(Parent).height(Parent)
        .padding(16.0)
        .child(Column::new(vec![
            boxed(m3::scale(Type::TitleMedium, crate::lang::text("Seletor de cores", "Color picker"), scheme.on_surface)),
            boxed(preview_box), boxed(input_box),
            boxed(Row::new(vec![boxed(eyedropper), boxed(copy)]).gap(8.0)),
            boxed(m3::scale_label(Type::LabelSmall, hint, scheme.on_surface_variant, 2)),
        ]).gap(14.0).width(Parent).height(Parent));

    overlay::layer("color", "cds-color-picker", &monitor.name, open, &scheme,
        overlay::Frame { width: WIDTH, height: HEIGHT, radius: m3::shape::XL, place: overlay::Place::Spot, full_screen: false },
        content, move |key| if key == Key::Enter { ColorPicker::copy_hex(); })
}
