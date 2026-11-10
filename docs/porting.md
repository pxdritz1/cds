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
| Three-part bar: workspaces, center Island, status | Native Iced bar | Three-group bar and live center content implemented; per-monitor visual parity and M3 geometry pending |
| Compact notch and clock | Native center layer | Compact center pill, focused title and clock implemented; exact notch silhouette and side islands pending |
| Focused workspace/window title and icon | Niri IPC + desktop icon lookup | Workspace focus and focused title implemented; focused-app icon and live Niri validation pending |
| Dynamic notification island | Freedesktop Notifications + content-sized center layer | Notification title/app icon and width adaptation implemented; hover expansion, per-notification timing and full preview pending |
| Media Island | MPRIS, artwork cache, timeline and controls | Player discovery, artwork caching, playback controls and timeline readout implemented; seeking, multi-player selection and exact animation parity pending |
| Dash Home/Media/Weather/Wallpapers/Alerts | Dash tabs | Media and Alerts have native content; Home, Weather and Wallpapers remain pending |
| Launcher | Original CDS vertical launcher, not Suzuha | Original 640×460 list layout, 36px icons, name/description search, 40 visible rows, pinned-first sorting, left-click launch, right-click pin, ↑↓/Tab/Enter and Esc implemented; live visual/runtime parity pending |
| Spotlight | Shared Nucleo app search plus quick actions | Native app search implemented; quick actions and visual parity pending |
| Full-screen app grid | Desktop entries, icon theme and six-column grid | Native scan, icon lookup, grid and launch implemented; pinning and visual parity pending |
| Clipboard history | `wl-paste --watch`, persistent 80-item text history, deduplication, search and copy | Native watcher, sensitive-selection filtering, legacy-format migration and keyboard navigation implemented; live Wayland validation pending |
| Power menu | Session/system actions | Pending |
| Control Center | pactl, nmcli, rfkill and brightnessctl | Output/input mute, volume buttons and slider, Wi-Fi/Bluetooth toggles and brightness controls implemented; device lists, detail pages and live hardware validation pending |
| Settings window | Native persistent settings + legacy `settings.conf` importer | Legacy language/theme/motion/alignment/flags/pinned-app preferences load and save; basic theme/language/motion and shell toggles exposed; remaining settings UI pending |
| Color picker | `iced_aw` color wheel, HEX input/copy and hyprpicker eyedropper | Native picker and eyedropper implemented; visual parity pending |
| Emoji picker | Unicode 18 catalog, Nucleo fuzzy search, pt-BR aliases, skin-tone metadata and copy | Native searchable grid, copy and Noto SVG artwork implemented; skin-tone UI pending |
| OSD | Bottom layer-shell overlay | Volume, microphone and brightness changes produce a timed bottom OSD; live key/hardware validation pending |
| Wallpaper layer and rotation | Matugen + wallpaper service | Pending |
| Notifications and previews | `org.freedesktop.Notifications` server | Notify, replacement IDs, timeout handling, close method/signal, Alerts list and content-sized island implemented; actions, hover timing and real-app validation pending |
| Tray and dbusmenu | Freedesktop + KDE StatusNotifierWatcher, SNI proxies and `com.canonical.dbusmenu` | Watcher names, registration signals, host registration, item metadata/icons, activation, GetLayout parsing, nested menu rendering and Event activation implemented; submenu expansion and live app validation pending |
| Lock screen | ext-session-lock + PAM authentication | Pending |

## Services and visual system

| Existing behavior | Native target | Status |
| --- | --- | --- |
| MPRIS playback | zbus MPRIS proxies | Active player discovery, metadata, artwork cache, playback status, timeline readout and previous/play-pause/next controls implemented; seeking and player selection pending |
| StatusNotifier tray | zbus watcher, item proxies and dbusmenu | Registration, item properties, activation, context-menu layout and menu-item activation implemented; submenu behavior and live validation pending |
| Audio and microphone | pactl / PipeWire-PulseAudio | Volume, mute, microphone mute, slider and OSD implemented; device selection and live validation pending |
| Wi-Fi | NetworkManager CLI via `nmcli` | Radio toggle implemented; scan, connection selection and NetworkManager D-Bus integration pending |
| Bluetooth | `rfkill` | Radio block/unblock implemented; device discovery/pairing and BlueZ D-Bus integration pending |
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
