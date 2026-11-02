# chloe's dynamic shell

A Material 3 Expressive desktop shell built with AGS 3, Astal 4, GJS, GTK4 and Gnim.

## Requirements

- AGS 3.1+
- Astal 4
- GTK 4
- A Wayland compositor with layer-shell support
- Google Sans Flex and Material Symbols Rounded (recommended)
- Optional integrations: `wpctl`, `playerctl`, `brightnessctl`, `cliphist`, `wl-copy`, `fuzzel`

## Development

```sh
ags run ./src/app.tsx
```

Generate editor typings:

```sh
ags types -d .
```

## Architecture

- `src/islands/`: independent shell surfaces for the three top corners
- `src/components/`: shared Material 3 components
- `src/services/`: system integrations and persistent preferences
- `src/theme.css`: Material 3 design tokens and component styling

## Design principles

- Material 3 Expressive shapes, tonal surfaces, typography and motion
- Three independent islands with expandable content
- Shared components and design tokens; no one-off visual styles
- Keyboard accessibility, clear focus states and reduced-motion support
- Optional system integrations must fail gracefully when their CLI is unavailable

The project is licensed under Apache License 2.0. See [LICENSE](./LICENSE).
