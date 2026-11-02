mod art;
mod clock;
mod island;
mod motion;
mod settings;
mod theme;
mod ui;
mod wallpaper;

use amane::App;

fn main() {
    App::new()
        .window_per_monitor(wallpaper::view)
        .window_per_monitor(island::reserve)
        .window_per_monitor(island::view)
        .ipc("theme", theme::command)
        .ipc("island", ui::command)
        .run();
}
