# Niri keybindings

CDS does not register global shortcuts. Add the commands below to `~/.config/niri/config.kdl`. The full personal example is in [niri-config.kdl](niri-config.kdl).

| Shortcut | Action |
| --- | --- |
| Mod+D / Mod+Space / XF86LaunchB | Application launcher |
| Mod+Shift+Space | Spotlight |
| Mod+period | Emoji picker |
| Mod+P | HEX color picker |
| Mod+Shift+V | Clipboard history |
| Mod+Comma | Settings |
| Mod+Shift+E | Power menu |
| Mod+Shift+C | Control center |
| Mod+Shift+W | Dash, Wallpapers tab |
| Mod+Shift+L | Lock screen |
| Escape | Close the active panel |

Example:

```kdl
Mod+D           { spawn-sh "cds open launcher"; }
Mod+Space       { spawn-sh "cds open launcher"; }
Mod+Shift+Space { spawn-sh "cds open spotlight"; }
XF86LaunchB     { spawn-sh "cds open launcher"; }
Mod+period      { spawn-sh "cds open emoji"; }
Mod+P           { spawn-sh "cds open color"; }
Mod+Shift+V     { spawn-sh "cds open clipboard"; }
Mod+Comma       { spawn-sh "cds open config"; }
Mod+Shift+E     { spawn-sh "cds open power"; }
Mod+Shift+C     { spawn-sh "cds open control"; }
Mod+Shift+W     { spawn-sh "cds dash wallpapers"; }
Mod+Shift+L     { spawn-sh "cds lock"; }
Escape          { spawn-sh "cds close"; }
```

## Keyboard and pointer behavior to preserve

- Launcher and Spotlight: Up/Down selection, Enter activation, Escape close.
- App grid: click to launch, Escape close.
- Emoji: search, arrow selection, Enter/click to copy.
- Color picker: valid HEX input and copy.
- Clipboard: Up/Down selection, Enter to copy, Escape close.
- Power menu: keyboard selection and activation.
- Tray: left-click activate, right-click context menu.
- Launcher/Spotlight results: right-click pin/unpin.
- Outside click: close the active panel.
