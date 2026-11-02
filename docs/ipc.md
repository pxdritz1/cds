# Referência de IPC

Toda comunicação com o shell passa pelo CLI:

```bash
amane ipc call <serviço> <ação...>
```

## island — a ilha principal

| Comando                            | Efeito                                   |
|------------------------------------|------------------------------------------|
| `island toggle`                    | abre/fecha o dash                        |
| `island open [aba]`                | abre o dash na aba dada                  |
| `island close`                     | fecha o dash e a tray                    |
| `island tray`                      | abre/fecha a tray de apps                |

Abas aceitas: `overview`, `media`, `wallpapers`, `alerts`, `apps`,
`settings` (ou `ajustes`).

## overlay — as ilhas de ferramentas

| Comando                            | Efeito                                   |
|------------------------------------|------------------------------------------|
| `overlay launcher`                 | abre/fecha o launcher de apps            |
| `overlay clipboard`                | abre/fecha o histórico de clipboard      |
| `overlay power`                    | abre/fecha o menu de energia             |
| `overlay close`                    | fecha o que estiver aberto               |
| `overlay panel`                    | diz qual painel está aberto (ou `none`)  |

## settings — imagens e modo

| Comando                            | Efeito                                   |
|------------------------------------|------------------------------------------|
| `settings wallpaper <imagem>`      | troca o wallpaper e regenera as cores    |
| `settings wallpaper`               | mostra o wallpaper atual                 |
| `settings wallpapers`              | lista as imagens do diretório            |
| `settings dark true|false|toggle`  | modo escuro/claro (regenera tudo)        |
| `settings scheme <nome>`           | troca o esquema matugen e regenera       |
| `settings refresh`                 | roda o matugen de novo sem mudar nada    |

Esquemas válidos: `scheme-tonal-spot`, `scheme-content`,
`scheme-expressive`, `scheme-neutral`, `scheme-vibrant`,
`scheme-rainbow`, `scheme-fruit-salad`, `scheme-fidelity`.

## tray — apps fixados na barra

| Comando                            | Efeito                                   |
|------------------------------------|------------------------------------------|
| `tray toggle`                      | abre/fecha o menu                        |
| `tray close`                       | fecha o menu                             |
| `tray pin "Nome do App"`           | fixa um app (nome do .desktop)           |
| `tray unpin "Nome do App"`          | desafixa                                 |
| `tray list`                        | lista os apps fixados                    |

A lista fixada vive em `~/.config/chloe-dynamic-shell/tray.conf`,
um nome por linha. Numa primeira instalação, sem arquivo, a tray mostra
um menu padrão com os apps comuns que já estiverem instalados.

## theme — cor crua (sem tocar no wallpaper)

| Comando                            | Efeito                                   |
|------------------------------------|------------------------------------------|
| `theme set <imagem> [scheme] [dark|light]` | gera cores a partir de uma imagem |
| `theme color <hex> [scheme] [dark|light]`  | gera cores a partir de um hex     |

## weather — clima da dash

| Comando     | Efeito                                        |
|-------------|-----------------------------------------------|
| `weather`   | mostra `condição | temp | umid. | vento`      |

O clima vem do open-meteo (sem chave de API) com a localização pela
ip pública; atualiza a cada 15 minutos em uma thread separada, então
nunca trava a interface. A última leitura fica em
`~/.cache/chloe-dynamic-shell/weather.json`.

## Exemplos de uso diário

```bash
# abrir o launcher e procurar
amane ipc call overlay launcher

# trocar de wallpaper
amane ipc call settings wallpaper ~/Pictures/Wallpapers/dragona.png

# forçar modo claro
amane ipc call settings dark false

# ver a cada quanto esquema estamos
amane ipc call settings scheme

# fixar o firefox na tray
amane ipc call tray pin firefox
```
