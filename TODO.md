# TODO

## Barra e ilhas

- [x] Adicionar abertura do Dash ao passar o mouse sobre a Island, com opção configurável para ativar/desativar o comportamento.
- [ ] Garantir que qualquer clique fora do Dash o feche, inclusive fora dos limites atuais da layer central, sem bloquear/interferir com cliques destinados às janelas subjacentes.

- [x] Reposicionar a tray na ilha direita, imediatamente depois do Wi-Fi no sentido do centro da tela.
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

- [x] Corrigir o botão de configurações do Dash e registrar a janela independente no Amane para que ela possa ser aberta sob demanda.

- [x] Manter o configurador visual como janela independente do Amane (não como widget da Dynamic Island).
- [ ] Ampliar o configurador para controlar posição e visibilidade dos elementos da barra, além de cores e aparência.
- [ ] Persistir as opções e aplicar alterações sem reiniciar o shell.

## Spotlight

- [x] Exibir no Spotlight os ícones reais dos aplicativos, usando os caminhos de ícone resolvidos pelo tema GTK/desktop; manter fallback Material Symbols quando não houver ícone.
- [ ] Garantir fallback para ícones de aplicações sem ícone GTK disponível, respeitando tema e escala.

## Dash

- [x] Ativar a animação contínua das ondas de CPU, RAM e bateria no Dash, mantendo o comprimento da onda vinculado ao valor de uso atualizado.
- [ ] No card de relógio do Dash, adicionar opção para alternar entre relógio digital e analógico.
- [x] Fazer o card de clima do Dash abrir diretamente a aba de clima ao ser clicado.

## Utilitários

- [ ] Adicionar um color picker integrado ao shell, com seleção de cor e cópia do valor hexadecimal para a área de transferência.
- [ ] Adicionar um seletor de emojis usando o conjunto visual Google Noto Color Emoji/Google Emoji, com busca e cópia do emoji selecionado para a área de transferência.

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
