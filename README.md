# chloe dynamic shell

shell para wayland feita em rust com amane, inspirada no material 3.

## requisitos

- wayland com suporte a `wlr-layer-shell`
- [amane](https://github.com/MystiaFin/amane)
- rust e cargo
- [matugen](https://github.com/InioX/matugen) 1.x

## desenvolvimento

```sh
amane target "$PWD"
amane dev
```

para compilar sem iniciar:

```sh
amane compile
```

## dynamic island

a island é o ponto central da shell. o estado compacto mostra notificações e a reprodução ativa; clicar na ilha abre o dash. quando há mídia mpris, a ilha assume o modo de player, com capa em moldura m3, rotação, linha de progresso ondulada e play/pause.

- scroll no player: ajusta o volume em passos de 5%.
- clique do scroll na área de mídia: reservado para pausar/retomar quando o ponteiro está sobre o player.
- clique na área de volume do dash: alterna o mudo.
- scroll na área de volume do dash: ajusta o volume.
- notificações recebidas pelo serviço do amane aparecem no dash.

o dash já reúne controles nativos do amane para áudio, rede, bluetooth, bateria, cpu, memória, áreas de trabalho e launcher. a integração dedicada com tray de status notifier, histórico de clipboard e provedor de clima ainda precisa ser implementada; por enquanto, essas seções aparecem identificadas como pendentes. o spotlight permanece separado e não faz parte da island.

o amane atua como daemon de notificações. para recebê-las, ele precisa ser o proprietário de `org.freedesktop.Notifications`; outro daemon como mako ou dunst pode impedir essa captura.

## cores dinâmicas

gere uma paleta material 3 a partir de um wallpaper:

```sh
matugen -c "$PWD/matugen/config.toml" image "$HOME/Pictures/wallpaper.jpg" -m dark
```

a paleta é gerada em `~/.config/chloe-dynamic-shell/generated/colors.json`. a leitura em runtime e a atualização visual automática serão integradas em uma etapa posterior.

## roadmap

1. base, módulos, tokens material 3 e matugen
2. dynamic island e dashboard (em implementação)
3. wallpapers, aplicação e configuração visual
4. launcher, grade de apps e spotlight
5. clipboard e tray
6. notificações, osd e integrações adicionais

## licença

apache-2.0. veja [license](LICENSE).
