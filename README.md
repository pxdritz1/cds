# Chloe Dynamic Shell

Shell Wayland feito com amane: a barra é uma dynamic island, tudo em Material 3 com cores geradas pelo matugen.

## Rodar

```sh
export AMANE_CONFIG=$PWD
amane compile
amane run
```

## IPC

```sh
amane ipc call island toggle
amane ipc call island open media
amane ipc call island close
amane ipc call theme set ~/Pictures/Wallpapers/wall.png
amane ipc call theme color "#6750A4"
```
