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

Install an AUR helper (`yay` or `paru`), then run the installer from the repository root:

```bash
bash scripts/install.sh
```

The installer installs Amane's system dependencies, Rust, `niri-git`, `matugen-bin`, Qt color-control tools and the CDS runtime utilities. It creates `~/.config/amane`, copies the CDS sources, compiles the shell, and backs up the existing Niri and Matugen configuration before applying changes.

It also installs and registers Matugen templates for:

- **Zed:** dark and light themes in `~/.config/zed/themes`. Select Chloe Dynamic Shell in the theme selector.
- **Prism Launcher:** widget theme in the Prism Launcher themes directory. Select Chloe Dynamic Shell in Settings → Appearance → Widgets.
- **Zen Browser:** imports the generated stylesheet into detected profiles and enables the user stylesheet preference.
- **GTK 3 and GTK 4:** generates `colors.css` and imports it from each GTK `gtk.css`.
- **Qt 5 and Qt 6:** generates palettes for qt5ct and qt6ct and configures their color scheme paths. Set `QT_QPA_PLATFORMTHEME=qt5ct` or `qt6ct` for applications using the corresponding Qt version.

The installer comments out legacy Niri, SCSS and Alacritty Matugen template sections rather than deleting them. To restore one, remove the leading `#` from its section in `~/.config/matugen/config.toml`.

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
