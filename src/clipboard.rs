use std::fs;
use std::path::PathBuf;
use std::process::Command;
use std::thread;
use std::time::Duration;

use amane::{
    Center, Column, Key, LayerWindow, Monitor, Padding, Parent, Pointer, Rectangle, Row,
    ScrollArea, Service, SpaceBetween, TextInput, Widget,
};

use crate::island::widgets::{COPY, PASTE, SEARCH, boxed, chip, icon, icon_button, label, text};
use crate::m3;
use crate::overlay::{self, Overlay, Panel, row_fill};
use crate::theme::{Scheme, Theme, cache_home, fade};

const WIDTH: f32 = 620.0;
const HEIGHT: f32 = 500.0;
const PADDING: f32 = 16.0;
const LIST: f32 = 308.0;

const INPUT: &str = "cds-clipboard-input";
const LIMIT: usize = 80;
const MAX_BYTES: usize = 200_000;

#[derive(Clone)]
struct Entry {
    text: String,
}

#[derive(Default)]
pub struct Clipboard {
    history: Vec<Entry>,
    query: String,
    selected: usize,

    // the text last seen on the clipboard, so a change is added once
    last: String,
}

impl Service for Clipboard {
    fn new() -> Self {
        let mut clipboard = Self::default();

        clipboard.load();

        clipboard
    }

    /*
     * wl-paste is asked on this thread instead of holding the service lock,
     * so copying a large file never makes the window wait
     */
    fn listen() {
        loop {
            // Keep clipboard updates responsive without launching wl-paste almost every frame.
            thread::sleep(Duration::from_millis(1500));

            let Some(text) = current() else {
                continue;
            };

            // reading first keeps an unchanged clipboard from waking every window
            if Self::read().last == text {
                continue;
            }

            let mut clipboard = Self::write();

            clipboard.last = text.clone();
            clipboard.insert(text);
            clipboard.save();
        }
    }
}

impl Clipboard {
    pub fn query(&self) -> &str {
        &self.query
    }

    pub fn selected(&self) -> usize {
        self.selected
    }

    pub fn set_query(query: String) {
        let mut clipboard = Self::write();

        clipboard.query = query;
        clipboard.selected = 0;
    }

    pub fn move_by(delta: isize, count: usize) {
        if count == 0 {
            return;
        }

        let mut clipboard = Self::write();

        let last = count.saturating_sub(1) as isize;
        let current = (clipboard.selected.min(last as usize)) as isize;

        clipboard.selected = (current + delta).clamp(0, last) as usize;
    }

    pub fn reset() {
        TextInput::set_text(INPUT, "");

        let mut clipboard = Self::write();

        clipboard.query.clear();
        clipboard.selected = 0;
    }

    pub fn toggle() {
        let showing = Overlay::read().panel() == Panel::Clipboard;

        if showing {
            Overlay::close();

            return;
        }

        Self::reset();
        Overlay::open(Panel::Clipboard);
    }

    pub fn open() {
        Self::reset();
        Overlay::open(Panel::Clipboard);
    }

    // puts an entry back on the clipboard and closes the list
    pub fn copy_text(text: &str) {
        {
            let mut clipboard = Self::write();

            clipboard.last = String::from(text);
            clipboard.insert(String::from(text));
            clipboard.save();
        }

        wl_copy(text);
        Overlay::close();
    }

    pub fn copy_selected() {
        let text = {
            let clipboard = Self::read();
            let count = clipboard.matches().len();

            if count == 0 {
                String::from(crate::lang::text("Nada copiado ainda", "Nothing copied yet"))
            } else if crate::settings::Settings::read().language() == "en" {
                format!("{count} items  ·  ↑↓ choose  ·  Enter copies")
            } else {
                format!("{count} itens  ·  ↑↓ selecionar  ·  Enter copiar")
            }
            },
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
            Column::new(vec![boxed(header), boxed(search), list, boxed(footer)])
                .width(Parent)
                .height(Parent)
                .gap(12.0),
        );

    overlay::layer(
        "clipboard",
        "cds-clipboard",
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
        move |key| match key {
            Key::Down | Key::Tab => Clipboard::move_by(1, count),
            Key::Up => Clipboard::move_by(-1, count),
            Key::Enter => Clipboard::copy_selected(),
            _ => {}
        },
    )
}

fn entry_row(scheme: &Scheme, text: &str, position: usize, selected: bool) -> Rectangle {
    let key = format!("clipboard:{position}");
    let (fill, color) = row_fill(&key, selected, scheme);

    let badge = Rectangle::new()
        .width(36.0)
        .height(36.0)
        .fill(scheme.primary_container)
        .radius(10.0)
        .align_child(Center, Center)
        .child(icon(PASTE, 19.0, scheme.on_primary_container));

    let info = Column::new(vec![
        boxed(label(preview(text), 13.0, color, amane::Weight::Medium, 1)),
        boxed(label(
            meta(text),
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

    let owned = String::from(text);
    let click = owned.clone();

    Rectangle::new()
        .width(Parent)
        .height(52.0)
        .fill(fill)
        .radius(16.0)
        .padding(8.0)
        .cursor(Pointer)
        .on_hover({
            let owned = key.clone();

            move |inside| crate::motion::set_hover(owned.clone(), inside)
        })
        .on_click(move |button| {
            if button == amane::Button::Left {
                Clipboard::copy_text(&click);
            }
        })
        .child(
            Row::new(vec![
                boxed(badge),
                boxed(info),
                boxed(icon_button(
                    &format!("clipboard-copy:{position}"),
                    COPY,
                    30.0,
                    scheme,
                    move || Clipboard::copy_text(&owned),
                )),
            ])
            .gap(12.0)
            .align(Center)
            .width(Parent)
            .height(Parent),
        )
}

// clipboard text flattened to a single line that fits a row
fn preview(text: &str) -> String {
    let flat = text.split_whitespace().collect::<Vec<_>>().join(" ");

    if flat.chars().count() > 78 {
        let mut cut: String = flat.chars().take(77).collect();

        cut.push('…');

        cut
    } else {
        flat
    }
}

fn meta(text: &str) -> String {
    let characters = text.chars().count();
    let lines = text.lines().count().max(1);

    if lines > 1 {
        format!("{characters} characters · {lines} lines")
    } else {
        format!("{characters} characters")
    }
}

fn empty(scheme: &Scheme) -> Rectangle {
    let circle = Rectangle::new()
        .width(64.0)
        .height(64.0)
        .fill(scheme.surface_container_highest)
        .radius(32.0)
        .align_child(Center, Center)
        .child(icon(PASTE, 30.0, scheme.on_surface_variant));

    Rectangle::new()
        .width(Parent)
        .height(LIST)
        .align_child(Center, Center)
        .child(
            Column::new(vec![
                boxed(Row::new(vec![boxed(circle)]).width(Parent).justify(Center)),
                boxed(
                    Row::new(vec![boxed(text(
                        crate::lang::text("O clipboard está vazio", crate::lang::text("O clipboard está vazio", "The clipboard is empty")),
                        15.0,
                        scheme.on_surface,
                        amane::Weight::Medium,
                    ))])
                    .width(Parent)
                    .justify(Center),
                ),
                boxed(
                    Row::new(vec![boxed(text(
                        crate::lang::text("Copie algo para aparecer aqui", "Copy something and it will show up here"),
                        12.0,
                        scheme.on_surface_variant,
                        amane::Weight::Regular,
                    ))])
                    .width(Parent)
                    .justify(Center),
                ),
            ])
            .gap(8.0)
            .width(Parent)
            .align(Center),
        )
}
