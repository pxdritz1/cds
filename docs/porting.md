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
| XDG paths | `~/.config/cds`, `~/.cache/cds`, `~/.local/share/cds` | Implemented with non-destructive import from the old `chloe-dynamic-shell` directories |
| Niri workspaces and focused window | `niri-ipc` event stream | Native event stream, workspace focus and focused title implemented; live compositor parity pending |

## Native crate capabilities

| Capability | Crate | Planned use |
| --- | --- | --- |
| Iced widget extensions | `iced_aw` 0.14.1 | Color picker, context menus, tabs, menus and utility widgets |
| Fuzzy search | `nucleo` 0.5 | Async search across desktop apps, actions, clipboard and emoji |
| Emoji catalog | `emojis` 0.9 | Unicode 18 metadata, groups, shortcodes and skin-tone variants |
| SVG artwork | `resvg` 0.49 | Render Noto emoji, app artwork and SVG assets to raster surfaces |
| Image resizing | `fast_image_resize` 6.1 | SIMD thumbnails, cover art and wallpaper previews |
| Image decoding | `image` 0.25 | PNG, JPEG, WebP, GIF, BMP and ICO assets |
| Image cache | `lru` 0.18 | Bounded thumbnail and artwork cache |
| Material color | `material-color-utils` 0.1.3 | HCT, dynamic M3 schemes and image-derived colors |

## Shell and panels

| Existing behavior | Native target | Status |
| --- | --- | --- |
| Three-part bar: workspaces, center Island, status | `ui/island` | Pending |
| Compact notch and clock | Island Compact mode | Pending |
| Focused workspace/window title and icon | Niri IPC + desktop icon lookup | Pending |
| Dynamic notification island | Notice mode with content-sized layout | Pending |
| Media Island | MPRIS, artwork, timeline and controls | Pending |
| Dash Home/Media/Weather/Wallpapers/Alerts | Dash tabs | Initial tab state scaffold; content pending |
| Launcher | Desktop entries, Nucleo fuzzy search, app launch and keyboard navigation | Native search and launch UI implemented; keyboard selection, pinning and visual parity pending |
| Spotlight | Shared Nucleo app search plus quick actions | Native app search implemented; quick actions and visual parity pending |
| Full-screen app grid | Desktop entries, icon theme and six-column grid | Native scan, icon lookup, grid and launch implemented; pinning and visual parity pending |
| Clipboard history | `wl-paste --watch`, persistent 80-item text history, deduplication, search and copy | Native watcher, sensitive-selection filtering, legacy-format migration and keyboard navigation implemented; live Wayland validation pending |
| Power menu | Session/system actions | Pending |
| Control Center | Audio, microphone, Wi-Fi, Bluetooth and brightness | Pending |
| Settings window | Persistent settings and live updates | Pending |
| Color picker | `iced_aw` color wheel, HEX input/copy and hyprpicker eyedropper | Native picker and eyedropper implemented; visual parity pending |
| Emoji picker | Unicode 18 catalog, Nucleo fuzzy search, pt-BR aliases, skin-tone metadata and copy | Native searchable grid and copy implemented; Noto SVG artwork and skin-tone UI pending |
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
| Matugen schemes and colors | Material Color Utilities HCT palette generation + Matugen export | Native light/dark scheme generation from HEX or image implemented; Matugen export and live UI application pending |
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
