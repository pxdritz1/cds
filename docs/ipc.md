# CDS IPC reference

The shell exposes a local Unix socket at `$XDG_RUNTIME_DIR/cds.sock`. Use the `cds` executable; the CLI sends structured requests to the running shell.

## Panels

```bash
cds open launcher
cds open apps
cds open spotlight
cds open clipboard
cds open power
cds open control
cds open config
cds open color
cds open emoji
cds close
cds lock
```

Panel aliases retained from the previous implementation include `menu`, `grade`, `clip`, `settings`, `color-picker` and `emoji-picker`.

## Dashboard

```bash
cds dash
cds dash home
cds dash media
cds dash weather
cds dash wallpapers
cds dash alerts
```

Aliases `clima`, `papeis` and `alertas` remain accepted.

## Service command surface

These command groups are part of the parity target and will be enabled as their native services land:

```bash
cds tray toggle
cds tray close
cds tray list
cds tray focus <id>
cds tray close-window <id>

cds settings wallpaper <image>
cds settings wallpaper
cds settings wallpapers
cds settings dark toggle
cds settings scheme <name>
cds settings directory <path>
cds settings location <latitude,longitude>
cds settings location auto
cds settings motion m3
cds settings motion fluent
cds settings clock analog
cds settings clock digital
cds settings refresh

cds theme set <image> [scheme] [dark|light]
cds theme color <hex> [scheme] [dark|light]
cds weather
```

**Current foundation status:** the native IPC transport and initial open/dash/close/lock routes are committed. Tray, settings, theme and weather currently return an explicit not-yet-ported response; they are not functional yet.
