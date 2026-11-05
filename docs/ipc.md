# IPC reference

Send commands through the Amane CLI:

```bash
amane ipc call <command> [arguments...]
```

## Panels

| Command | Action |
| --- | --- |
| open launcher | Open the application launcher |
| open apps | Open the application grid |
| open spotlight | Open search and quick actions |
| open clipboard | Open clipboard history |
| open power | Open the power menu |
| open control | Open the control center |\n| open color | Open the HEX color picker |\n| open emoji | Open the searchable emoji picker |
| open config | Open shell settings |
| closed | Close open panels |
| lock | Lock the current session |

## Dash

```bash
amane ipc call dash
amane ipc call dash home
amane ipc call dash media
amane ipc call dash weather
amane ipc call dash wallpapers
amane ipc call dash alerts
```

Supported tabs: home, media, weather, wallpapers and alerts. Older aliases such as clima, papeis and alertas are still accepted.

## Tray

```bash
amane ipc call tray toggle
amane ipc call tray close
amane ipc call tray list
amane ipc call tray focus <id>
amane ipc call tray close-window <id>
```

## Settings

```bash
amane ipc call settings wallpaper <image>
amane ipc call settings wallpaper
amane ipc call settings wallpapers
amane ipc call settings dark true
amane ipc call settings dark false
amane ipc call settings dark toggle
amane ipc call settings scheme <name>
amane ipc call settings directory <path>
amane ipc call settings location <latitude,longitude>
amane ipc call settings location auto
amane ipc call settings refresh
```

The Settings window also exposes appearance, wallpaper folder, weather coordinates, hover behavior and language.

Supported Matugen schemes: scheme-tonal-spot, scheme-content, scheme-expressive, scheme-neutral, scheme-vibrant, scheme-rainbow, scheme-fruit-salad and scheme-fidelity.

## Theme

```bash
amane ipc call theme set <image> [scheme] [dark|light]
amane ipc call theme color <hex> [scheme] [dark|light]
```

## Weather

```bash
amane ipc call weather
```

Weather data comes from Open-Meteo and is cached in ~/.cache/chloe-dynamic-shell/weather.json. The service refreshes in the background.

## Example

```bash
amane ipc call open launcher
amane ipc call settings wallpaper ~/Pictures/Wallpapers/example.png
amane ipc call settings location -20.44,-54.65
amane ipc call tray list
```
