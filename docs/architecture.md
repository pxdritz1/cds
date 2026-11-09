# Native architecture

## Runtime

CDS is one native Rust process. Iced owns application state, the message loop, widget tree and rendering. iced_exwlshell provides the Wayland shell roles that ordinary Iced windows do not provide:

- wlr-layer-shell surfaces for the bar, islands, overlays and wallpaper layer.
- ext-session-lock for the lock screen.
- Per-output surface lifecycle and shell events.

The app starts through `cds daemon` or `cds run`. A Unix-domain IPC server lives at `$XDG_RUNTIME_DIR/cds.sock`; the `cds` CLI sends structured requests to the running process. The socket is private to the current user.

## Target module layout

```text
native/
  main.rs                 CLI entry point
  cli.rs                  cds commands and source-checkout build actions
  paths.rs                XDG paths
  ipc.rs                  Unix socket protocol and server
  app.rs                  Iced daemon, window registry and shell messages
  services/
    niri.rs               Niri IPC and workspace/focused-window state
    settings.rs           Persistent preferences
    theme.rs              Matugen palette and wallpaper integration
    media.rs              MPRIS
    tray.rs               StatusNotifierWatcher and dbusmenu
    notifications.rs      org.freedesktop.Notifications service
    clipboard.rs          Clipboard history
    audio.rs              Audio devices, volume and microphone
    network.rs            NetworkManager and Wi-Fi
    bluetooth.rs          BlueZ
    weather.rs            Open-Meteo
    desktop_apps.rs       Desktop entries, icons and launching
  ui/
    island/               Compact, Media, Notice and Dash
    panels/               Launcher, Spotlight, Apps, Clipboard, Power, Control
    settings/             Settings window
    lockscreen.rs         Session-lock surface and PAM
    m3/                   Material 3 components and tokens
    motion.rs             M3 and Fluent motion
```

The old `src/` tree is retained temporarily as a behavioral reference and is not compiled by the native binary. It will be retired after each feature has a native replacement and passes validation.

## State and concurrency

- UI state changes happen through Iced messages.
- D-Bus, filesystem, network and subprocess work must not run in the drawing path.
- Long-lived services publish typed updates to the Iced message loop.
- D-Bus calls and subprocesses are asynchronous; expensive parsing and image work stay off the UI thread.
- Settings and caches use the XDG paths documented in the README.
- Layer surfaces are tracked by Iced window ID and output; closing a surface removes its state.

## Design invariants

1. No dependency on Amane at build time or runtime.
2. The command and binary name is `cds`.
3. Existing panel names, keybindings and interaction semantics are retained.
4. Multi-monitor behavior is first-class; surfaces are not hard-coded to one output.
5. The Material 3 palette remains Matugen-driven, with pt-BR/English language selection.
6. Feature parity is not declared until static validation and live Niri/Wayland tests pass.
