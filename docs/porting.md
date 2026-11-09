# Native port checklist

This is the parity contract for the Iced rewrite. The old implementation is the behavioral reference. A feature is complete only when its UI, input behavior, service integration, persistence and IPC behavior have been checked.

## Runtime and infrastructure

| Existing behavior | Native target | Status |
| --- | --- | --- |
| Amane multi-window runtime | Iced daemon + iced_exwlshell | Foundation committed; runtime validation pending |
| Per-monitor surfaces | Iced window registry + Wayland output events | Foundation committed; validation pending |
| Layer-shell bar and overlays | wlr-layer-shell via iced_exwlshell | Foundation committed; visual parity pending |
| ext-session-lock | iced_exwlshell session-lock actions | Crate selected; implementation pending |
| IPC commands | Private Unix socket + `cds` CLI | Initial open/dash/close/lock route committed |
| XDG paths | chloe-dynamic-shell config/cache/data dirs | Initial path helper committed |
| Niri workspaces and focused window | `niri-ipc` event stream | Dependency selected; implementation pending |

## Shell and panels

| Existing behavior | Native target | Status |
| --- | --- | --- |
| Three-part bar: workspaces, center Island, status | `ui/island` | Pending |
| Compact notch and clock | Island Compact mode | Pending |
| Focused workspace/window title and icon | Niri IPC + desktop icon lookup | Pending |
| Dynamic notification island | Notice mode with content-sized layout | Pending |
| Media Island | MPRIS, artwork, timeline and controls | Pending |
| Dash Home/Media/Weather/Wallpapers/Alerts | Dash tabs | Initial tab state scaffold; content pending |
| Launcher | Desktop entries, search, keyboard navigation and pinning | Pending |
| Spotlight | Search and quick actions | Pending |
| Full-screen app grid | Desktop entries and icon theme | Pending |
| Clipboard history | Wayland clipboard watcher, persistence and search | Pending |
| Power menu | Session/system actions | Pending |
| Control Center | Audio, microphone, Wi-Fi, Bluetooth and brightness | Pending |
| Settings window | Persistent settings and live updates | Pending |
| Color picker | HEX input, copy and hyprpicker integration | Pending |
| Emoji picker | Search, selection and copy | Pending |
| OSD | Volume/brightness feedback | Pending |
| Wallpaper layer and rotation | Matugen + wallpaper service | Pending |
| Notifications and previews | D-Bus notification service and notification Island | Pending |
| Tray and dbusmenu | StatusNotifierWatcher, item activation and context menus | Pending |
| Lock screen | ext-session-lock + PAM authentication | Pending |

## Services and visual system

| Existing behavior | Native target | Status |
| --- | --- | --- |
| MPRIS playback | zbus MPRIS proxies | Pending |
| StatusNotifier tray | zbus watcher and item proxies | Pending |
| Audio and microphone | PipeWire/PulseAudio controls | Pending |
| Wi-Fi | NetworkManager D-Bus | Pending |
| Bluetooth | BlueZ D-Bus | Pending |
| Weather | Open-Meteo and cached forecast | Pending |
| Matugen schemes and colors | Matugen process + validated palette parser | Pending |
| Wallpaper selection/rotation | Persistent wallpaper service | Pending |
| Material 3 components | Native Iced widget styles | Pending |
| M3/Fluent motion | Shared animation scheduler | Pending |
| Google Sans Flex and Material Symbols | Bundled/system font loading | Pending |
| Brazilian Portuguese and English | Shared language catalog | Pending |
| Trans-flag meter palette option | Settings + meter palette | Pending |

## Validation gates

- [ ] `cargo check --bin cds` succeeds on CachyOS.
- [ ] `cargo build --release --bin cds` succeeds.
- [ ] Shell starts on Niri and creates one bar surface per output.
- [ ] Hot-plugging and removing monitors updates surfaces without stale IDs.
- [ ] All panels open, close on Escape/outside click and preserve keyboard focus.
- [ ] Existing Niri keybindings work with `cds` commands.
- [ ] Tray, dbusmenu, MPRIS and notifications tested with multiple real applications.
- [ ] Lock/unlock tested with PAM and Niri session-lock support.
- [ ] Wallpaper changes and Matugen palette updates tested repeatedly.
- [ ] No Amane dependency, executable, generated manifest or runtime path remains in the native build.
