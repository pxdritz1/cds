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
│   ├── cds-settingsview·     painel de Ajustes (Overlay)   │
│   ├── (lock)        · tela de lock via ext-session-lock    │
│   └── wallpaper     · fundo de tela (Layer::Bottom)       │
└───────────────────────────────────────────────────────────┘
        ↑ camada de serviços (RwLock por serviço)
┌───────────────────────────────────────────────────────────┐
│ Services: Theme, Settings, Ui, Overlay, SettingsView,     │
│ Tray, Weather, Launcher, Clipboard, Power, Spotlight +    │
│ serviços da amane (Lock, Apps, Media, Audio, Battery,     │
│ Network, Bluetooth, Brightness, Workspaces…)              │
└───────────────────────────────────────────────────────────┘
        ↑ IPC (amane ipc call …) ── threads de escuta
```

## Módulos

| Arquivo                    | Papel                                                     |
|----------------------------|-----------------------------------------------------------|
| `main.rs`                  | registra janelas por monitor + handlers de IPC            |
| `theme.rs`                 | roda o matugen, merge com a config do usuário, observa o colors.conf |
| `settings.rs`              | wallpaper, dark, esquema, diretório, lugar do clima, persistência |
| `settingsview.rs`          | painel de Ajustes (visual): aparência, papéis de parede, clima  |
| `ui.rs`                    | estado do dash (aberto? qual aba?) + IPC island                 |
| `island.rs`                | janelas da ilha: compacta, media peek, notice, dash, tray       |
| `island/compact.rs`        | a barra compacta (dots, relógio, bateria, chevron)              |
| `island/dash.rs`           | dash de 6 abas: Overview/Media/Wallpapers/Alertas/Clima/Ajustes  |
| `island/peek.rs`           | media peek e notice peek                                        |
| `island/widgets.rs`        | ícones, chips, sliders, cards, botões e o shape cookie (M3)      |
| `overlay.rs`               | serviço Overlay + helper `layer` (Under/Spot/Right/Middle)       |
| `ipc.rs`                   | pontos de entrada do CLI (nomes curtos: open/lock/dash/…)   |
| `apps.rs`                  | grade de aplicativos (app drawer M3)                        |
| `spotlight.rs`             | busca central: apps + ações rápidas do shell               |
| `control.rs`               | central de controle à direita (wifi/bt/dnd/sliders)          |
| `lockscreen.rs`            | tela de lock via ext-session-lock + PAM                     |
| `launcher.rs`              | launcher: pesquisa de apps, setas/Enter, clique abre            |
| `clipboard.rs`             | clipboard:poll de wl-paste, wl-copy, histórico persistido        |
| `power.rs`                 | menu de energia (lock/logout/suspend/reboot/shutdown)           |
| `tray.rs`                  | apps em segundo plano (janelas do niri), menu de contexto        |
| `weather.rs`               | open-meteo (+ hoas do dia), ip-api ou coordenadas do Settings    |
| `motion.rs`                | motor de animações M3 (fade, presence, follow, cores)     |
| `art.rs`, `clock.rs`, `wallpaper.rs` | capas de mídia, relógio, papel de parede        |

## Serviços

Um serviço declara `Service` e ganha uma instância global com lock:
`Service::read()` para olhar, `Service::write()` para mudar (o guard
sinaliza mudança ao soltar, a menos que o poll não tenha achado nada
novo em `update()`). Métodos estáticos (`Tray::toggle()`) encapsulam
mutações. A camada de desenho roda por janela; mudar um serviço pede
redesenho naquilo que o mostra.

Regras que mantêm a UI fluida:

- **Nada de I/O com o lock de desenho.** quem toca em disco ou rede
  (`Weather::update`, o scan de `Apps`) faz isso fora do lock; a tray
  pergunta ao niri a cada 2,5 s via `update()` do próprio `Service`.
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
| `~/.config/chloe-dynamic-shell/settings.conf`  | wallpaper, esquema, dark, diretório, lugar do clima |
| `~/.cache/chloe-dynamic-shell/colors.conf`     | papéis de cores do matugen    |
| `~/.cache/chloe-dynamic-shell/clipboard.conf`  | histórico (1 entrada/linha)   |
| `~/.cache/chloe-dynamic-shell/weather.json`    | última leitura do clima       |

## Painéis: como uma "ilha de ferramenta" abre

Os painéis (launcher, clipboard, power, apps, spotlight, control, config)
usam o helper `overlay::layer`: uma janela full-screen `Layer::Overlay`,
backdrop escurecido, e um painel com sombra-36 que cai de baixo da barra
(margin-top 58px) com fade + settle (0.96 → 1.0). O lugar de cada painel
é definido por `Frame::place` — `Under` (launcher e amigos, logo abaixo da
barra), `Spot` (spotlight, centralizado e maior), `Right` (a central de
controle, ancorada na borda direita) ou `Middle`. Teclado é
`Keyboard::Exclusive` quando aberto, Esc fecha, clique fora fecha.
A presença não desaparece na hora: a janela fica mapeada até a animação
terminar (`motion::presence`), tanto na abertura como no fechamento.

## Tray

A tray é a lista de **apps em segundo plano**: o serviço pergunta ao
niri (`niri msg --json windows`) a cada 2,5 s e guarda id, app-id,
título e janela focada. O ícone real vem do `.desktop` que casa com o
app-id (via `Apps`). Clique esquerdo levanta a janela
(`niri msg action focus-window`), clique direito abre o menu com
**Abrir** e **Fechar** (`close-window`); o menu mora dentro do mesmo
sheet, que a barra e o menu compartilham a animação de follow
(largura/altura/raio). Quando a janela morre, o menu de contexto dela
sai junto.

## Cookie

O shape expressivo do Material 3 (`island/widgets.rs::cookie`) desenha
um círculo cuja borda escalopa `lobes` lóbulos ao redor, amostrado em
coordenadas polares num `Path` fechado preenchido num `Canvas`. A aba
Clima usa atrás do glifo da condição; trocar `lobes` muda o número de
ondas, `depth` (9% do raio) muda o quanto as reentrâncias cortam.
