# Binds do teclado (niri)

O shell não registra atalhos por si só: quem dispara tudo é o niri,
via `amane ipc call <serviço> <ação>`. Adicione os blocos abaixo na
seção `binds { }` do `~/.config/niri/config.kdl`.

## Aba de comandos sugeridos

```kdl
    // dash da island (o painel grande com as 6 abas)
    Mod+Space        { spawn "sh" "-c" "amane ipc call island toggle"; }

    // abrir o dash direto em uma aba
    Mod+O            { spawn "sh" "-c" "amane ipc call island open overview"; }
    Mod+M            { spawn "sh" "-c" "amane ipc call island open media"; }
    Mod+W            { spawn "sh" "-c" "amane ipc call island open wallpapers"; }
    Mod+A            { spawn "sh" "-c" "amane ipc call island open alerts"; }
    Mod+G            { spawn "sh" "-c" "amane ipc call island open clima"; }

    // tray de apps em segundo plano (o mesmo chevron da barra)
    Mod+T            { spawn "sh" "-c" "amane ipc call island tray"; }

    // overlays estilo ilha
    Mod+D            { spawn "sh" "-c" "amane ipc call overlay launcher"; }
    Mod+V            { spawn "sh" "-c" "amane ipc call overlay clipboard"; }
    Mod+Shift+E      { spawn "sh" "-c" "amane ipc call overlay power"; }
    Mod+S            { spawn "sh" "-c" "amane ipc call overlay settings"; }

    // fechar qualquer ilha aberta (dash, tray, launcher, ajustes…)
    Escape           { spawn "sh" "-c" "amane ipc call overlay close; amane ipc call island close"; }

    // tema: trocar wallpaper / alternar claro e escuro / regenerar tudo
    Mod+N            { spawn "sh" "-c" "amane ipc call settings dark toggle"; }
    Mod+Shift+N      { spawn "sh" "-c" "amane ipc call settings refresh"; }
    Mod+Shift+W      { spawn "sh" "-c" "amane ipc call settings wallpaper $(ls ~/Pictures/Wallpapers/* | shuf -n1)"; }
```

> **Dica:** o `spawn "sh" "-c"` existe porque pipelines e substituições
> de comando ($(…)) não são interpretadas pelo niri sozinhas.
> Para comandos simples, `spawn "amane" "ipc" "call" "island" "toggle"`
> também funciona.

## Navegação dentro das ilhas (sem bind, já vem de fábrica)

| Lugar            | Teclas                       |
|------------------|------------------------------|
| Launcher         | ↑/↓ escolhe, Enter abre, Tab desce, Esc fecha |
| Clipboard        | ↑/↓ escolhe, Enter copia, Esc fecha |
| Power            | ↑/↓ escolhe, Enter executa, Esc fecha |
| Ajustes          | campos aceitam Enter pra aplicar, Esc fecha |
| Dash             | Esc fecha (o foco de teclado é exclusivo) |
| Tray             | clique fora do menu fecha; Esc fecha se a janela tiver foco |

## Mouse

| Lugar                    | Esquerdo         | Direito                             |
|--------------------------|------------------|-------------------------------------|
| App na tray              | levanta a janela | abre o menu de contexto (Abrir / Fechar) |
| Resultado do launcher    | abre o app       | —                                   |
| Entrada do clipboard     | copia de volta   | —                                   |
| Chevron da barra         | abre/fecha a tray| —                                   |
| Fora de qualquer ilha    | fecha a ilha     | —                                   |
