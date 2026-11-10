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

## Native service commands

These commands are currently wired to the native service layer:

```bash
cds tray list

cds settings get
cds settings set language pt-BR
cds settings set language en
cds settings set dark true
cds settings set dark false
cds settings set motion m3
cds settings set motion fluent
cds settings flag <name> true
cds settings flag <name> false

cds theme color <hex>
cds theme set <image>

cds clipboard list
cds clipboard clear
```

The tray panel also supports left-click activation and right-click dbusmenu requests. The native settings service imports the original `settings.conf` format and writes changes to `~/.config/cds/settings.conf`. Weather remains pending; commands for wallpaper rotation, automatic location and tray focus/close are not implemented yet.


