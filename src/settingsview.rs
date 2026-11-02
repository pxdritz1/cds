use amane::{
    Center, Column, Key, LayerWindow, Monitor, Padding, Parent, Rectangle, Row, ScrollArea,
    Service, Start, TextInput, Widget,
};

use crate::island::widgets::{
    DARK_MODE, LIGHT_MODE, OPEN_NEW, SUNNY, TUNE, boxed, card, chip, icon, label, text,
};
use crate::m3;
use crate::overlay::{self, Overlay, Panel};
use crate::settings::Settings;
use crate::theme::{Scheme, Theme};

const WIDTH: f32 = 620.0;
const HEIGHT: f32 = 470.0;
const PADDING: f32 = 16.0;

const DIRECTORY_INPUT: &str = "cds-settings-directory";
const LOCATION_INPUT: &str = "cds-settings-location";

#[derive(Default)]
pub struct SettingsView {
    directory: String,
    location: String,
}

impl Service for SettingsView {
    fn new() -> Self {
        let settings = Settings::read();

        Self {
            directory: String::from(settings.directory_path()),
            location: String::from(settings.location()),
        }
    }

    // the panel only changes through input and ipc
    fn listen() {}
}

impl SettingsView {
    // keeps the two fields in sync with what the service holds
    pub fn reset() {
        let settings = Settings::read();

        let directory = String::from(settings.directory_path());
        let location = String::from(settings.location());

        drop(settings);

        TextInput::set_text(DIRECTORY_INPUT, &directory);
        TextInput::set_text(LOCATION_INPUT, &location);

        let mut view = Self::write();

        view.directory = directory;
        view.location = location;
    }

    pub fn toggle() {
        let showing = Overlay::read().panel() == Panel::Config;

        if showing {
            Overlay::close();

            return;
        }

        Self::reset();
        Overlay::open(Panel::Config);
    }
}

// a section title with its icon, the little header inside every card
fn section(glyph: &'static str, name: &str, scheme: &Scheme) -> Row {
    Row::new(vec![
        Box::new(icon(glyph, 20.0, scheme.primary)),
        Box::new(label(name, 15.0, scheme.on_surface, amane::Weight::SemiBold, 1)),
    ])
    .gap(10.0)
    .align(Center)
}

// an editable row: a caption and a text input under it
fn field(
    name: &str,
    input: &'static str,
    hint: &str,
    scheme: &Scheme,
    on_change: impl Fn(String) + 'static,
    on_submit: impl Fn(String) + 'static,
) -> Column {
    let input = TextInput::new(input)
        .size(13.5)
        .color(scheme.on_surface)
        .placeholder(hint)
        .on_change(on_change)
        .on_submit(on_submit)
        .width(Parent);

    Column::new(vec![
        Box::new(label(name, 12.0, scheme.on_surface_variant, amane::Weight::Medium, 1)),
        Box::new(
            Rectangle::new()
                .width(Parent)
                .height(42.0)
                .fill(scheme.surface_container_highest)
                .radius(14.0)
                .padding(Padding {
                    top: 0.0,
                    right: 12.0,
                    bottom: 0.0,
                    left: 12.0,
                })
                .align_child(Center, Center)
                .child(input),
        ),
    ])
    .gap(6.0)
    .width(Parent)
}

pub fn view(monitor: &Monitor) -> LayerWindow {
    let theme = Theme::read();
    let scheme = theme.scheme();

    let open = Overlay::read().panel() == Panel::Config;

    let settings = Settings::read();

    let dark = settings.dark();

    let current = String::from(settings.scheme_name());

    let directory = String::from(settings.directory_path());

    let location = String::from(settings.location());

    drop(settings);

    const SCHEMES: [&str; 8] = [
        "scheme-tonal-spot",
        "scheme-content",
        "scheme-expressive",
        "scheme-neutral",
        "scheme-vibrant",
        "scheme-rainbow",
        "scheme-fruit-salad",
        "scheme-fidelity",
    ];

    // one chip per scheme, in rows of three, the selected one highlighted
    let scheme_rows: Vec<Box<dyn Widget>> = SCHEMES
        .chunks(3)
        .map(|group| {
            boxed(
                Row::new(
                    group
                        .iter()
                        .map(|name| {
                            boxed(chip(
                                &format!("sv:{name}"),
                                &scheme_chip_name(name),
                                128.0,
                                current == *name,
                                scheme,
                                move || {
                                    Settings::set_scheme(name);
                                },
                            ))
                        })
                        .collect(),
                )
                .gap(8.0),
            )
        })
        .collect();

    let appearance = card(WIDTH - PADDING * 2.0, scheme.surface_container_low).child(
        Column::new(vec![
            Box::new(section(TUNE, "Aparência", scheme)),
            Box::new(
                Row::new(vec![
                    boxed(chip(
                        "sv:dark",
                        "Escuro",
                        118.0,
                        dark,
                        scheme,
                        move || Settings::set_dark(true),
                    )),
                    boxed(chip(
                        "sv:light",
                        "Claro",
                        118.0,
                        !dark,
                        scheme,
                        move || Settings::set_dark(false),
                    )),
                    boxed(
                        Rectangle::new()
                            .width(Parent)
                            .height(36.0)
                            .align_child(Center, Start)
                            .child(icon(
                                if dark { DARK_MODE } else { LIGHT_MODE },
                                22.0,
                                scheme.primary,
                            )),
                    ),
                ])
                .gap(8.0)
                .width(Parent),
            ),
            Box::new(
                Column::new(scheme_rows).gap(8.0).width(Parent),
            ),
        ])
        .gap(12.0)
        .width(Parent),
    );

    /*
     * the wallpaper section: one input moves where the dash reads images
     * from, applying as soon as the path is submitted
     */
    let wallpapers = card(WIDTH - PADDING * 2.0, scheme.surface_container_low).child(
        Column::new(vec![
            Box::new(section(OPEN_NEW, "Papéis de parede", scheme)),
            Box::new(field(
                "Diretório das imagens",
                DIRECTORY_INPUT,
                &directory,
                scheme,
                {
                    let current = directory.clone();

                    move |_text| {
                        // nothing while typing; the submit applies it
                        let _ = current.clone();
                    }
                },
                move |path| {
                    Settings::set_directory(&path);
                },
            )),
            Box::new(text(
                &format!("{} imagens encontradas", Settings::read().wallpapers().len()),
                11.5,
                scheme.on_surface_variant,
                amane::Weight::Regular,
            )),
        ])
        .gap(10.0)
        .width(Parent),
    );

    // the weather section, where the dash reads its forecast spot from
    let climate = card(WIDTH - PADDING * 2.0, scheme.surface_container_low).child(
        Column::new(vec![
            Box::new(section(SUNNY, "Clima", scheme)),
            Box::new(field(
                "Coordenadas (latitude,longitude — vazio volta ao automático)",
                LOCATION_INPUT,
                if location.is_empty() {
                    "automático pelo ip"
                } else {
                    &location
                },
                scheme,
                move |_text| {},
                move |spot| {
                    Settings::set_location(&spot);
                },
            )),
        ])
        .gap(10.0)
        .width(Parent),
    );

    let footer = Row::new(vec![
        boxed(text(
            "As mudanças regeneram o matugen na hora",
            11.5,
            scheme.on_surface_variant,
            amane::Weight::Medium,
        )),
        boxed(chip(
            "sv:refresh",
            "Regenerar",
            108.0,
            false,
            scheme,
            || {
                let settings = Settings::read();

                let (wallpaper, scheme_name, dark) = (
                    settings.wallpaper().to_string(),
                    settings.scheme_name().to_string(),
                    settings.dark(),
                );

                drop(settings);

                if !wallpaper.is_empty() {
                    crate::theme::apply("image", &wallpaper, &scheme_name, dark);
                }
            },
        )),
    ])
    .width(Parent)
    .justify(amane::SpaceBetween)
    .align(Center);

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
                boxed(
                    ScrollArea::new(
                        "settings-panels",
                        Column::new(vec![
                            Box::new(appearance),
                            Box::new(wallpapers),
                            Box::new(climate),
                        ])
                        .gap(12.0)
                        .width(Parent),
                    )
                    .width(Parent)
                    .height(Parent),
                ),
                Box::new(footer),
            ])
            .width(Parent)
            .height(Parent)
            .gap(10.0),
        );

    overlay::layer(
        "settingsview",
        "cds-settingsview",
        &monitor.name,
        open,
        scheme,
        overlay::Frame {
            width: WIDTH,
            height: HEIGHT,
            radius: m3::shape::XL,
            place: overlay::Place::Under,
        },
        content,
        |key| {
            if key == Key::Escape {
                Overlay::close();
            }
        },
    )
}

// "scheme-tonal-spot" becomes "Tonal", for the chip labels
fn scheme_chip_name(scheme: &str) -> String {
    let name = scheme.trim_start_matches("scheme-").replace('-', " ");

    let mut letters = name.chars();

    match letters.next() {
        Some(first) => first.to_uppercase().collect::<String>() + letters.as_str(),
        None => String::from(name.as_str()),
    }
}
