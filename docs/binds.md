# Binds do teclado (niri)

O shell não registra atalhos por si só: quem dispara tudo é o niri,
via `amane ipc call <serviço> <ação>`. Adicione os blocos abaixo na
seção `binds { }` do `~/.config/niri/config.kdl`.

## Aba de comandos sugeridos

```kdl
    // dash do shell (o painel grande)
    Mod+Space        { spawn "sh" "-c" "amane ipc call dash"; }

    // abrir o dash direto em uma aba
    Mod+O            { spawn "sh" "-c" "amane ipc call dash home"; }
    Mod+M            { spawn "sh" "-c" "amane ipc call dash media"; }
    Mod+W            { spawn "sh" "-c" "amane ipc call dash papeis"; }
    Mod+A            { spawn "sh" "-c" "amane ipc call dash alertas"; }
    Mod+G            { spawn "sh" "-c" "amane ipc call dash clima"; }

    // tray de apps em segundo plano (o mesmo chevron da barra)
    Mod+T            { spawn "sh" "-c" "amane ipc call island tray"; }

    // painéis estilo ilha
    Mod+D            { spawn "sh" "-c" "amane ipc call open launcher"; }
    Mod+V            { spawn "sh" "-c" "amane ipc call open clipboard"; }
    Mod+Shift+E      { spawn "sh" "-c" "amane ipc call open power"; }
    Mod+S            { spawn "sh" "-c" "amane ipc call open control"; }

    // o lock screen real (ext-session-lock + PAM)
    Mod+Shift+L      { spawn "sh" "-c" "amane ipc call lock"; }

    // o spotlight: busca central de apps e ações
    Mod+Shift+D      { spawn "sh" "-c" "amane ipc call open spotlight"; }

    // fechar qualquer ilha aberta (dash, tray, launcher, painéis…)
    Escape           { spawn "sh" "-c" "amane ipc call closed"; }

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
| Spotlight        | ↑/↓ escolhe, Enter executa, Esc fecha |
| Grade de apps    | clique abre, Esc fecha |
| Clipboard        | ↑/↓ escolhe, Enter copia, Esc fecha |
| Power            | ↑/↓ escolhe, Enter executa, Esc fecha |
| Ajustes          | campos aceitam Enter pra aplicar, Esc fecha |
| Central de controle | clique ativa/desativa, sliders arrastáveis, Esc fecha |
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
