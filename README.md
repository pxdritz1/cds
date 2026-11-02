# Chloe Dynamic Shell

Um shell de desktop para Wayland, construído em Rust com o framework Amane e uma identidade visual baseada no Material Design 3 (Material You).

> 📖 **Documentação completa em [`docs/`](docs/):** [`binds.md`](docs/binds.md) (atalhos do niri), [`ipc.md`](docs/ipc.md) (comando por comando) e [`arquitetura.md`](docs/arquitetura.md) (como o shell funciona por dentro).

## Características

- **Material Design 3**: paleta dinâmica gerada pelo matugen, tipografia Google Sans Flex e ícones Material Symbols Rounded.
- **Dynamic Island**: barra compacta que vira pílula de mídia ao passar o mouse, mostra notificações recentes e abre o dash com um clique.
- **Dash**: seis abas (Overview, Media, Wallpapers, Alerts, Apps e **Ajustes**) com relógio, **card de clima (open-meteo)**, controles de áudio/brilho, medidores de CPU/RAM/bateria, grade de wallpapers e **configuração visual** (claro/escuro + esquema matugen na hora).
- **Tray de apps**: o chevron da barra substitui o ícone de wifi e abre um menuzinho com os apps fixados; clique esquerdo abre, **clique direito abre o menu de contexto** (Abrir / Desafixar). Fixe apps pelo menu da aba Apps ou do launcher (clique direito).
- **Launcher**: busca de aplicativos por nome ou descrição, navegação por teclado e abertura com `Enter`.
- **Clipboard**: histórico persistente do `wl-clipboard`, busca, cópia de volta ao clipboard e limpeza do histórico.
- **Menu de energia**: bloquear, sair da sessão, suspender, reiniciar e desligar.
- **Animações M3**: morphing da ilha, overlays com scrim, fade e escala, fades de conteúdo por modo/aba, state layers de hover, cores e larguras animadas.
- **Interação no estilo Dux**: hover para espiar a mídia, clique para abrir o dash, clique fora ou `Esc` para fechar qualquer painel; launcher, clipboard e energia entram como **ilhas ancoradas no topo**, caindo de trás da barra.

## Atalhos

O shell não configura bindings sozinho; use os comandos IPC no seu compositor. Para o niri:

```kdl
Mod+Space       { spawn-sh "amane ipc call island toggle"; }        // dash
Mod+D           { spawn-sh "amane ipc call overlay launcher"; }
Mod+Shift+V     { spawn-sh "amane ipc call overlay clipboard"; }
Mod+Shift+E     { spawn-sh "amane ipc call overlay power"; }
Mod+T           { spawn-sh "amane ipc call island tray"; }          // tray de apps
Mod+N           { spawn-sh "amane ipc call settings dark toggle"; }
```

Dentro dos ilhas:

| Tecla | Ação |
| --- | --- |
| `Esc` | fecha o painel atual (dash, tray e overlays) |
| `↑` / `↓` / `Tab` | percorre as opções |
| `Enter` | abre o app, copia o item ou executa a ação |
| clique fora | fecha o painel |

A lista completa de sugestões de binds vive em [`docs/binds.md`](docs/binds.md).

## IPC

```bash
amane ipc call overlay launcher       # alterna o launcher
amane ipc call overlay clipboard      # alterna o clipboard
amane ipc call overlay power          # alterna o menu de energia
amane ipc call overlay close          # fecha qualquer overlay
amane ipc call overlay panel          # imprime o painel aberto

amane ipc call island toggle          # alterna o dash
amane ipc call island open [aba]      # abre o dash numa aba (…|settings)
amane ipc call island close
amane ipc call island tray            # alterna a tray de apps

amane ipc call settings wallpaper ~/Pictures/Wallpapers/foo.png
amane ipc call settings wallpapers
amane ipc call settings dark [true|false|toggle]
amane ipc call settings scheme <esquema>
amane ipc call settings refresh       # regenera o matugen de tudo

amane ipc call tray [toggle|close|pin "App"|unpin "App"|list]
amane ipc call weather

amane ipc call theme set <imagem> [scheme] [dark|light]
amane ipc call theme color <hex> [scheme] [dark|light]
```

## Arquitetura

```
src/
├── art.rs              # Capa de álbum e fallbacks de metadados
├── clipboard.rs        # Histórico do clipboard, IPC e overlay
├── clock.rs            # Relógio, data e saudação (via date(1))
├── island.rs           # Janela da ilha, modos e transições
├── launcher.rs         # Busca de aplicativos e overlay
├── motion.rs           # Animações, presença de overlay e state layers
├── overlay.rs          # Painel ativo, scrim e componentes compartilhados
├── power.rs            # Menu de energia e ações de sessão
├── settings.rs         # Wallpapers, tema, esquema e IPC
├── theme.rs            # Tokens de cor, matugen e tipografia
├── tray.rs             # Tray de apps fixados, menu de contexto e IPC
├── ui.rs               # Estado do dash e comandos IPC
├── wallpaper.rs        # Janela do papel de parede
├── weather.rs          # Clima (open-meteo) para a dashboard
└── island/
    ├── compact.rs      # Barra compacta (workspaces, relógio, chevron)
    ├── dash.rs         # Conteúdo das abas do dash (inclui Ajustes)
    ├── peek.rs         # Pílulas de mídia e notificação
    └── widgets.rs      # Componentes e ícones Material
```

Cada overlay é uma layer window própria em tela cheia, criada com `visible(false)`. Ao abrir, o estado global em `overlay.rs` mostra só a janela do monitor focado, aplica `Keyboard::Exclusive` e anima a presença; ao fechar, a janela continua mapeada até o fade terminar. Assim o compositor não perde o foco do teclado no meio da animação e o `Esc` sempre chega ao painel. Ver [`docs/arquitetura.md`](docs/arquitetura.md) para o completo.

## Execução

O projeto é um shell do Amane, então o fluxo normal é:

```bash
amane dev       # compila e recarrega a cada alteração
amane compile   # gera o binário otimizado
amane run       # inicia o binário compilado
```

## Tipografia

O shell pede **Google Sans Flex** ao fontconfig para o texto e **Material Symbols Rounded** para os ícones.

A Google Sans Flex é uma fonte variável com mais de 32 *tuples* de `gvar` por glifo. A versão do `ttf-parser` usada pelo Amane não consegue desenhar esses glifos sem a feature `gvar-alloc`, o que fazia todo o texto sumir. Por isso o sistema usa instâncias estáticas da fonte (Regular/Medium/SemiBold/Bold) em `~/.local/share/fonts/GoogleSansFlex/`; a fonte variável original foi preservada em `~/.local/share/fonts-disabled/`.

Para regenerar as instâncias estáticas:

```bash
python -m pip install fonttools
python scripts/google-sans-static.py GoogleSansFlex.ttf ~/.local/share/fonts/GoogleSansFlex
fc-cache -f
```

## Temas

O sistema de cores é gerenciado pelo matugen. O shell monta um config temporário em `~/.cache/chloe-dynamic-shell/matugen/config.toml` que **inclui o config global do usuário** (`~/.config/matugen/config.toml`) mais a template do shell. Dessa forma uma troca de wallpaper atualiza o shell e os outros aplicativos (alacritty, niri, spicetify etc.) na mesma execução.

O shell também passa `--prefer saturation` e `--continue-on-error`, então imagens com várias cores de origem não travam esperando uma escolha no terminal.

O histórico do clipboard fica em `~/.cache/chloe-dynamic-shell/clipboard.conf`; as preferências ficam em `~/.config/chloe-dynamic-shell/settings.conf`.

## Licença

Veja o arquivo LICENSE para detalhes.
