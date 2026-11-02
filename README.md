# Chloe Dynamic Shell

Um shell de desktop para Wayland, construído em Rust com o framework Amane e uma identidade visual baseada no Material Design 3 (Material You).

## Características

- **Material Design 3**: paleta dinâmica gerada pelo matugen, tipografia Google Sans Flex e ícones Material Symbols Rounded.
- **Dynamic Island**: barra compacta que vira pílula de mídia ao passar o mouse, mostra notificações recentes e abre o dash com um clique.
- **Dash**: cinco abas (visão geral, mídia, papéis de parede, alertas e aplicativos) com relógio, controles de áudio/brilho, medidores de CPU/RAM/bateria, grade de wallpapers e lançador de aplicativos.
- **Animações M3**: morphing da ilha com curvas de desaceleração, fades de conteúdo por modo e por aba, state layers de hover, cores e larguras animadas.
- **Interação no estilo Dux**: hover para espiar a mídia, clique para abrir o dash e clique fora ou `Esc` para fechar.

## Arquitetura

```
src/
├── art.rs              # Capa de álbum e fallbacks de metadados
├── clock.rs            # Relógio, data e saudação (via date(1))
├── island.rs           # Janela da ilha, modos e transições
├── motion.rs           # Animações e state layers do Material 3
├── settings.rs         # Papéis de parede e preferências persistidas
├── theme.rs            # Tokens de cor, tipografia e utilidades
├── ui.rs               # Estado do dash e comandos IPC
├── wallpaper.rs        # Janela do papel de parede
└── island/
    ├── compact.rs      # Barra compacta (workspaces, relógio, status)
    ├── dash.rs         # Conteúdo das abas do dash
    ├── peek.rs         # Pílulas de mídia e notificação
    └── widgets.rs      # Componentes e ícones Material
```

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

O sistema de cores é gerenciado pelo matugen. Para alterar o tema:

```bash
amane ipc call theme set <imagem> [scheme] [dark|light]
amane ipc call theme color <hex> [scheme] [dark|light]
```

## Licença

Veja o arquivo LICENSE para detalhes.
