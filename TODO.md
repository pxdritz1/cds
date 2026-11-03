# TODO

## Barra e ilhas

- [ ] Adicionar abertura do Dash ao passar o mouse sobre a Island, com opção configurável para ativar/desativar o comportamento.
- [ ] Fechar o Dash ao clicar fora de sua área, respeitando cliques em controles internos e outras janelas do shell.

- [ ] Reposicionar a tray na ilha direita, imediatamente depois do Wi-Fi no sentido do centro da tela.
- [ ] Substituir a falsa tray baseada em janelas do Niri por uma StatusNotifierItem (SNI) real via D-Bus, com registro, atualização, ativação e menus de contexto.
- [ ] Adicionar window title com ícone do aplicativo à ilha esquerda, imediatamente à direita dos workspace pills.
- [x] Adicionar ícone de busca à ilha central, imediatamente à esquerda do relógio, integrado ao Spotlight existente.
- [x] Adicionar somente o ícone/indicador de clima à direita da data, sem temperatura ou texto adicional.

## Tema e wallpapers

- [ ] Adicionar nas configurações uma opção para ativar/desativar a rotação automática de wallpapers e configurar o intervalo.

- [x] Serializar execuções do Matugen no processo, executar a geração fora da thread de desenho e aplicar timeout de 45 s.
- [ ] Verificar se o IPC que chama a geração síncrona também precisa ser movido para worker; testar congelamentos reais no compositor.
- [x] Tornar a geração resiliente a falhas/timeouts e publicar a paleta validada atomicamente, preservando a última paleta válida.
- [x] Exibir o diretório de wallpapers com prefixo `~/` e aceitar `~/` ou caminhos absolutos na entrada do diretório e do wallpaper.
- [ ] Aplicar a mesma apresentação `~/` a todos os lugares que exibem o caminho atual do wallpaper.
- [x] Criar/garantir o diretório padrão de wallpapers sem sobrescrever diretórios configurados pelo usuário.

## Configurações

- [ ] Corrigir o botão de configurações do Dash para abrir a janela independente de configurações de forma confiável.

- [x] Manter o configurador visual como janela independente do Amane (não como widget da Dynamic Island).
- [ ] Ampliar o configurador para controlar posição e visibilidade dos elementos da barra, além de cores e aparência.
- [ ] Persistir as opções e aplicar alterações sem reiniciar o shell.

## Spotlight

- [ ] Exibir os ícones reais dos aplicativos no Spotlight, usando os ícones GTK/desktop correspondentes em vez de reutilizar apenas o ícone genérico de abertura de aplicativos do GNOME.
- [ ] Garantir fallback para ícones de aplicações sem ícone GTK disponível, respeitando tema e escala.

## Dash

- [ ] Animar de verdade os indicadores ondulados de CPU, RAM e bateria: movimento contínuo e suave, alimentado por valores atualizados, sem ondas estáticas.
- [ ] No card de relógio do Dash, adicionar opção para alternar entre relógio digital e analógico.
- [ ] Fazer o card de clima do Dash abrir diretamente a aba de clima ao ser clicado.

## Notificações

- [ ] Exibir no preview da notificação o ícone real do aplicativo de origem, com fallback Material 3.
- [ ] Quando o protocolo/origem disponibilizar esses metadados, exibir também o avatar/ícone da pessoa ou conversa destinatária/remetente.
- [ ] Definir precedência e fallback: avatar da conversa/pessoa > ícone do aplicativo > ícone genérico.
- [ ] Verificar suporte e limites dos hints de imagem/ícone do protocolo de notificações; não inferir identidade de contato a partir do texto da mensagem.

## Qualidade e validação

- [ ] Seguir os padrões existentes de Rust, serviços Amane, componentes Material 3 e separação por módulos.
- [ ] Não remover nem sobrescrever alterações locais do usuário.
- [ ] Validar compilação, inicialização, troca repetida de wallpapers, tray SNI com mais de um aplicativo, notificações e múltiplos monitores.
- [ ] Atualizar README e documentação de arquitetura/IPC após a implementação.
