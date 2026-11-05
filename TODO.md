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
- [x] Add independent left/right/center placement for the workspace group, center Island, status group and dock.
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

## UI modes

- [x] Mode 1: Islands, preserving the existing Island shell.
- [x] Mode 2: standalone macOS-like application Dock; remove the combined Dock + bar mode.
- [x] Add IPC commands to query, select and toggle Islands / Dock.
- [x] Add a Niri keybind for animated mode switching.
- [x] Coordinate Island contraction and Dock expansion through the shared motion frame scheduler.
- [x] Open the application grid from the Dock.
- [x] Focus an existing Niri window when its running application icon is clicked.
- [x] Keep pinned apps, running-app indicators, magnification and trash drag-and-drop.
## K4-style Dock refinement

- [x] Replace the static capsule with a fixed-surface, animated K4-inspired path silhouette.
- [x] Use pointer-distance Gaussian magnification instead of a binary hovered-icon scale.
- [x] Keep enlarged icons, app labels and window-count badges inside reserved Dock headroom.
- [x] Separate pinned applications from automatically discovered running applications.
- [x] Show running indicators and multi-window counts.
- [x] Animate the icon when an application is launched or focused.
- [x] Allow scrolling over an app icon to cycle through its open Niri windows.
- [x] Add an in-Dock window picker with per-window titles.
- [ ] Add live window previews and drag-to-reorder / drag-out-to-unpin interactions.
- [x] Add configurable auto-hide with a four-pixel edge reveal target.
- [x] Add a searchable application drawer attached to the Dock surface.
- [ ] Animate the drawer as a morph of the Dock silhouette.
- [ ] Reproduce the K4 drop/impact transition and the coordinated split/fall Island transition as dedicated animation layers.

## Notifications

- [x] Show the source application's icon in notification previews.
- [x] Use conversation avatars when notification metadata provides them.
- [x] Define a fallback order for notification artwork.

## Validation

- [ ] Test startup and repeated wallpaper changes.
- [ ] Test tray behavior with multiple SNI applications.
- [ ] Test notifications and multi-monitor sessions.
- [ ] Run a full build and runtime check on supported systems.
