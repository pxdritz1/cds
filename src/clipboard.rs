use std::fs;
use std::path::PathBuf;
use std::process::Command;
use std::thread;
use std::time::Duration;

use amane::{
    Center, Column, Key, LayerWindow, Monitor, Padding, Parent, Pointer, Rectangle, Row, ScrollArea,
    Service, SpaceBetween, TextInput, Widget,
};

use crate::island::widgets::{COPY, PASTE, SEARCH, boxed, chip, icon, icon_button, label, text};
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
            thread::sleep(Duration::from_millis(900));

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
                None
            } else {
                let index = clipboard.selected.min(count - 1);

                let entry = clipboard.matches().get(index).copied();

                entry.map(|index| clipboard.history[index].text.clone())
            }
        };

        if let Some(text) = text {
            Self::copy_text(&text);
        }
    }

    pub fn clear() {
        let mut clipboard = Self::write();

        clipboard.history.clear();
        clipboard.selected = 0;
        clipboard.save();
    }

    fn insert(&mut self, text: String) {
        // the newest copy sits on top, without the same text showing twice
        self.history.retain(|entry| entry.text != text);
        self.history.insert(0, Entry { text });
        self.history.truncate(LIMIT);

        self.selected = self.selected.min(self.history.len().saturating_sub(1));
    }

    // positions in the history that match what was typed
    fn matches(&self) -> Vec<usize> {
        let needle = self.query.trim().to_lowercase();

        self.history
            .iter()
            .enumerate()
            .filter(|(_, entry)| {
                needle.is_empty() || entry.text.to_lowercase().contains(&needle)
            })
            .map(|(index, _)| index)
            .collect()
    }

    fn file() -> PathBuf {
        cache_home().join("chloe-dynamic-shell/clipboard.conf")
    }

    fn load(&mut self) {
        let Ok(text) = fs::read_to_string(Self::file()) else {
            return;
        };

        for line in text.lines() {
            if line.is_empty() {
                continue;
            }

            self.history.push(Entry {
                text: unescape(line),
            });
        }

        self.history.truncate(LIMIT);
    }

    fn save(&self) {
        let file = Self::file();

        if let Some(folder) = file.parent() {
            let _ = fs::create_dir_all(folder);
        }

        let mut text = String::new();

        for entry in &self.history {
            text.push_str(&escape(&entry.text));
            text.push('\n');
        }

        let _ = fs::write(file, text);
    }
}

// a line per entry, with newlines escaped so entries stay apart
fn escape(text: &str) -> String {
    text.replace('\\', "\\\\")
        .replace('\n', "\\n")
        .replace('\r', "\\r")
}

fn unescape(line: &str) -> String {
    let mut text = String::new();
    let mut letters = line.chars();

    while let Some(letter) = letters.next() {
        if letter != '\\' {
            text.push(letter);

            continue;
        }

        match letters.next() {
            Some('n') => text.push('\n'),
            Some('r') => text.push('\r'),
            Some('\\') => text.push('\\'),
            Some(other) => {
                text.push('\\');
                text.push(other);
            }
            None => text.push('\\'),
        }
    }

    text
}

// the clipboard text, or nothing when it holds something else like an image
fn current() -> Option<String> {
    let output = Command::new("timeout")
        .arg("2")
        .arg("wl-paste")
        .arg("--no-newline")
        .output()
        .ok()?;

    if !output.status.success() {
        return None;
    }

    let text = String::from_utf8(output.stdout).ok()?;
    let text = text.trim_end_matches('\n').to_string();

    if text.trim().is_empty() || text.len() > MAX_BYTES {
        return None;
    }

    Some(text)
}

fn wl_copy(text: &str) {
    let text = String::from(text);

    // wl-copy forks a owner of its own; waiting here would hold up an input handler
    thread::spawn(move || {
        let Ok(mut child) = Command::new("wl-copy").arg("--").arg(&text).spawn() else {
            return;
        };

        let _ = child.wait();
    });
}

pub fn view(monitor: &Monitor) -> LayerWindow {
    let theme = Theme::read();
    let scheme = theme.scheme();

    let state = Overlay::read();
    let open = state.panel() == Panel::Clipboard;

    let clipboard = Clipboard::read();
    let matches = clipboard.matches();

    let selected = if matches.is_empty() {
        0
    } else {
        clipboard.selected().min(matches.len() - 1)
    };

    let count = matches.len();

    let search = overlay::search(
        "clipboard",
        INPUT,
        SEARCH,
        "Search the clipboard",
        open,
        scheme,
        Clipboard::set_query,
        |_| Clipboard::copy_selected(),
    );

    let header = Row::new(vec![
        boxed(text(
            "Clipboard",
            20.0,
            scheme.on_surface,
            amane::Weight::SemiBold,
        )),
        boxed(chip(
            "clipboard-clear",
            "Clear",
            84.0,
            false,
            scheme,
            Clipboard::clear,
        )),
    ])
    .width(Parent)
    .justify(SpaceBetween)
    .align(Center);

    let rows: Vec<Box<dyn Widget>> = matches
        .iter()
        .enumerate()
        .map(|(position, &index)| {
            boxed(entry_row(
                scheme,
                &clipboard.history[index].text,
                position,
                position == selected,
            ))
        })
        .collect();

    let list: Box<dyn Widget> = if rows.is_empty() {
        boxed(empty(scheme))
    } else {
        boxed(
            ScrollArea::new("clipboard-results", Column::new(rows).gap(4.0).width(Parent))
                .width(Parent)
                .height(LIST),
        )
    };

    let footer = Row::new(vec![
        boxed(text(
            if count == 0 {
                String::from("Nothing copied yet")
            } else {
                format!(
                    "{count} item{}  ·  ↑↓ choose  ·  Enter copies",
                    if count == 1 { "" } else { "s" }
                )
            },
            11.5,
            scheme.on_surface_variant,
            amane::Weight::Medium,
        )),
        boxed(text(
            "Esc closes",
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
                boxed(search),
                list,
                boxed(footer),
            ])
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
        WIDTH,
        HEIGHT,
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
        boxed(label(
            preview(text),
            13.0,
            color,
            amane::Weight::Medium,
            1,
        )),
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
        .on_click(move |_| Clipboard::copy_text(&click))
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
                        "The clipboard is empty",
                        15.0,
                        scheme.on_surface,
                        amane::Weight::Medium,
                    ))])
                    .width(Parent)
                    .justify(Center),
                ),
                boxed(
                    Row::new(vec![boxed(text(
                        "Copy something and it shows up here",
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
