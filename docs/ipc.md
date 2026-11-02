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
| `island tray`                      | abre/fecha a tray de apps em 2º plano    |

Abas aceitas: `overview`, `media`, `wallpapers`, `alerts`, `weather`
(ou `clima`), `settings` (ou `ajustes`).

## overlay — as ilhas de ferramentas

| Comando                            | Efeito                                   |
|------------------------------------|------------------------------------------|
| `overlay launcher`                 | abre/fecha o launcher de apps            |
| `overlay clipboard`                | abre/fecha o histórico de clipboard      |
| `overlay power`                    | abre/fecha o menu de energia             |
| `overlay settings`                 | abre/fecha o painel de Ajustes           |
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
| `settings scheme`                  | mostra o esquema atual                   |
| `settings directory <pasta>`       | muda a pasta de onde a dash lê imagens   |
| `settings directory`               | mostra a pasta atual                     |
| `settings location <lat,lon>`      | fixa as coordenadas do clima             |
| `settings location auto`           | volta ao automático pelo ip              |
| `settings location`                | mostra o lugar atual (vazio = auto)      |
| `settings refresh`                 | roda o matugen de novo sem mudar nada    |

Esquemas válidos: `scheme-tonal-spot`, `scheme-content`,
`scheme-expressive`, `scheme-neutral`, `scheme-vibrant`,
`scheme-rainbow`, `scheme-fruit-salad`, `scheme-fidelity`.

Tudo aqui é o mesmo que o painel de Ajustes (`overlay settings`)
configura visualmente: aparência, pasta de papéis de parede e lugar
do clima.

## tray — apps em segundo plano

A tray mostra as janelas que o niri conhece — os apps rodando em
segundo plano ou parados em outro workspace — com o ícone real do
.desktop de cada um.

| Comando                            | Efeito                                   |
|------------------------------------|------------------------------------------|
| `tray toggle`                      | abre/fecha o menu                        |
| `tray close`                       | fecha o menu                             |
| `tray list`                        | lista as janelas ativas (id + título)    |
| `tray focus <id>`                  | levanta a janela (como o clique na tray) |
| `tray close-window <id>`           | fecha a janela (como o menu Fechar)      |

Na interface: clique esquerdo levanta a janela, clique direito abre
o menu com **Abrir** e **Fechar**.

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

# abrir o painel de Ajustes (visual)
amane ipc call overlay settings

# fixar o lugar do clima pelas coordenadas
amane ipc call settings location -20.44,-54.65

# ver quem está em segundo plano e levantar uma janela
amane ipc call tray list
amane ipc call tray focus 8
```
