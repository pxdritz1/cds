# TODO

## Barra e ilhas

- [ ] Reposicionar a tray na ilha direita, imediatamente depois do Wi-Fi no sentido do centro da tela.
- [ ] Substituir a falsa tray baseada em janelas do Niri por uma StatusNotifierItem (SNI) real via D-Bus, com registro, atualização, ativação e menus de contexto.
- [ ] Adicionar window title com ícone do aplicativo à ilha esquerda, imediatamente à direita dos workspace pills.
- [x] Adicionar ícone de busca à ilha central, imediatamente à esquerda do relógio, integrado ao Spotlight existente.
- [x] Adicionar somente o ícone/indicador de clima à direita da data, sem temperatura ou texto adicional.

## Tema e wallpapers

- [x] Serializar execuções do Matugen no processo, executar a geração fora da thread de desenho e aplicar timeout de 45 s.
- [ ] Verificar se o IPC que chama a geração síncrona também precisa ser movido para worker; testar congelamentos reais no compositor.
- [x] Tornar a geração resiliente a falhas/timeouts e publicar a paleta validada atomicamente, preservando a última paleta válida.
- [x] Exibir caminhos de wallpaper e diretórios com prefixo `~/` quando estiverem dentro do HOME, mantendo caminhos absolutos internamente e aceitando ambos na entrada.
- [x] Criar/garantir o diretório padrão de wallpapers sem sobrescrever diretórios configurados pelo usuário.

## Configurações

- [x] Manter o configurador visual como janela independente do Amane (não como widget da Dynamic Island).
- [ ] Ampliar o configurador para controlar posição e visibilidade dos elementos da barra, além de cores e aparência.
- [ ] Persistir as opções e aplicar alterações sem reiniciar o shell.

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
