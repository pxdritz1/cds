# Chloe Dynamic Shell

Um shell de desktop completo para Wayland, construído com Rust e o framework Amane, com identidade visual baseada no Material Design 3 (Material You).

## Características

- **Material Design 3**: Sistema de cores completo com tokens, tipografia Google Sans Flex e Material Symbols Rounded.
- **Dynamic Island**: Component de estado com múltiplos modos (compacto, mídia, notificações, volume, brilho, privacidade, rede, workspaces).
- **Launcher**: Pesquisa fuzzy com aplicativos, comandos, cálculos, arquivos e emojis.
- **Clipboard Manager**: Histórico com fixação, pesquisa e navegação por teclado.
- **System Tray**: Suporte a StatusNotifierItem.
- **Control Center**: Conectividade (Wi-Fi, Bluetooth, VPN), áudio, tela e energia.
- **Notification Center**: Agrupamento, histórico, ações e integração com a Island.
- **Dashboard**: Widgets de relógio, mídia, uso de CPU/RAM, bateria, rede e muito mais.
- **MPRIS**: Integração com players de mídia.
- **Configurações**: Interface completa com persistência entre sessões.

## Arquitetura

```
src/
├── art.rs              # Componentes reutilizáveis (cover, etc.)
├── clock.rs            # Relógio e data
├── island.rs           # Dynamic Island
├── motion.rs           # Animações
├── settings.rs         # Configurações persistidas
├── theme.rs            # Sistema de cores Material 3 e tipografia
├── ui.rs               # Estado da interface
├── wallpaper.rs        # Papel de parede
├── island/
│   ├── compact.rs      # Estado compacto da barra
│   ├── dash.rs         # Dashboard da Island
│   ├── peek.rs         # Estados expandidos (mídia, notificações)
│   └── widgets.rs      # Componentes reutilizáveis da Island
├── launcher.rs         # Launcher (Spotlight-like)
├── clipboard.rs        # Gerenciador de área de transferência
├── tray.rs             # Bandeja de sistema
├── control_center.rs   # Painel de controle
├── dashboard.rs        # Dashboard completo
├── settings_ui.rs      # Interface de configurações
└── media.rs            # Integração MPRIS
```

## Instalação

```bash
cargo build --release
```

## Execução

```bash
./target/release/amane-shell
```

## Configuração

As configurações são salvas em `~/.config/chloe-dynamic-shell/settings.conf`.

## Temas

O sistema de cores é gerenciado pelo matugen. Para alterar o tema:

```bash
amane-shell ipc theme set <imagem> [scheme] [dark|light]
amane-shell ipc theme color <hex> [scheme] [dark|light]
```

## Licença

Veja o arquivo LICENSE para detalhes.