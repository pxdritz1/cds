# Chloe Dynamic Shell

A Wayland desktop shell written in Rust with Amane and a Material 3-inspired interface.

## Features

- Island with workspace indicators, clock, weather shortcut, media controls and tray.
- Dash with system overview, media, weather, wallpapers and notifications.
- Spotlight and application launcher.
- Control center for Wi-Fi, Bluetooth, audio, microphone and brightness.
- Session lock screen using ext-session-lock and PAM.
- Searchable clipboard history.
- Matugen themes and wallpaper settings.
- Brazilian Portuguese and English, selectable in Settings.

## Requirements

The shell targets a Wayland session. Runtime integrations include niri, Matugen, wl-clipboard, pactl and PAM. Some features depend on the compositor and services available in the session.

## Run

```bash
amane dev       # build and reload during development
amane compile   # build the optimized binary
amane run       # launch the compiled shell
```

## IPC

```bash
amane ipc call open launcher
amane ipc call open spotlight
amane ipc call open control
amane ipc call dash weather
amane ipc call settings dark toggle
amane ipc call lock
amane ipc call closed
```

See the [IPC reference](docs/ipc.md) and [niri keybindings](docs/binds.md).

## Project layout

```text
src/
  apps.rs             Application grid
  clipboard.rs        Clipboard history
  clock.rs            Clock and date
  control.rs          Control center
  ipc.rs              CLI command handlers
  island.rs           Island windows and state
  launcher.rs         Application launcher
  lang.rs             UI language helper
  lockscreen.rs       Session lock screen
  m3.rs               Material 3 components
  motion.rs            Animation helpers
  overlay.rs          Shared overlay handling
  settings.rs         Persistent preferences
  settingsview.rs     Settings window
  spotlight.rs        Search and quick actions
  theme.rs            Matugen and color tokens
  tray.rs              Status notifier support
  weather.rs           Weather data
  island/              Island and Dash components
docs/
  architecture.md      Internal design
  binds.md              Example niri shortcuts
  ipc.md                IPC reference
```

## Fonts

The interface uses Google Sans Flex and Material Symbols Rounded. To generate static Google Sans Flex instances:

```bash
python -m pip install fonttools
python scripts/google-sans-static.py GoogleSansFlex.ttf ~/.local/share/fonts/GoogleSansFlex
fc-cache -f
```

## Configuration

Settings are stored in ~/.config/chloe-dynamic-shell/settings.conf. Runtime caches live in ~/.cache/chloe-dynamic-shell/.

Matugen combines the user's configuration with the shell template when generating colors.

## License

Apache-2.0. See LICENSE.
