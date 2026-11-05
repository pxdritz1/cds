# TODO

## Island and panels

- [x] Open the Dash on hover, with a setting to disable it.
- [x] Close the Dash on outside click without blocking underlying windows.
- [x] Move the tray to the right Island.
- [x] Dispatch StatusNotifierItem Activate and ContextMenu D-Bus methods asynchronously.
- [x] Read and render the com.canonical.dbusmenu Menu object when a tray item exposes one.
- [x] Show the focused window title and app icon beside the workspaces.
- [x] Add Spotlight and weather shortcuts to the center Island.

## Appearance

- [x] Run Matugen outside the drawing thread and serialize theme jobs.
- [x] Validate generated palettes before replacing active colors.
- [x] Accept home-relative and absolute wallpaper paths.
- [x] Add automatic wallpaper rotation and an interval setting.
- [x] Use consistent wallpaper path formatting across panels.

## Settings

- [x] Keep Settings in a dedicated window.
- [x] Add Brazilian Portuguese and English.
- [x] Add controls for Island layout and visibility.
- [x] Add independent left/right/center placement for the workspace group, center Island and status group.
- [x] Allow repositioning each status indicator independently.
- [x] Apply all appearance settings without restarting the shell.
- [x] Keep the Matugen regeneration hint and regenerate action pinned to the bottom of the Settings window.
- [x] Add a setting to switch CPU, memory and battery meters between the standard palette and transgender-flag colors.

## Dash

- [x] Replace animated usage meters with static colored bars; provide standard and transgender-flag palettes in Settings.
- [x] Open the weather tab when its card is clicked.
- [x] Keep the Dashboard clock style fixed and remove its analog/digital setting control.

## Spotlight and Launcher

- [x] Use desktop application icons with a Material Symbols fallback.
- [x] Improve icon fallback for apps without a GTK icon.
- [x] Make Enter activate the selected Spotlight result.
- [x] Add right-click pin/unpin support for apps in Spotlight and Launcher.
- [x] Add a full-screen app-grid launcher with an app-grid button in the shell.

## Utilities

- [x] Add a color picker with HEX input and copy.
- [x] Add an emoji picker with search and copy.

## Notifications

- [x] Show the source application's icon in notification previews.
- [x] Use conversation avatars when notification metadata provides them.
- [x] Define a fallback order for notification artwork.

## Validation

- [ ] Test startup and repeated wallpaper changes.
- [ ] Test tray behavior with multiple SNI applications.
- [ ] Test notifications and multi-monitor sessions.
- [ ] Run a full build and runtime check on supported systems.
