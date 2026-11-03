# Referência de IPC

Toda comunicação com o shell passa pelo CLI:

```bash
amane ipc call <serviço> <ação...>
```

## Nomes curtos (o padrão)

| Comando                            | Efeito                                   |
|------------------------------------|------------------------------------------|
| `open <painel>`                    | abre/fecha um painel (launcher, apps, spotlight, clipboard, power, control, config) |
| `closed`                           | fecha **tudo** que estiver aberto        |
| `dash [aba]`                       | abre/fecha o dash (clima, media, papeis…)|
| `lock`                             | bloqueia a sessão (tela de lock)         |

Aliases antigos (`island`, `overlay`) ainda funcionam, mas a documentação usa os novos.

## open — os painéis

| Comando                            | Efeito                                   |
|------------------------------------|------------------------------------------|
| `open launcher`                    | busca de aplicativos                     |
| `open apps`                        | grade de aplicativos (app drawer)        |
| `open spotlight`                   | busca central + ações rápidas do shell   |
| `open clipboard`                   | histórico de clipboard                   |
| `open power`                       | menu de energia                          |
| `open control`                     | central de controle à direita            |
| `open config`                      | painel de Ajustes do shell               |

## dash — o painel grande

```bash
amane ipc call dash                # alterna o dash
amane ipc call dash clima          # abre o dash direto na aba clima
```

Abas aceitas: `home`, `media`, `clima`, `papeis`, `alertas`.

## lock — a tela de bloqueio

```bash
amane ipc call lock
```

Fecha tudo, mostra a tela de lock real via `ext-session-lock` com PAM. Encerra-se com a senha do usuário.

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

Tudo aqui é o mesmo que o painel de Ajustes (`open config`)
configura visualmente: aparência, pasta de papéis de parede e lugar
do clima.

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
amane ipc call open launcher

# trocar de wallpaper
amane ipc call settings wallpaper ~/Pictures/Wallpapers/dragona.png

# abrir o painel de Ajustes (visual)
amane ipc call open config

# fixar o lugar do clima pelas coordenadas
amane ipc call settings location -20.44,-54.65

# ver quem está em segundo plano e levantar uma janela
amane ipc call tray list
amane ipc call tray focus 8
```
