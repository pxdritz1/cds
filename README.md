# Chloe Dynamic Shell

Um shell de desktop para Wayland, construído em Rust com o framework Amane e uma identidade visual baseada no Material Design 3 (Material You).

> 📖 **Documentação completa em [`docs/`](docs/):** [`binds.md`](docs/binds.md) (atalhos do niri), [`ipc.md`](docs/ipc.md) (comando por comando) e [`arquitetura.md`](docs/arquitetura.md) (como o shell funciona por dentro).

## Características

- **Material Design 3**: paleta dinâmica gerada pelo matugen, tipografia Google Sans Flex e ícones Material Symbols Rounded — incluindo os **shapes expressivos** M3 (cookie), usados no badge da aba Clima e no lockscreen.
- **Dynamic Island**: barra compacta que vira pílula de mídia ao passar o mouse, mostra notificações recentes e abre o dash com um clique.
- **Rail de workspaces**: ilha fininha na borda esquerda com os workspaces numerados (ativo destacado, urgente em erro, vazios em meia força).
- **Dash**: cinco abas (Home, Media, Clima, Papéis e Alertas) com hero de relógio, clima, controles, medidores wavy M3 e mídia.
- **Painéis**: Launcher (pesquisa), **Spotlight** (busca central + ações rápidas), **Grade de Apps** (o app drawer), Clipboard, Menu de energia e **Central de Controle** à direita (Wi-Fi, Bluetooth, Não perturbe, tema, sliders de volume/brilho, bateria e notificações).
- **Lockscreen**: tela de bloqueio real via `ext-session-lock` com PAM — relógio grande e senha.
- **Ajustes por IPC**: um appzinho visual (`open config`) que configura o shell na hora — modo escuro/claro, os 8 esquemas matugen, a pasta de papéis de parede e o lugar do clima.
- **Tray de apps em segundo plano**: o chevron da barra lista as **janelas que o niri conhece** com o ícone real do `.desktop`; clique esquerdo levanta, clique direito abre o menu (Abrir / Fechar).
- **Clipboard**: histórico persistente do `wl-clipboard`, busca, cópia de volta ao clipboard e limpeza do histórico.
- **Menu de energia**: bloquear, sair da sessão, suspender, reiniciar e desligar.
- **Animações M3**: morphing da ilha, overlays com scrim, fade e escala, fades de conteúdo por modo/aba, state layers de hover, cores e larguras animadas.
- **Interação no estilo Dux**: hover para espiar a mídia, clique para abrir o dash, clique fora ou `Esc` para fechar qualquer painel; launcher, clipboard e energia entram como **ilhas ancoradas no topo**, caindo de trás da barra.

## Atalhos

O shell não configura bindings sozinho; use os comandos IPC no seu compositor. Para o niri:

```kdl
Mod+Space       { spawn-sh "amane ipc call dash"; }                // dash
Mod+D           { spawn-sh "amane ipc call open launcher"; }
Mod+V           { spawn-sh "amane ipc call open clipboard"; }
Mod+Shift+E     { spawn-sh "amane ipc call open power"; }
Mod+S           { spawn-sh "amane ipc call open control"; }         // central à direita
Mod+T           { spawn-sh "amane ipc call tray toggle"; }          // tray de apps
Mod+N           { spawn-sh "amane ipc call settings dark toggle"; }
Mod+Shift+L     { spawn-sh "amane ipc call lock"; }                 // bloqueia a sessão
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
amane ipc call open launcher       # alterna o launcher
amane ipc call open apps           # grade de aplicativos (o app drawer)
amane ipc call open spotlight      # busca central estilo spotlight
amane ipc call open clipboard      # alterna o clipboard
amane ipc call open power          # alterna o menu de energia
amane ipc call open control        # central de controle à direita
amane ipc call open config         # o painel de Ajustes do shell
amane ipc call closed              # fecha tudo que estiver aberto

amane ipc call dash [aba]          # abre/fecha o dash (clima, media, papeis…)
amane ipc call tray [toggle|close|list|focus <id>|close-window <id>]
amane ipc call lock                # bloqueia a sessão (tela de lock)

amane ipc call settings wallpaper ~/Pictures/Wallpapers/foo.png
amane ipc call settings wallpapers
amane ipc call settings dark [true|false|toggle]
amane ipc call settings scheme <esquema>
amane ipc call settings directory [pasta]   # de onde a dash lê os wallpapers
amane ipc call settings location [lat,lon|auto] # o lugar do clima
amane ipc call settings refresh       # regenera o matugen de tudo
amane ipc call theme set <imagem> [scheme] [dark|light]
amane ipc call theme color <hex> [scheme] [dark|light]
```

Os nomes antigos (`overlay`, `island`) ainda funcionam como apelidos, mas a documentação aponta pros novos.

## Arquitetura

```
src/
├── art.rs              # Capa de álbum e fallbacks de metadados
├── apps.rs             # Grade de aplicativos (app drawer M3)
├── clipboard.rs        # Histórico do clipboard, IPC e overlay
├── clock.rs            # Relógio, data e saudação (via date(1))
├── control.rs          # Central de controle à direita (M3)
├── island.rs           # Janela da ilha, modos e transições
├── ipc.rs              # Comandos IPC (nomes curtos)
├── launcher.rs         # Busca de aplicativos e overlay
├── lockscreen.rs       # Tela de lock (ext-session-lock + PAM)
├── m3.rs               # Biblioteca de componentes M3
├── motion.rs           # Animações, presença de overlay e state layers
├── overlay.rs          # Painel ativo, scrim e componentes compartilhados
├── rail.rs             # Ilha de workspaces na borda esquerda
├── settings.rs         # Wallpapers, tema, esquema, lugar do clima e IPC
├── settingsview.rs     # Painel de Ajustes (open config)
├── spotlight.rs        # Spotlight: busca + ações rápidas
├── theme.rs            # Tokens de cor, matugen e tipografia
├── tray.rs             # Tray de apps em segundo plano (janelas do niri) e IPC
├── ui.rs               # Estado do dash e comandos IPC
├── wallpaper.rs        # Janela do papel de parede
├── weather.rs          # Clima (open-meteo, com previsão por hora)
└── island/
    ├── compact.rs      # Barra compacta (workspaces, relógio, chevron)
    ├── dash.rs         # Conteúdo das abas do dash (Clima, cookie M3)
    ├── peek.rs         # Pílulas de mídia e notificação
    └── widgets.rs      # Componentes, ícones e o shape cookie (M3)
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
