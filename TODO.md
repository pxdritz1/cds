# TODO

## Barra e ilhas

- [ ] Reposicionar a tray na ilha direita, imediatamente depois do Wi-Fi no sentido do centro da tela.
- [ ] Substituir a falsa tray baseada em janelas do Niri por uma StatusNotifierItem (SNI) real via D-Bus, com registro, atualização, ativação e menus de contexto.
- [ ] Adicionar window title com ícone do aplicativo à ilha esquerda, imediatamente à direita dos workspace pills.
- [ ] Adicionar ícone de busca à ilha central, imediatamente à esquerda do relógio, integrado ao Spotlight existente.
- [ ] Adicionar somente o ícone/indicador de clima à direita da data, sem temperatura ou texto adicional.

## Tema e wallpapers

- [ ] Investigar e corrigir congelamentos ao trocar wallpaper: serializar execuções do Matugen, evitar concorrência entre processos e impedir I/O bloqueante no caminho de desenho/IPC.
- [ ] Tornar a geração de cores resiliente a falhas e timeouts; preservar a última paleta válida e publicar o arquivo de cores atomicamente.
- [ ] Exibir caminhos de wallpaper e diretórios com prefixo `~/` quando estiverem dentro do HOME, mantendo caminhos absolutos internamente e aceitando ambos na entrada.
- [ ] Criar/garantir os diretórios padrão de wallpapers sem sobrescrever diretórios configurados pelo usuário.

## Configurações

- [ ] Manter o configurador visual como janela/painel independente (não como widget da Dynamic Island).
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
