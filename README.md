# Chloe Dynamic Shell

A Wayland desktop shell written in Rust with Amane and a Material 3-inspired interface.

## Features

- Island with workspace indicators, focused-window title and app icon, digital/analog clock, weather shortcut, media controls and tray.
- Dash with system overview, media, weather, wallpapers and notifications.
- Spotlight and application launcher.
- Control center for Wi-Fi, Bluetooth, audio, microphone and brightness.
- Session lock screen using ext-session-lock and PAM.
- Searchable clipboard history, emoji picker and HEX color picker.
- Matugen themes, wallpaper settings and optional timed wallpaper rotation.
- M3 or Fluent motion timing with animated bubble transitions between panels.
- Brazilian Portuguese and English, selectable in Settings.

## Requirements

The shell targets a Wayland compositor with layer-shell support. It integrates with Niri, Matugen, wl-clipboard, PulseAudio-compatible `pactl`, PAM and `hyprpicker` for the screen eyedropper. Weather lookup and the bundled emoji-asset installer use `curl`.

### Arch Linux / CachyOS

Install the runtime dependencies:

```bash
sudo pacman -S --needed matugen wl-clipboard libpulse pam curl python
```

Install Niri if you want to use the included Niri configuration, and install `hyprpicker` to enable the screen eyedropper:

```bash
sudo pacman -S --needed niri hyprpicker
```

The emoji selector uses Google's official Noto Emoji SVG artwork, not the system emoji font. Download the assets once after cloning the repository:

```bash
bash scripts/install-emoji-assets.sh
```

This places the SVGs in `~/.local/share/chloe-dynamic-shell/emoji`. The original Unicode characters are still copied to the clipboard. The artwork is from the Noto Emoji project; see its included license and [upstream repository](https://github.com/googlefonts/noto-emoji).

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

See the [IPC reference](docs/ipc.md) and [niri keybindings](docs/binds.md). The ported personal Niri configuration is in `docs/niri-config.kdl`; install it with `bash scripts/install-niri-config.sh` (the script creates a timestamped backup first).

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
