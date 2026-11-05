use amane::{
    Center, Column, Key, Padding, Parent, Rectangle, Row, ScrollArea, Service, Start, TextInput,
    Widget, Window,
};

use crate::island::widgets::{
    DARK_MODE, LIGHT_MODE, OPEN_NEW, CLEAR_DAY, TUNE, MUSIC, boxed, chip, icon, label, text,
};
use crate::settings::Settings;
use crate::theme::{Scheme, Theme};

const WIDTH: f32 = 620.0;
const HEIGHT: f32 = 560.0;
const PADDING: f32 = 16.0;

const DIRECTORY_INPUT: &str = "cds-settings-directory";
const LOCATION_INPUT: &str = "cds-settings-location";

fn settings_card(width: f32, height: f32, fill: amane::Color) -> Rectangle {
    Rectangle::new()
        .width(width)
        .height(height)
        .fill(fill)
        .radius(24.0)
        .clip()
        .padding(16.0)
}

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
        // Amane keeps one instance per name, so repeated requests do not
        // create duplicate settings windows.
        Self::reset();
        amane::open_window("cds-settings", window);
    }
}

// a section title with its icon, the little header inside every card
fn section(glyph: &'static str, name: &str, scheme: &Scheme) -> Row {
    Row::new(vec![
        Box::new(icon(glyph, 20.0, scheme.primary)),
        Box::new(label(
            name,
            15.0,
            scheme.on_surface,
            amane::Weight::SemiBold,
            1,
        )),
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
        Box::new(label(
            name,
            12.0,
            scheme.on_surface_variant,
            amane::Weight::Medium,
            1,
        )),
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

pub fn window() -> Window {
    let theme = Theme::read();
    let scheme = theme.scheme();

    let settings = Settings::read();

    let language = String::from(settings.language());
    let dark = settings.dark();
    let hover_dash = settings.flag("hover_dash");
    let hover_delay_ms = settings.hover_delay_ms();
    let trans_meter_colors = settings.flag("trans_meter_colors");
    let media_title = settings.flag("media_show_title");
    let media_artist = settings.flag("media_show_artist");
    let media_clock = settings.flag("media_show_clock");
    let media_artwork = settings.flag("media_show_artwork");
    let media_title_max_width = settings.media_title_max_width();
    let wallpaper_rotation = settings.wallpaper_rotation();
    let wallpaper_interval_minutes = settings.wallpaper_interval_minutes();
    let shell_mode = settings.shell_mode();
    let motion_style = String::from(settings.motion_style());
    let dock_position = String::from(settings.dock_position());
    let dock_alignment = String::from(settings.dock_alignment());
    let show_workspaces = settings.flag("show_workspaces");
    let show_center_island = settings.flag("show_center_island");
    let show_status = settings.flag("show_status");

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
                                    Settings::set_scheme_live(name);
                                },
                            ))
                        })
                        .collect(),
                )
                .gap(8.0),
            )
        })
        .collect();

    let interface = settings_card(WIDTH - PADDING * 2.0, 360.0, scheme.surface_container_low).child(
        Column::new(vec![
            Box::new(section(TUNE, crate::lang::text("Layout da interface", "Interface layout"), scheme)),
            Box::new(Row::new(vec![
                boxed(chip("sv:layout-1", crate::lang::text("Ilhas", "Islands"), 88.0, shell_mode == 1, scheme, || Settings::set_shell_mode(1))),
                boxed(chip("sv:layout-2", "Dock", 88.0, shell_mode == 2, scheme, || Settings::set_shell_mode(2))),
                boxed(chip("sv:layout-3", crate::lang::text("Dock + barra", "Dock + bar"), 124.0, shell_mode == 3, scheme, || Settings::set_shell_mode(3))),
            ]).gap(8.0).width(Parent)),
            Box::new(section(TUNE, crate::lang::text("Linguagem de movimento", "Motion language"), scheme)),
            Box::new(Row::new(vec![
                boxed(chip("sv:motion-m3", "Material 3", 116.0, motion_style == "m3", scheme, || Settings::set_motion_style("m3"))),
                boxed(chip("sv:motion-fluent", "Fluent", 100.0, motion_style == "fluent", scheme, || Settings::set_motion_style("fluent"))),
            ]).gap(8.0).width(Parent)),
            Box::new(section(TUNE, crate::lang::text("Posição do dock", "Dock position"), scheme)),
            Box::new(Row::new(vec![
                boxed(chip("sv:dock-top", crate::lang::text("Topo", "Top"), 88.0, dock_position == "top", scheme, || Settings::set_dock_position("top"))),
                boxed(chip("sv:dock-bottom", crate::lang::text("Base", "Bottom"), 88.0, dock_position == "bottom", scheme, || Settings::set_dock_position("bottom"))),
            ]).gap(8.0).width(Parent)),
            Box::new(section(TUNE, crate::lang::text("Alinhamento do dock", "Dock alignment"), scheme)),
            Box::new(Row::new(vec![
                boxed(chip("sv:dock-left", crate::lang::text("Esquerda", "Left"), 100.0, dock_alignment == "left", scheme, || Settings::set_dock_alignment("left"))),
                boxed(chip("sv:dock-center", crate::lang::text("Centro", "Center"), 100.0, dock_alignment == "center", scheme, || Settings::set_dock_alignment("center"))),
                boxed(chip("sv:dock-right", crate::lang::text("Direita", "Right"), 100.0, dock_alignment == "right", scheme, || Settings::set_dock_alignment("right"))),
            ]).gap(8.0).width(Parent)),
            Box::new(section(TUNE, crate::lang::text("Visibilidade das ilhas", "Island visibility"), scheme)),
            Box::new(Row::new(vec![
                boxed(chip("sv:show-workspaces", crate::lang::text("Espaços", "Workspaces"), 112.0, show_workspaces, scheme, || Settings::toggle_flag("show_workspaces"))),
                boxed(chip("sv:show-center", crate::lang::text("Centro", "Center"), 100.0, show_center_island, scheme, || Settings::toggle_flag("show_center_island"))),
                boxed(chip("sv:show-status", crate::lang::text("Status", "Status"), 100.0, show_status, scheme, || Settings::toggle_flag("show_status"))),
            ]).gap(8.0).width(Parent)),
        ])
        .gap(10.0)
        .width(Parent),
    );

    let behavior = settings_card(WIDTH - PADDING * 2.0, 300.0, scheme.surface_container_low).child(
        Column::new(vec![
            Box::new(section(TUNE, crate::lang::text("Comportamento", "Behavior"), scheme)),
            Box::new(chip(
                "sv:hover-dash",
                if hover_dash {
                    crate::lang::text("Abrir Dash ao passar o mouse: ligado", "Open Dash on hover: on")
                } else {
                    crate::lang::text("Abrir Dash ao passar o mouse: desligado", "Open Dash on hover: off")
                },
                360.0,
                hover_dash,
                scheme,
                || {
                    Settings::toggle_flag("hover_dash");
                    crate::ui::Ui::refresh();
                },
            )),
            Box::new(section(TUNE, crate::lang::text("Atraso do hover", "Hover delay"), scheme)),
            Box::new(
                Row::new(vec![
                    boxed(chip("sv:hover-delay-250", "250 ms", 88.0, hover_delay_ms == 250, scheme, || Settings::set_hover_delay_ms(250))),
                    boxed(chip("sv:hover-delay-500", "500 ms", 88.0, hover_delay_ms == 500, scheme, || Settings::set_hover_delay_ms(500))),
                    boxed(chip("sv:hover-delay-750", "750 ms", 88.0, hover_delay_ms == 750, scheme, || Settings::set_hover_delay_ms(750))),
                    boxed(chip("sv:hover-delay-1000", "1 s", 88.0, hover_delay_ms == 1000, scheme, || Settings::set_hover_delay_ms(1000))),
                ])
                .gap(8.0)
                .width(Parent),
            ),
            Box::new(section(TUNE, crate::lang::text("Cores dos medidores", "Meter colors"), scheme)),
            Box::new(chip(
                "sv:meter-colors",
                if trans_meter_colors {
                    crate::lang::text("Bandeira trans", "Trans flag")
                } else {
                    crate::lang::text("Cores do tema", "Theme colors")
                },
                220.0,
                trans_meter_colors,
                scheme,
                || {
                    Settings::toggle_flag("trans_meter_colors");
                    crate::ui::Ui::refresh();
                },
            )),
        ])
        .gap(10.0)
        .width(Parent),
    );

    let media_settings = settings_card(WIDTH - PADDING * 2.0, 184.0, scheme.surface_container_low).child(
        Column::new(vec![
            Box::new(section(MUSIC, crate::lang::text("Player de mídia", "Media player"), scheme)),
            Box::new(Row::new(vec![
                boxed(chip("sv:media-title", crate::lang::text("Título", "Title"), 112.0, media_title, scheme, || { Settings::toggle_flag("media_show_title"); crate::ui::Ui::refresh(); })),
                boxed(chip("sv:media-artist", crate::lang::text("Artista", "Artist"), 112.0, media_artist, scheme, || { Settings::toggle_flag("media_show_artist"); crate::ui::Ui::refresh(); })),
                boxed(chip("sv:media-clock", crate::lang::text("Relógio", "Clock"), 112.0, media_clock, scheme, || { Settings::toggle_flag("media_show_clock"); crate::ui::Ui::refresh(); })),
                boxed(chip("sv:media-artwork", crate::lang::text("Capa", "Artwork"), 112.0, media_artwork, scheme, || { Settings::toggle_flag("media_show_artwork"); crate::ui::Ui::refresh(); })),
            ]).gap(8.0).width(Parent)),
            Box::new(section(TUNE, crate::lang::text("Largura máxima do título", "Maximum title width"), scheme)),
            Box::new(Row::new(vec![
                boxed(chip("sv:media-width-104", "104 px", 72.0, media_title_max_width == 104, scheme, || Settings::set_media_title_max_width(104))),
                boxed(chip("sv:media-width-160", "160 px", 72.0, media_title_max_width == 160, scheme, || Settings::set_media_title_max_width(160))),
                boxed(chip("sv:media-width-220", "220 px", 72.0, media_title_max_width == 220, scheme, || Settings::set_media_title_max_width(220))),
                boxed(chip("sv:media-width-280", "280 px", 72.0, media_title_max_width == 280, scheme, || Settings::set_media_title_max_width(280))),
                boxed(chip("sv:media-width-320", "320 px", 72.0, media_title_max_width == 320, scheme, || Settings::set_media_title_max_width(320))),
                boxed(chip("sv:media-width-400", "400 px", 72.0, media_title_max_width == 400, scheme, || Settings::set_media_title_max_width(400))),
                boxed(chip("sv:media-width-520", "520 px", 72.0, media_title_max_width == 520, scheme, || Settings::set_media_title_max_width(520))),
            ]).gap(6.0).width(Parent)),
        ]).gap(9.0).width(Parent),
    );

    let appearance = settings_card(WIDTH - PADDING * 2.0, 330.0, scheme.surface_container_low).child(
        Column::new(vec![
            Box::new(section(TUNE, crate::lang::text("Aparência", "Appearance"), scheme)),
            Box::new(
                Row::new(vec![
                    boxed(chip("sv:dark", crate::lang::text("Escuro", "Dark"), 118.0, dark, scheme, move || {
                        Settings::set_dark_live(true)
                    })),
                    boxed(chip("sv:light", crate::lang::text("Claro", "Light"), 118.0, !dark, scheme, move || {
                        Settings::set_dark_live(false)
                    })),
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
            Box::new(Column::new(scheme_rows).gap(8.0).width(Parent)),
            Box::new(section(TUNE, crate::lang::text("Idioma", "Language"), scheme)),
            Box::new(Row::new(vec![
                boxed(chip("sv:lang-pt", "Português (Brasil)", 150.0, language == "pt-BR", scheme, || Settings::set_language("pt-BR"))),
                boxed(chip("sv:lang-en", "English", 110.0, language == "en", scheme, || Settings::set_language("en"))),
            ]).gap(8.0).width(Parent)),
        ])
        .gap(12.0)
        .width(Parent),
    );

    /*
     * the wallpaper section: one input moves where the dash reads images
     * from, applying as soon as the path is submitted
     */
    let wallpapers = settings_card(WIDTH - PADDING * 2.0, 146.0, scheme.surface_container_low).child(
        Column::new(vec![
            Box::new(section(OPEN_NEW, crate::lang::text("Papéis de parede", "Wallpapers"), scheme)),
            Box::new(field(
                crate::lang::text("Diretório das imagens", "Image folder"),
                DIRECTORY_INPUT,
                &directory,
                scheme,
                // The directory is applied only when the field is submitted.
                move |_text| {},
                move |path| {
                    Settings::set_directory(&path);
                },
            )),
            Box::new(text(
                &format!(
                    "{} {}",
                    Settings::read().wallpapers().len(),
                    crate::lang::text("imagens encontradas", "images found")
                ),
                11.5,
                scheme.on_surface_variant,
                amane::Weight::Regular,
            )),
        ])
        .gap(10.0)
        .width(Parent),
    );

    let wallpaper_rotation_card = settings_card(WIDTH - PADDING * 2.0, 148.0, scheme.surface_container_low).child(
        Column::new(vec![
            Box::new(section(OPEN_NEW, crate::lang::text("Rotação automática", "Automatic rotation"), scheme)),
            Box::new(chip(
                "sv:wallpaper-rotation",
                if wallpaper_rotation {
                    crate::lang::text("Trocar wallpaper automaticamente: ligado", "Automatic wallpaper rotation: on")
                } else {
                    crate::lang::text("Trocar wallpaper automaticamente: desligado", "Automatic wallpaper rotation: off")
                },
                360.0,
                wallpaper_rotation,
                scheme,
                move || Settings::set_wallpaper_rotation(!wallpaper_rotation),
            )),
            Box::new(Row::new(vec![
                boxed(chip("sv:wallpaper-interval-5", "5 min", 88.0, wallpaper_interval_minutes == 5, scheme, || Settings::set_wallpaper_interval_minutes(5))),
                boxed(chip("sv:wallpaper-interval-15", "15 min", 88.0, wallpaper_interval_minutes == 15, scheme, || Settings::set_wallpaper_interval_minutes(15))),
                boxed(chip("sv:wallpaper-interval-30", "30 min", 88.0, wallpaper_interval_minutes == 30, scheme, || Settings::set_wallpaper_interval_minutes(30))),
                boxed(chip("sv:wallpaper-interval-60", "1 h", 88.0, wallpaper_interval_minutes == 60, scheme, || Settings::set_wallpaper_interval_minutes(60))),
            ]).gap(8.0).width(Parent)),
        ]).gap(8.0).width(Parent),
    );

    // the weather section, where the dash reads its forecast spot from
    let climate = settings_card(WIDTH - PADDING * 2.0, 122.0, scheme.surface_container_low).child(
        Column::new(vec![
            Box::new(section(CLEAR_DAY, crate::lang::text("Clima", "Weather"), scheme)),
            Box::new(field(
                crate::lang::text("Coordenadas (latitude,longitude — vazio volta ao automático)", "Coordinates (latitude,longitude — empty uses automatic location)"),
                LOCATION_INPUT,
                if location.is_empty() {
                    crate::lang::text("automático pelo ip", "automatic (IP)")
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
            crate::lang::text("As mudanças regeneram o matugen na hora", "Changes are applied with Matugen"),
            11.5,
            scheme.on_surface_variant,
            amane::Weight::Medium,
        )),
        boxed(chip(
            "sv:refresh",
            crate::lang::text("Regenerar", "Regenerate"),
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
                    crate::theme::apply_async("image", &wallpaper, &scheme_name, dark);
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
        .fill(scheme.surface)
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
                            Box::new(interface),
                            Box::new(media_settings),
                            Box::new(behavior),
                            Box::new(wallpapers),
                            Box::new(wallpaper_rotation_card),
                            Box::new(climate),
                        ])
                        .gap(12.0)
                        .width(Parent),
                    )
                    .width(Parent)
                    .height(HEIGHT - PADDING * 2.0 - 46.0),
                ),
                Box::new(footer),
            ])
            .width(Parent)
            .height(Parent)
            .gap(10.0),
        );

    Window::new()
        .title(crate::lang::text("Chloe Dynamic Shell — Configurações", "Chloe Dynamic Shell — Settings"))
        .size(WIDTH, HEIGHT)
        .resizable(false)
        .child(content)
        .on_key(|key| {
            if key == Key::Escape {
                amane::close_window("cds-settings");
            }
        })
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
