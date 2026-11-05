# Niri keybindings

The shell does not register global shortcuts. Add the commands below to
`~/.config/niri/config.kdl`. The complete example matching the current
personal setup is in [niri-config.kdl](niri-config.kdl).

## CDS panels

| Shortcut | Action |
| --- | --- |
| Mod+D | Application launcher |
| Mod+Space / XF86LaunchB | Application launcher |\n| Mod+Shift+Space | Spotlight |
| Mod+period | Emoji picker |
| Mod+P | HEX color picker |
| Mod+Shift+V | Clipboard history |
| Mod+Comma | Settings |
| Mod+Shift+E | Power menu |
| Mod+Shift+D | Control center |
| Mod+Shift+W | Dash, Wallpapers tab |
| Mod+Shift+L | Lock screen |
| Escape | Close the active panel |

Example:

```kdl
Mod+D           { spawn-sh "amane ipc call open launcher"; }
Mod+Space       { spawn-sh "amane ipc call open launcher"; }\nMod+Shift+Space { spawn-sh "amane ipc call open spotlight"; }
Mod+period      { spawn-sh "amane ipc call open emoji"; }
Mod+P           { spawn-sh "amane ipc call open color"; }
Mod+Shift+V     { spawn-sh "amane ipc call open clipboard"; }
Mod+Comma       { spawn-sh "amane ipc call open config"; }
Mod+Shift+E     { spawn-sh "amane ipc call open power"; }
Mod+Shift+D     { spawn-sh "amane ipc call open control"; }
Mod+Shift+W     { spawn-sh "amane ipc call dash wallpapers"; }
Mod+Shift+L     { spawn-sh "amane ipc call lock"; }
Escape          { spawn-sh "amane ipc call closed"; }
```

## Keyboard navigation

| Panel | Keys |
| --- | --- |
| Launcher | Up/Down to select, Enter to launch, Esc to close |
| Spotlight | Up/Down to select, Enter to run, Esc to close |
| Application grid | Click to launch, Esc to close |
| Emoji picker | Search, arrow keys to select, Enter or click to copy |
| Color picker | Enter or click to copy the valid `#RRGGBB` value |
| Clipboard | Up/Down to select, Enter to copy, Esc to close |
| Power menu | Up/Down to select, Enter to run, Esc to close |
| Settings | Enter applies the focused field, Esc closes |
| Control center | Click toggles controls; sliders can be dragged |
| Dash | Esc closes |
| Tray | Esc closes when focused; clicking outside closes its menu |

## Mouse

| Element | Left click | Right click |
| --- | --- | --- |
| Tray item | Activate | Open its context menu |
| Launcher result | Launch the app | Pin/unpin |
| Spotlight app result | Launch the app | Pin/unpin |
| Emoji | Copy emoji | — |
| Color copy action | Copy HEX | — |
| Clipboard entry | Copy the item | — |
| Island tray chevron | Toggle the tray | — |
| Outside an open panel | Close the panel | — |

## IPC

See [ipc.md](ipc.md) for the complete command reference.