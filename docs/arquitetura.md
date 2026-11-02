# Arquitetura

## Visão geral

```
┌───────────────────────────────────────────────────────────┐
│ niri (compositor)                                         │
│   ├── cds-reserve   · reserva 44px no topo (click-through)│
│   ├── cds-island    · a ilha (barra compacta/dash/tray)   │
│   ├── cds-launcher  · ilha do launcher (Layer::Overlay)   │
│   ├── cds-clipboard ·       ilha do clipboard             │
│   ├── cds-power     ·       ilha de energia               │
│   └── wallpaper     · fundo de tela (Layer::Bottom)       │
└───────────────────────────────────────────────────────────┘
        ↑ camada de serviços (RwLock por serviço)
┌───────────────────────────────────────────────────────────┐
│ Services: Theme, Settings, Ui, Overlay, Tray, Weather,    │
│ Launcher, Clipboard, Power + serviços da amane (Apps,     │
│ Media, Audio, Battery, Network, Workspaces…)              │
└───────────────────────────────────────────────────────────┘
        ↑ IPC (amane ipc call …) ── threads de escuta
```

## Módulos

| Arquivo                    | Papel                                                     |
|----------------------------|-----------------------------------------------------------|
| `main.rs`                  | registra janelas por monitor + handlers de IPC            |
| `theme.rs`                 | roda o matugen, merge com a config do usuário, observa o colors.conf |
| `settings.rs`              | wallpaper, dark, esquema, persistência                    |
| `ui.rs`                    | estado do dash (aberto? qual aba?) + IPC island           |
| `island.rs`                | janelas da ilha: compacta, media peek, notice, dash, tray |
| `island/compact.rs`        | a barra compacta (dots, relógio, bateria, chevron)        |
| `island/dash.rs`           | dash de 6 abas: Overview/Media/Wallpapers/Alerts/Apps/Ajustes |
| `island/peek.rs`           | media peek e notice peek                                  |
| `island/widgets.rs`        | ícones material symbols, chips, sliders, cards, botões    |
| `overlay.rs`               | serviço Overlay + helper `layer` que ancora painéis no topo |
| `launcher.rs`              | launcher: pesquisa de apps, setas/Enter, clique-direito fixa |
| `clipboard.rs`             | clipboard:poll de wl-paste, wl-copy, histórico persistido |
| `power.rs`                 | menu de energia (lock/logout/suspend/reboot/shutdown)     |
| `tray.rs`                  | tray de apps fixados + menu de contexto                   |
| `weather.rs`               | open-meteo + ip-api, thread própria, card da dash         |
| `motion.rs`                | motor de animações M3 (fade, presence, follow, cores)     |
| `art.rs`, `clock.rs`, `wallpaper.rs` | capas de mídia, relógio, papel de parede        |

## Serviços

Um serviço declara `Service` e ganha uma instância global com lock:
`Service::read()` para olhar, `Service::write()` para mudar. Métodos
estáticos (`Tray::toggle()`) encapsulam mutações. A camada de desenho
roda por janela; mudar um serviço pede redesenho naquilo que o mostra.

Regras que mantêm a UI fluida:

- **Nada de I/O com o lock de desenho.** quem toca em disco ou rede
  (`Clipboard::listen`, `Weather::listen`, o scan de `Apps`) faz isso em
  thread própria, tudo que está dentro de `listen()` roda fora do lock.
- **O lock do Settings sai antes do matugen.** `set_wallpaper`/`set_dark`
  clonam os dados, `drop` do guard, e só depois chamam `theme::apply` —
  trocar de wallpaper nunca congela o desenho.
- **Matugen em subprocesso, nunca em primeiro plano.** `theme::apply`
  monta a config merged num arquivo em cache (`/tmp/matugen/config.toml`)
  que junta a configuração do usuário com o template do shell, e roda
  `matugen … -t scheme -m dark|light --prefer saturation
  --continue-on-error` com `amane::spawn`.

## Persistência

| Arquivo                                        | O que guarda                  |
|------------------------------------------------|-------------------------------|
| `~/.config/chloe-dynamic-shell/settings.conf`  | wallpaper, esquema, dark, diretório |
| `~/.config/chloe-dynamic-shell/tray.conf`      | apps fixados na tray          |
| `~/.cache/chloe-dynamic-shell/colors.conf`     | papéis de cores do matugen    |
| `~/.cache/chloe-dynamic-shell/clipboard.conf`  | histórico (1 entrada/linha)   |
| `~/.cache/chloe-dynamic-shell/weather.json`    | última leitura do clima       |

## Overlays: como uma "ilha de ferramenta" abre

As três ferramentas (launcher, clipboard, power) usam o helper
`overlay::layer`: uma janela full-screen `Layer::Overlay`, backdrop
escurecido, e um painel com sombra-32 que cai de baixo da barra
(margin-top 58px) com fade + settle (0.96 → 1.0). Teclado é
`Keyboard::Exclusive` quando aberto, Esc fecha, clique fora fecha.
A presença não desaparece na hora: a janela fica mapeada até a animação
terminar (`motion::presence`), tanto na abertura como no fechamento.

## Tray

O chevron troca o wifi na barra (rolar a direita). Abrindo, a mesma
ilha estica pra baixo dentro de uma cartão `surface_container_highest`
com grade de apps fixados (4 por linha). Clique esquerdo abre o app,
clique direito abre o menu de contexto com Abrir/Desafixar.
O menu mora dentro do mesmo sheet; a barra e o menu compartilham a
animação de follow (largura/altura/raio).
