use std::process::Command;
use std::thread;

use amane::{
    Center, Column, Key, LayerWindow, Monitor, Padding, Parent, Pointer, Rectangle, Row,
    ScrollArea, Service, TextInput, Widget,
};

use crate::m3::{self, boxed};
use crate::overlay::{self, Overlay, Panel};
use crate::theme::Theme;

const INPUT: &str = "cds-emoji-search";
const WIDTH: f32 = 620.0;
const HEIGHT: f32 = 500.0;
const COLUMNS: usize = 8;

// Keep the catalog small, searchable and dependency-free. Aliases are translated
// by matching both the emoji and its stable English keyword.
const EMOJIS: &[(&str, &str)] = &[
    ("😀","grinning happy face"),("😃","smile happy face"),("😄","smile happy face"),("😁","beaming grin"),
    ("😆","laughing face"),("😅","sweat laugh"),("🤣","rolling laugh"),("😂","tears joy laugh"),
    ("🙂","slight smile"),("🙃","upside down"),("😉","wink"),("😊","blush smile"),
    ("😍","heart eyes love"),("🥰","hearts love"),("😘","kiss"),("😗","kiss"),
    ("😋","yum delicious"),("😛","tongue"),("😜","wink tongue"),("🤪","crazy"),
    ("🤗","hug"),("🤔","think"),("🤨","skeptical"),("😐","neutral"),
    ("😶","silent"),("🙄","eye roll"),("😏","smirk"),("😴","sleep"),
    ("🥳","party celebration"),("😭","cry sad"),("😢","sad"),("😤","angry"),
    ("😡","rage angry"),("🤯","mind blown"),("🥺","pleading"),("😱","scream"),
    ("🤓","nerd"),("😎","cool sunglasses"),("🫡","salute"),("🫶","heart hands"),
    ("❤️","red heart love"),("🩷","pink heart love"),("🧡","orange heart"),("💛","yellow heart"),
    ("💚","green heart"),("💙","blue heart"),("🩵","light blue heart"),("💜","purple heart"),
    ("🤎","brown heart"),("🖤","black heart"),("🩶","gray heart"),("🤍","white heart"),
    ("💔","broken heart"),("💕","two hearts"),("💖","sparkling heart"),("💗","growing heart"),
    ("💘","cupid heart"),("💝","gift heart"),("✨","sparkles magic"),("⭐","star"),
    ("🌟","glowing star"),("🔥","fire hot"),("💥","collision boom"),("🎉","party popper"),
    ("🎊","confetti"),("🎈","balloon"),("🎀","ribbon"),("🌈","rainbow"),
    ("☀️","sun sunny"),("🌙","moon night"),("⚡","lightning"),("☁️","cloud"),
    ("🌧️","rain"),("❄️","snowflake cold"),("🌸","cherry blossom flower"),("🌻","sunflower"),
    ("🌷","tulip flower"),("🪻","hyacinth flower"),("🍀","four leaf clover"),("🌿","herb leaf"),
    ("🍄","mushroom"),("🪷","lotus"),("🐈","cat"),("🐈‍⬛","black cat"),
    ("🐕","dog"),("🦊","fox"),("🐸","frog"),("🐼","panda"),
    ("🦋","butterfly"),("🐝","bee"),("🐢","turtle"),("🦄","unicorn"),
    ("🍓","strawberry"),("🍒","cherry"),("🍑","peach"),("🍉","watermelon"),
    ("🍇","grapes"),("🍍","pineapple"),("🥑","avocado"),("🍕","pizza"),
    ("🍔","burger"),("🍟","fries"),("🍩","donut"),("🍪","cookie"),
    ("☕","coffee"),("🧋","bubble tea"),("🥤","drink"),("🍰","cake"),
    ("🎧","headphones music"),("🎵","music note"),("🎶","music notes"),("🎮","game"),
    ("🕹️","joystick"),("🎨","art palette"),("📷","camera"),("💻","computer"),
    ("📱","phone"),("⌨️","keyboard"),("🖱️","mouse"),("💡","idea light bulb"),
    ("📌","pin"),("📎","paperclip"),("✏️","pencil"),("📚","books"),
    ("🚀","rocket"),("✈️","airplane"),("🚗","car"),("🚲","bicycle"),
    ("🏳️‍⚧️","transgender flag pride"),("🏳️‍🌈","rainbow pride flag"),("💗","pink heart"),
    ("👍","thumbs up"),("👎","thumbs down"),("👏","clap"),("🙌","raised hands"),
    ("🙏","please thanks"),("🤝","handshake"),("💅","nail polish slay"),("👀","eyes"),
    ("💀","skull"),("🤡","clown"),("🫠","melting"),("🧠","brain"),
    ("✅","check"),("❌","cross"),("⚠️","warning"),("❓","question"),
    ("🔒","lock"),("🔑","key"),("🔔","bell"),("📍","location"),
];

#[derive(Default)]
pub struct EmojiPicker {
    query: String,
    selected: usize,
}

impl Service for EmojiPicker {
    fn new() -> Self { Self::default() }
    fn listen() {}
}

impl EmojiPicker {
    pub fn set_query(query: String) {
        let mut state = Self::write();
        state.query = query;
        state.selected = 0;
    }

    fn matches(&self) -> Vec<(&'static str, &'static str)> {
        let needle = self.query.trim().to_lowercase();
        EMOJIS.iter().copied().filter(|(emoji, names)| needle.is_empty() || emoji.contains(&needle) || names.contains(&needle)).collect()
    }

    pub fn move_by(delta: isize, count: usize) {
        if count == 0 { return; }
        let mut state = Self::write();
        let last = count.saturating_sub(1) as isize;
        state.selected = ((state.selected.min(last as usize) as isize) + delta).clamp(0, last) as usize;
    }

    pub fn reset() {
        TextInput::set_text(INPUT, "");
        let mut state = Self::write();
        state.query.clear();
        state.selected = 0;
    }

    pub fn open() {
        Self::reset();
        Overlay::open(Panel::Emoji);
    }

    pub fn copy_selected() {
        let picked = {
            let state = Self::read();
            state.matches().get(state.selected).map(|(emoji, _)| *emoji)
        };
        if let Some(emoji) = picked { copy(emoji); }
    }
}

fn copy(value: &str) {
    let value = value.to_string();
    thread::spawn(move || {
        if let Ok(mut child) = Command::new("wl-copy").arg("--").arg(value).spawn() {
            let _ = child.wait();
        }
    });
    Overlay::close();
}

pub fn view(monitor: &Monitor) -> LayerWindow {
    let theme = Theme::read();
    let scheme = theme.scheme();
    let open = Overlay::read().panel() == Panel::Emoji;
    let state = EmojiPicker::read();
    let matches = state.matches();
    let selected = state.selected.min(matches.len().saturating_sub(1));
    let count = matches.len();

    let search = overlay::search(
        "emoji", INPUT, m3::SEARCH,
        crate::lang::text("Buscar emoji…", "Search emoji…"),
        open, &scheme, EmojiPicker::set_query, |_| EmojiPicker::copy_selected(),
    );

    let rows: Vec<Box<dyn Widget>> = matches.chunks(COLUMNS).enumerate().map(|(row, group)| {
        boxed(Row::new(group.iter().enumerate().map(|(column, (emoji, _))| {
            let index = row * COLUMNS + column;
            let value = *emoji;
            let active = index == selected;
            boxed(
                Rectangle::new().width(62.0).height(54.0)
                    .fill(if active { scheme.secondary_container } else { scheme.surface_container_high })
                    .radius(14.0).cursor(Pointer)
                    .on_click(move |_| copy(value))
                    .align_child(Center, Center)
                    .child(m3::text(value, 27.0, if active { scheme.on_secondary_container } else { scheme.on_surface }, amane::Weight::Regular))
            )
        }).collect()).gap(6.0))
    }).collect();

    let grid: Box<dyn Widget> = if rows.is_empty() {
        boxed(m3::empty_state(m3::SEARCH, crate::lang::text("Nenhum emoji encontrado", "No emoji found"), crate::lang::text("Tente outra palavra", "Try another keyword"), Parent, 220.0, &scheme))
    } else {
        boxed(ScrollArea::new("emoji-results", Column::new(rows).gap(6.0).width(Parent)).width(Parent).height(Parent))
    };

    let footer = m3::hint_bar(
        &format!("{} {}", count, crate::lang::text("emojis · ↑↓ navega · Enter copia", "emojis · ↑↓ navigate · Enter copies")),
        crate::lang::text("Esc fecha", "Esc closes"), &scheme,
    );

    let content = Rectangle::new().width(Parent).height(Parent)
        .padding(Padding { top: 16.0, right: 16.0, bottom: 16.0, left: 16.0 })
        .child(Column::new(vec![boxed(search), grid, boxed(footer)]).gap(12.0).width(Parent).height(Parent));

    overlay::layer("emoji", "cds-emoji", &monitor.name, open, &scheme,
        overlay::Frame { width: WIDTH, height: HEIGHT, radius: m3::shape::XL, place: overlay::Place::Spot },
        content,
        move |key| match key {
            Key::Down | Key::Right => EmojiPicker::move_by(1, count),
            Key::Up | Key::Left => EmojiPicker::move_by(-1, count),
            Key::Enter => EmojiPicker::copy_selected(),
            _ => {},
        })
}
