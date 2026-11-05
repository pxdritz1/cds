# Architecture

## Overview

Chloe Dynamic Shell is a collection of Amane windows backed by shared services. The compositor manages the layer surfaces; Amane handles rendering, input and service updates.

```text
niri
 ├─ Island and Dash
 ├─ Launcher, Spotlight and application grid
 ├─ Clipboard and power menu
 ├─ Control center and Settings
 ├─ Lock screen
 └─ Wallpaper layer

Amane services
 ├─ Settings and Theme
 ├─ UI and Overlay state
 ├─ Weather, Clock and Clipboard
 ├─ Tray, Launcher and Spotlight
 └─ System services provided by Amane
```

## Main modules

| Module | Responsibility |
| --- | --- |
| main.rs | Register windows and IPC handlers |
| island.rs | Island surfaces, modes and transitions |
| island/compact.rs | Workspaces, clock, weather and system status |
| island/dash.rs | Dash tabs and cards |
| island/peek.rs | Media and notification previews |
| island/widgets.rs | Shared Island widgets |
| overlay.rs | Panel state, layer placement and outside-click handling |
| settings.rs | Persistent preferences and wallpaper selection |
| settingsview.rs | Settings window |
| lang.rs | Language-aware UI strings |
| theme.rs | Matugen, color tokens and font setup |
| motion.rs | Animation timing and state transitions |
| ipc.rs | CLI command handlers |
| control.rs | Audio, network and device controls |
| weather.rs | Open-Meteo data and forecast parsing |
| tray.rs | StatusNotifierWatcher and tray items |
| lockscreen.rs | Session lock screen and password input |

## Services and rendering

Modules that implement Amane's Service trait share a synchronized state. Views read that state while building widgets; actions update it through the service write guard.

Keep slow work out of the drawing path:

- File and network operations run in background workers.
- Service locks should not be held while running subprocesses.
- Only publish new service state when the data has changed.
- Coalesce repeated work when only the latest request matters.
- Use the shared motion scheduler for continuous animations.

## Shell layouts and motion

The shell has three runtime layouts, selected in Settings or with
`amane ipc call settings layout <1|2|3>`:

- Layout 1 keeps the original three-island bar.
- Layout 2 hides the bar and shows the standalone app dock.
- Layout 3 combines the dock with the existing bar and center Island.

The dock has independent top/bottom placement and left/center/right alignment.
The workspace group, center Island and status group can also be aligned
independently. Workspace, center-Island, status and dock visibility can be
changed at runtime.
The center clock can be digital or analog.

Overlay transitions use a restrained bubble burst behind the panel. The motion
language can be switched between Material 3 and Fluent timing presets. Animation
frames are requested through the shared frame-pulse service so only windows
subscribed to active motion are woken.

The focused-window service listens to Niri's event stream and queries the
focused-window response over the compositor's Unix IPC socket. It publishes
only changed title/app-id metadata, avoiding a polling subprocess.

## Overlays

Panels are separate layer windows created through overlay::layer. The helper handles monitor focus, keyboard exclusivity, backdrop, placement and visibility transitions.

The panel stays mapped until its closing animation finishes. Escape and outside clicks close the active panel.

## Tray

The tray service watches registered StatusNotifierItems over D-Bus and keeps a cached list for the Island. Item activation and context-menu support depend on the application exporting the corresponding interfaces.

## Persistence

| Path | Data |
| --- | --- |
| ~/.config/chloe-dynamic-shell/settings.conf | Appearance, wallpaper, location, hover and language settings |
| ~/.cache/chloe-dynamic-shell/colors.conf | Generated Matugen colors |
| ~/.cache/chloe-dynamic-shell/clipboard.conf | Clipboard history |
| ~/.cache/chloe-dynamic-shell/weather.json | Last weather response |

## Language

The selected language is stored in settings.conf. Brazilian Portuguese is the default for existing installations; English can be selected in Settings. UI strings use the shared language helper so the preference is applied consistently.
