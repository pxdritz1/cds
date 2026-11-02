mod art;
mod clipboard;
mod clock;
mod island;
mod launcher;
mod motion;
mod overlay;
mod power;
mod settings;
mod theme;
mod ui;
mod wallpaper;

use amane::App;

fn main() {
    App::new()
        .font("Google Sans Flex")
        .window_per_monitor(wallpaper::view)
        .window_per_monitor(island::reserve)
        .window_per_monitor(island::view)
        .window_per_monitor(launcher::view)
        .window_per_monitor(clipboard::view)
        .window_per_monitor(power::view)
        .ipc("theme", theme::command)
        .ipc("island", ui::command)
        .ipc("settings", settings::command)
        .ipc("overlay", overlay::command)
        .run();
}
