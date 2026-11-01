# chloe dynamic shell

shell para wayland feito em rust com amane, inspirado no material 3.

o projeto está sendo desenvolvido em fases. esta primeira etapa estabelece a estrutura, a janela-base, os tokens visuais e a integração inicial com matugen.

## requisitos

- wayland com suporte a `wlr-layer-shell`
- [amane](https://github.com/MystiaFin/amane)
- rust e cargo
- [matugen](https://github.com/InioX/matugen) 1.x

## desenvolvimento

aponte o amane para a pasta clonada:

```sh
amane target "$PWD"
amane dev
```

para compilar sem iniciar:

```sh
amane compile
```

## cores dinâmicas

gere a paleta material 3 a partir de um wallpaper:

```sh
matugen -c "$PWD/matugen/config.toml" image "$HOME/Pictures/wallpaper.jpg" -m dark
```

a paleta é escrita em `~/.config/chloe-dynamic-shell/generated/colors.json`. a configuração não altera o wallpaper por conta própria. nesta etapa, a shell ainda usa os tokens de fallback; a leitura e a atualização em tempo real da paleta entram na fase da dynamic island.

## estrutura

- `src/main.rs`: entrada e composição da shell
- `src/theme.rs`: tokens de fallback material 3
- `matugen/`: configuração e template de cores dinâmicas
- `.github/workflows/`: verificações automatizadas

## roadmap

1. base, módulos, tokens material 3 e matugen
2. dynamic island, estados, animações e dashboard
3. wallpapers, aplicação e configuração visual
4. launcher, grade de apps e spotlight
5. clipboard e tray
6. notificações, osd e integrações adicionais

## licença

apache-2.0. veja [license](LICENSE).
