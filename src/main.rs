/*
 * the lancamentos: windows registered per monitor, then one ipc entry
 * point with short names.
 */
mod apps;
mod art;
mod clipboard;
mod clock;
mod control;
mod color;
mod emoji;
mod glyphs;
mod ipc;
mod island;
mod launcher;
mod lang;
mod lockscreen;
mod m3;
mod motion;
mod osd;
mod overlay;
mod power;
mod settings;
mod settingsview;
mod spotlight;
mod theme;
mod tray;
mod ui;
mod wallpaper;
mod weather;

use amane::App;

fn main() {
    App::new()
        .font("Google Sans Flex")
        .window_per_monitor(wallpaper::view)
        .normal_window("cds-settings", settingsview::window)
        .window_per_monitor(island::reserve)
        .window_per_monitor(island::left)
        .window_per_monitor(island::view)
        .window_per_monitor(island::right)
        .window_per_monitor(apps::view)
        .window_per_monitor(spotlight::view)
        .window_per_monitor(launcher::view)
        .window_per_monitor(clipboard::view)
        .window_per_monitor(power::view)
        .window_per_monitor(control::view)
        .window_per_monitor(color::view)
        .window_per_monitor(emoji::view)
        .window_per_monitor(osd::view)
        .lock(lockscreen::view)
        .ipc("open", ipc::open)
        .ipc("closed", ipc::closed)
        .ipc("dash", ipc::dash)
        .ipc("tray", tray::command)
        .ipc("settings", ipc::settings)
        .ipc("weather", weather::command)
        .ipc("theme", theme::command)
        .ipc("lock", ipc::open_lock)
        .ipc("island", ipc::island_legacy)
        .ipc("overlay", ipc::overlay_legacy)
        .run();
}
