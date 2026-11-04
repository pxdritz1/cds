# niri keybindings

The shell does not register shortcuts. Add the commands you want to use to the binds section of ~/.config/niri/config.kdl.

## Suggested shortcuts

```kdl
Mod+Space       { spawn "sh" "-c" "amane ipc call dash"; }
Mod+O           { spawn "sh" "-c" "amane ipc call dash home"; }
Mod+M           { spawn "sh" "-c" "amane ipc call dash media"; }
Mod+W           { spawn "sh" "-c" "amane ipc call dash wallpapers"; }
Mod+A           { spawn "sh" "-c" "amane ipc call dash alerts"; }
Mod+G           { spawn "sh" "-c" "amane ipc call dash weather"; }

Mod+T           { spawn "sh" "-c" "amane ipc call tray toggle"; }
Mod+D           { spawn "sh" "-c" "amane ipc call open launcher"; }
Mod+V           { spawn "sh" "-c" "amane ipc call open clipboard"; }
Mod+Shift+E     { spawn "sh" "-c" "amane ipc call open power"; }
Mod+S           { spawn "sh" "-c" "amane ipc call open control"; }
Mod+Shift+D     { spawn "sh" "-c" "amane ipc call open spotlight"; }
Mod+Shift+L     { spawn "sh" "-c" "amane ipc call lock"; }

Escape          { spawn "sh" "-c" "amane ipc call closed"; }
Mod+N           { spawn "sh" "-c" "amane ipc call settings dark toggle"; }
Mod+Shift+N     { spawn "sh" "-c" "amane ipc call settings refresh"; }
```

Use sh -c for shell features such as pipes and command substitution.

## Keyboard navigation

| Panel | Keys |
| --- | --- |
| Launcher | Up/Down to select, Enter to launch, Esc to close |
| Spotlight | Up/Down to select, Enter to run, Esc to close |
| Application grid | Click to launch, Esc to close |
| Clipboard | Up/Down to select, Enter to copy, Esc to close |
| Power menu | Up/Down to select, Enter to run, Esc to close |
| Settings | Enter applies the focused field, Esc closes |
| Control center | Click toggles controls; sliders can be dragged |
| Dash | Esc closes |
| Tray | Esc closes when focused; clicking outside closes its menu |

## Mouse

| Element | Left click | Right click |
| --- | --- | --- |
| Tray item | Focus the window | Open its context menu |
| Launcher result | Launch the app | — |
| Clipboard entry | Copy the item | — |
| Island tray chevron | Toggle the tray | — |
| Outside an open panel | Close the panel | — |
