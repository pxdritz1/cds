# TODO

## Island and panels

- [x] Open the Dash on hover, with a setting to disable it.
- [x] Close the Dash on outside click without blocking underlying windows.
- [x] Move the tray to the right Island.
- [ ] Complete StatusNotifierItem support, including D-Bus context menus for tray apps.
- [ ] Show the focused window title and app icon beside the workspaces.
- [x] Add Spotlight and weather shortcuts to the center Island.

## Appearance

- [x] Run Matugen outside the drawing thread and serialize theme jobs.
- [x] Validate generated palettes before replacing active colors.
- [x] Accept home-relative and absolute wallpaper paths.
- [ ] Add automatic wallpaper rotation and an interval setting.
- [ ] Use consistent wallpaper path formatting across panels.

## Settings

- [x] Keep Settings in a dedicated window.
- [x] Add Brazilian Portuguese and English.
- [ ] Add controls for Island layout and visibility.
- [ ] Allow repositioning shell elements (left/right/center, including dock and individual indicators).
- [ ] Apply all appearance settings without restarting the shell.
- [ ] Move the Matugen regeneration hint to the bottom of the Settings window.
- [ ] Add a setting to switch CPU, memory and battery meters between the standard palette and transgender-flag colors.

## Dash

- [x] Replace animated usage meters with static colored bars; provide standard and transgender-flag palettes in Settings.
- [x] Open the weather tab when its card is clicked.
- [ ] Allow switching the clock card between digital and analog.

## Spotlight and Launcher

- [x] Use desktop application icons with a Material Symbols fallback.
- [ ] Improve icon fallback for apps without a GTK icon.
- [x] Make Enter activate the selected Spotlight result.
- [ ] Add pin/unpin support for objects in Spotlight and Launcher.
- [ ] Add a full-screen app-grid launcher with an app-grid button in the shell.

## Utilities

- [ ] Add a color picker with HEX input and copy.
- [ ] Add an emoji picker with search and copy.

## UI modes

- [ ] Mode 1: current shell layout (Islands and one bar).
- [ ] Mode 2: standalone macOS-like translucent app dock, with magnification on hover, optional trash target supporting drag-and-drop, and an app-grid button.
- [ ] Mode 3: dock plus bar, centered main Island combining existing shell controls and additional content.
- [ ] Add per-element position controls so users can place the dock, main Island and individual indicators on different screen edges/sides.

## Notifications

- [ ] Show the source application's icon in notification previews.
- [ ] Use conversation avatars when notification metadata provides them.
- [ ] Define a fallback order for notification artwork.

## Validation

- [ ] Test startup and repeated wallpaper changes.
- [ ] Test tray behavior with multiple SNI applications.
- [ ] Test notifications and multi-monitor sessions.
- [ ] Run a full build and runtime check on supported systems.
