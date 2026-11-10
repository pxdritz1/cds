# Chloe Dynamic Shell

A native Rust desktop shell for Wayland and Niri, with a Material 3-inspired interface.

> **Native port in progress.** The Amane implementation remains on its original branch. This branch replaces the runtime with Iced and a native Wayland shell backend. Feature parity is tracked in [docs/porting.md](docs/porting.md); do not treat the port as complete until runtime and feature checks pass.

## Runtime stack

- **Iced 0.14** for UI, rendering and application state.
- **iced_exwlshell 0.21.4** for Wayland layer-shell and ext-session-lock.
- **zbus** for D-Bus integrations: MPRIS, StatusNotifier and notifications.
- **niri-ipc** for Niri workspace and focused-window state.
- **freedesktop-desktop-entry** and **freedesktop-icons** for desktop applications and icons.
- `iced_aw` for the color picker, context menus, tabs and other Iced widgets.
- `nucleo` for fast, asynchronous fuzzy search in Launcher, Spotlight and emoji search.
- `emojis` for current Unicode emoji metadata, groups and skin-tone variants.
- `resvg` and `fast_image_resize` for SVG artwork and SIMD-accelerated image resizing; `lru` for bounded image/artwork caches.
- `material-color-utils` for Material 3 HCT, dynamic schemes and wallpaper-derived palette extraction.
- Tokio, reqwest, notify, serde and supporting crates for asynchronous services and persistence.

The shell targets Wayland compositors with wlr-layer-shell; Niri is the primary target. The lock screen requires compositor support for ext-session-lock and a working PAM configuration.

## Build

```bash
cargo build --release --bin cds
```

Install the executable:

```bash
cargo install --path . --bin cds --force
```

Or use the installer:

```bash
bash scripts/install.sh
```

The installer does not replace the user's Niri configuration. To install the example configuration explicitly:

```bash
bash scripts/install.sh --install-niri-config
```

## Commands

```bash
cds run                 # start the shell if it is not running
cds daemon              # start the shell in the foreground
cds dev                 # cargo run from a source checkout
cds compile             # optimized build from a source checkout

cds open launcher
cds open apps
cds open spotlight
cds open clipboard
cds open power
cds open control
cds open config
cds open color
cds open emoji

cds dash [home|media|weather|wallpapers|alerts]
cds close
cds lock
```

The native IPC already exposes `cds tray list`, `cds settings get`, `cds settings set <language|dark|motion> <value>`, `cds settings flag <name> <true|false>`, and clipboard list/clear. Tray menu actions and theme commands are being refined; `cds weather` remains pending. See [docs/ipc.md](docs/ipc.md) and the parity checklist in [docs/porting.md](docs/porting.md).

## Configuration and runtime paths

- Settings: `~/.config/cds/`
- Cache: `~/.cache/cds/`
- Persistent data: `~/.local/share/cds/`
- IPC socket: `$XDG_RUNTIME_DIR/cds.sock`

The executable and all shell-facing commands are named `cds`. No Amane CLI or Amane-generated project is used by this branch.

## Existing design and features

The migration target includes the existing Dynamic Island (workspaces, center notch and status group), Compact/Media/Notice/Dash modes, Home/Media/Weather/Wallpapers/Alerts tabs, launcher, Spotlight, application grid, clipboard history, StatusNotifier tray and dbusmenu, MPRIS, Control Center, Wi-Fi/Bluetooth/audio/microphone/brightness controls, notifications, session lock, settings, Matugen, wallpaper rotation, weather, OSD, power menu, emoji picker, HEX color picker, pt-BR/English strings, Material 3 components and M3/Fluent motion.

The source-to-native mapping and validation state live in [docs/porting.md](docs/porting.md).

## License

Apache-2.0. See [LICENSE](LICENSE).
