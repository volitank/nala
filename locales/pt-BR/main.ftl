### Mensagens em português do Brasil exibidas pelo Nala.
###
### Tradução adaptada do catálogo pt_BR.po escrito por tradutores voluntários:
### Ciro Mota <contato.ciromota@outlook.com>, David Brochero e marcelo cripe.

## Rótulos e erros gerais

unknown = Desconhecido
none = Nenhum
error = Erro
mirrors = Espelhos:

log-error = Erro:
log-notice = Aviso:
log-warning = Advertência:
log-info = Informações:
log-verbose = Detalhado:
log-debug = Depuração:

not-implemented = Ainda não implementado.
subcommand-missing = Subcomando não fornecido

# Variables:
#   $response (String) - Invalid confirmation response.
prompt-invalid = '{ $response }' não é uma resposta válida
prompt-refused = O usuário recusou a confirmação
prompt-continue = Você quer continuar?
prompt-choice = [S/n]

# Variables:
#   $command (String) - Command that requires elevated privileges.
root-required = O Nala precisa de privilégios de root para { $command }

## Operações e motivos de transações

op-remove = Remover
op-auto-remove = Remoção automática
op-purge = Limpar
op-auto-purge = Limpeza automática
op-install = Instalar
op-reinstall = Reinstalar
op-upgrade = Atualizar
op-downgrade = Rebaixar
op-configure = Configurar
op-held = Mantido

reason-excluded = Excluído
reason-manual-hold = Retenção manual
reason-phased = Em fases
reason-kept-back = Mantido

# Variables:
#   $percentage (Number) - Phased update percentage.
reason-phased-percent = Em fases { $percentage }%

## Resumo da transação

summary-title = Sumário
summary-total-download = Tamanho total do download:
summary-disk-required = Espaço em disco necessário:
summary-disk-free = Espaço em disco a ser liberado:

summary-total-download-value = Tamanho total do download: { $size }
summary-disk-required-value = Espaço em disco necessário: { $size }
summary-disk-free-value = Espaço em disco a ser liberado: { $size }
summary-op-count = { $operation } { $count }
summary-op-count-colon = { $operation }: { $count }

summary-nothing = Nada a fazer.
summary-essential = Os seguintes pacotes são essenciais!
summary-remove-essential = Você tentou remover pacotes essenciais

# Variables:
#   $switch (String) - Command-line switch that permits essential package removal.
summary-use-switch = Use '{ $switch }' se tiver certeza.

summary-reboot = É necessário reiniciar para concluir estas alterações.
summary-reboot-packages = Os seguintes pacotes exigem uma reinicialização:

## Tabela de resumo dos pacotes

table-package = Pacote:
table-version = Versão:
table-old-version = Versão antiga:
table-new-version = Nova versão:
table-reason = Motivo:
table-size = Tamanho:

## Seleção de pacotes

# Variables:
#   $package (String) - Package name or selection token.
pkg-not-found = '{ $package }' não foi encontrado
pkg-not-found-cache = Alguns pacotes não foram encontrados no cache
pkg-no-candidate = { $package } não tem candidato para instalação
pkg-mark-install = Não foi possível marcar '{ $package }' para instalação
pkg-not-installed = { $package } não está instalado
pkg-reinstall-missing = { $package } não está instalado, portanto não pode ser reinstalado
pkg-invalid-name = Nome de pacote inválido: '{ $package }'

# Variables:
#   $package (String) - Package name.
#   $operation-a (String) - First requested operation.
#   $operation-b (String) - Conflicting requested operation.
pkg-op-conflict = Operações conflitantes para '{ $package }': { $operation-a } vs { $operation-b }

# Variables:
#   $package (String) - Package name.
#   $version-a (String) - First requested version.
#   $version-b (String) - Conflicting requested version.
pkg-version-conflict = Versões fixadas conflitantes para '{ $package }': { $version-a } vs { $version-b }

# Variables:
#   $package (String) - Package name.
#   $version (String) - Package version.
pkg-already-latest = { $package }{ $version } já está instalado e na versão mais recente
pkg-version-missing = Não foi possível encontrar a versão '{ $version }' de '{ $package }'
pkg-cache-missing = O pacote '{ $package }' não foi encontrado no cache
pkg-recorded-version-missing = Nenhuma versão registrada para '{ $package }'
pkg-version-cache-missing = A versão '{ $version }' não foi encontrada para '{ $package }'

# Variables:
#   $pin (String) - Version pin supplied by the user.
pkg-invalid-pin = Fixação de versão inválida: '{ $pin }'
pkg-pin-exact = A fixação de versão exige o nome exato do pacote: '{ $pin }'

# Variables:
#   $pattern (String) - Package exclusion pattern.
pkg-exclude-missing = O padrão de exclusão '{ $pattern }' não correspondeu a nenhum pacote

# Variables:
#   $package (String) - Virtual package name.
pkg-virtual-no-providers = { $package } não tem fornecedores e é puramente virtual
pkg-virtual-providers = O pacote { $package } é um pacote virtual fornecido por:

# Variables:
#   $package (String) - Virtual package name.
#   $provider (String) - Selected concrete provider.
pkg-virtual-select = Selecionando { $provider } em vez do pacote virtual { $package }
pkg-virtual-one = Você deve selecionar apenas um.

# Variables:
#   $selected (String) - Selected package name.
#   $requested (String) - Originally requested package name.
pkg-select = Selecionando o pacote '{ $selected }' em vez de '{ $requested }'

pkg-fix-broken = Não foi possível corrigir os pacotes quebrados
pkg-required = Você deve especificar um pacote

## Saída de downloads

download-complete = Downloads concluídos:
download-failed = Falha nos downloads
download-some-missing = Alguns pacotes não foram encontrados.
download-auth-warning = Os seguintes pacotes não podem ser autenticados!
download-auth-allowed = A configuração permite a instalação de pacotes não autenticados.
download-auth-required =
    Alguns pacotes não puderam ser autenticados.
      Se tiver certeza, use { $switch }
download-exit = Saindo a pedido do usuário
download-items = Itens

# Variables:
#   $package (String) - Package name.
#   $version (String) - Package version.
download-source-missing = Não foi possível encontrar uma fonte para baixar a versão '{ $version }' de '{ $package }'

# Variables:
#   $package (String) - Package name.
#   $path (String) - Destination path.
download-written = { $package } foi gravado em { $path }

# Variables:
#   $uri (String) - Download URI.
download-hash-missing = Nenhum hash encontrado para '{ $uri }'
download-protocol-missing = Nenhum protocolo foi definido
download-uri-missing = Nenhuma URI foi definida
download-malformed = '{ $uri }' está malformada!
download-request-failed = A solicitação de download falhou para '{ $uri }'
download-stream-failed = Não foi possível transmitir os dados de '{ $uri }'
download-get = Não foi possível enviar a solicitação de download
download-content-length = content-length não existe em { $headers }
download-content-str = Falha ao converter content-length em texto: { $headers }
download-content-parse = Falha ao analisar content-length: { $headers }

# Variables:
#   $file (String) - Downloaded filename.
download-checksum = A soma de verificação não corresponde a { $file }
download-no-uris = Nenhuma URI pôde ser baixada para { $file }

# Variables:
#   $path (String) - Path without a valid filename.
download-filename = { $path } não tem um nome de arquivo válido!

download-error = Erro: { $error }
download-select-domain = Selecionando { $domain } para { $file }
download-start = Iniciando: { $uri }, tentativas: { $retries }
download-finished = Concluído: { $uri }

## Progresso

progress-working = Trabalhando...
progress-nothing = Nada para buscar
progress-label = Progresso
progress-total = Total
progress-speed = Velocidade
progress-elapsed = Decorrido
progress-remaining = Restante
progress-finished = Concluído:

# Variables:
#   $size (String) - Total amount fetched.
#   $time (String) - Elapsed time.
#   $rate (String) - Transfer rate.
progress-fetched = Baixados { $size } em { $time } ({ $rate }/s)

## Interface de texto

tui-upgrade-title = Atualização do Nala
tui-history-title = Histórico do Nala
tui-pkgs = Pacotes
tui-help-move = (↑) mover para cima | (↓) mover para baixo
tui-help-tabs = (→) próxima aba | (←) aba anterior
tui-help-show = (Enter) exibir changelog | (s) exibir informações da versão
tui-help-confirm = (q) sair | (y) iniciar atualização
tui-help-quit = (q) sair
tui-changelog-missing = Não foi possível encontrar a URI do changelog

## Arquivos, configuração e arquivos de pacotes

# Variables:
#   $path (String) - File or directory path.
file-remove = Falha ao remover { $path }
file-read = Falha ao ler '{ $path }'
file-read-defaults = Falha ao ler { $path }, usando os padrões
file-parse-defaults = Falha ao analisar { $path }, usando os padrões
file-create = Não foi possível criar '{ $path }'
file-write = Não foi possível gravar em '{ $path }'
file-replace = Não foi possível substituir '{ $path }'
file-deserialize = Não foi possível desserializar '{ $path }'

# Variables:
#   $operation (String) - Filesystem operation.
#   $path (String) - Source path.
fs-failed = Falha ao { $operation } { $path }

# Variables:
#   $operation (String) - Filesystem operation.
#   $path (String) - Source path.
#   $target (String) - Destination path.
fs-failed-target = Falha ao { $operation } { $path } => { $target }

# Variables:
#   $option (String) - Unsupported APT configuration override.
config-option = A opção '{ $option }' não é compatível
color-expected = um nome de cor, índice de 0 a 255 ou hexadecimal #RRGGBB
color-index = o índice de cor deve estar entre 0 e 255
color-unknown = cor desconhecida '{ $color }'
color-rgb-components = Rgb exige três componentes
color-rgb-expected = era esperado Rgb ou Indexed
color-modifier-expected = uma opção de estilo como "BOLD | ITALIC" ou uma lista
color-modifier-unknown = opção de estilo desconhecida '{ $modifier }'

archive-unsupported = O tipo de arquivo não é compatível

# Variables:
#   $path (String) - Debian archive path.
deb-control-missing = arquivo control não encontrado em { $path }

# Variables:
#   $type (String) - Hash type.
hash-unsupported = O tipo de hash { $type } não é compatível

# Variables:
#   $package (String) - Package name.
#   $version (String) - Package version.
hash-unavailable =
    A integridade de { $package } { $version } não pode ser verificada.
    Não existem hashes disponíveis para este pacote.

## Dpkg

dpkg-child-failed = O processo filho do dpkg falhou: { $error }
dpkg-status-utf8 = O descritor de status do dpkg retornou UTF-8 inválido
dpkg-exit = O dpkg saiu com o código: '{ $code }'
dpkg-poll = Não foi possível consultar o processo filho
dpkg-read-status = Não foi possível ler o descritor de status
dpkg-read-pty = Não foi possível ler do pty
dpkg-write-pty = Não foi possível enviar a entrada para o pty
dpkg-removing = Removendo:
dpkg-unpacking = Desempacotando:
dpkg-setting-up = Configurando:
dpkg-processing = Processando:

## List

list-virtual = Virtual

## Show: saída geral

show-local-install = Instalado localmente

# Variables:
#   $count (Number) - Number of additional package records.
#   $switch (String) - Command-line switch that displays every record.
show-more-records = Existem { $count } registros adicionais. Use a opção { $switch } para exibi-los.

show-no-description = Sem descrição
show-no-summary = Sem resumo

## Show: rótulos dos registros de pacotes

show-package = Pacote
show-version = Versão
show-architecture = Arquitetura
show-priority = Prioridade
show-essential = Essencial
show-section = Seção
show-source = Fonte
show-installed-size = Tamanho instalado
show-size = Tamanho
show-maintainer = Mantenedor
show-original-maintainer = Mantenedor original
show-homepage = Página inicial
show-sha256 = SHA256
show-archive = Arquivo
show-origin = Origem
show-codename = Codinome
show-component = Componente
show-provides = Fornece
show-description = Descrição
show-attributes = Atributos
show-apt-sources = Fontes do APT
show-depends = Depende
show-pre-depends = Pré-depende
show-suggests = Sugere
show-recommends = Recomenda
show-conflicts = Conflita
show-replaces = Substitui
show-obsoletes = Torna obsoleto
show-breaks = Quebra
show-enhances = Melhora

## Show: atributos dos pacotes

show-attr-installed = Instalado
show-attr-local = Local
show-attr-auto-removable = Removível automaticamente
show-attr-automatic = Automático

# Variables:
#   $version (String) - Package version available for upgrade.
show-attr-upgradable-to = Atualizável para: { $version }

# Variables:
#   $version (String) - Currently installed package version.
show-attr-upgradable-from = Atualizável a partir de: { $version }

## Policy

policy-none = nenhum
policy-installed = Instalado:
policy-candidate = Candidato:
policy-version-table = Tabela de versões:
policy-no-versions = Nenhuma versão.
policy-origin = origem
policy-package-files = Arquivos de pacotes:
policy-pinned = Pacotes fixados:
policy-release = versão

## History: saída geral

# Variables:
#   $count (Number) - Number of history entries that were cleared.
history-cleared =
    { $count ->
        [one] Limpou { $count } entrada do histórico.
       *[other] Limpou { $count } entradas do histórico.
    }

history-empty = Nenhuma entrada de histórico encontrada.

# Variables:
#   $id (Number) - ID of the history entry.
history-not-found = A entrada de histórico com ID '{ $id }' não existe
history-not-replayable = A entrada de histórico '{ $id }' não pode ser reaplicada porque não foi registrada como aplicada
history-no-changes = A entrada de histórico '{ $id }' não tem alterações de pacotes para reaplicar
history-cleared-entry = Entrada de histórico { $id } removida.

## History: rótulos

history-id = ID
history-command = Comando
history-date-time = Data e hora
history-requested-by = Solicitado por
history-altered = Alterados
history-status = Status
history-status-applied = Aplicada
history-started = Iniciada
history-finished = Concluída
history-targets = Alvos solicitados

## History: erros de repetição

# Variables:
#   $package (String) - Package name.
history-undo-version-missing = Não é possível desfazer '{ $package }' porque a versão anterior não foi registrada
history-undo-config-only = Não é possível desfazer '{ $package }' porque a restauração do estado somente arquivos de configuração não foi implementada
history-undo-installed-version-missing = Não é possível desfazer '{ $package }' porque a versão instalada anteriormente não foi registrada
history-undo-reinstall = Não é possível desfazer '{ $package }' porque a reinstalação não tem uma operação inversa registrada
history-undo-held = O pacote retido '{ $package }' não pode ser desfeito
history-redo-version-missing = Não é possível refazer '{ $package }' porque a versão resultante não foi registrada
history-redo-reinstall-missing = Não é possível refazer '{ $package }' porque a versão reinstalada não foi registrada
history-redo-held = O pacote retido '{ $package }' não pode ser refeito

## History: erros de seleção e armazenamento

history-clear-target = A limpeza do histórico exige um seletor de entrada ou --all
history-config-undo = O pacote configurado '{ $package }' não pode ser desfeito
history-config-redo = O pacote configurado '{ $package }' não pode ser refeito
history-serialize = Não foi possível serializar a entrada do histórico

## Fetch

fetch-no-mirrors = O Nala não conseguiu encontrar nenhum espelho.
fetch-none-selected = Nenhum espelho foi selecionado.
fetch-release-detect = Houve um problema ao detectar a versão.

# Variables:
#   $distro (String) - Distribution name.
fetch-unsupported = { $distro } não é compatível.

# Variables:
#   $file (String) - Sources file path.
fetch-sources-written = As fontes foram gravadas no arquivo { $file }

fetch-title = Nala Fetch
fetch-score = Pontuação:
fetch-score-help = A pontuação indica quantos milissegundos são necessários para baixar o arquivo Release.
fetch-help = Use ↓↑ para mover, Espaço para selecionar/desmarcar, Home/End para ir ao início/fim e q/Enter para sair.

## Update

# Variables:
#   $count (Number) - Number of upgradable packages.
#   $command (String) - Command that lists upgradable packages.
update-upgradable =
    { $count ->
        [one] { $count } pacote pode ser atualizado. Execute '{ $command }' para vê-lo.
       *[other] { $count } pacotes podem ser atualizados. Execute '{ $command }' para vê-los.
    }

update-no-change = Sem alterações
update-updated = Atualizado
update-ignored = Ignorado
update-downloading = Baixando
update-processing = Processando
update-item = { $state }: { $description }

## Upgrade

upgrade-exclude-unsafe =
    Os pacotes selecionados não podem ser excluídos da atualização com segurança.
    { $error }
upgrade-protect = Protegendo { $package } contra { $reason }
upgrade-reason-upgrade = atualização
upgrade-reason-auto-remove = remoção automática
upgrade-config-missing = Nenhuma árvore de configuração!

## Install

install-downloaded = Baixado: { $path }
