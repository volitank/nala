### Messages migrated from legacy/python/po/pt.po.
### Missing messages fall back to English.

log-error = Erro:

log-notice = Aviso:

log-warning = Advertência:

prompt-continue = Quer continuar?

prompt-choice = Y/y N/n

root-required = O Nala requer o acesso root para { $command }

op-remove = Remover

op-auto-remove = Remoção automática

op-purge = Purgar

op-auto-purge = Purga automática

op-install = Instalar

op-reinstall = Reinstalar

op-upgrade = Atualizar

op-downgrade = Desatualizar

op-configure = Configurar

op-held = Mantidos

reason-kept-back = Mantidos

summary-title = Resumo

summary-total-download = Tamanho total da transferência:

summary-disk-required = Espaço em disco necessário:

summary-disk-free = Espaço em disco a libertar:

summary-total-download-value = Tamanho total da transferência: { $size }

summary-disk-required-value = Espaço em disco necessário: { $size }

summary-disk-free-value = Espaço em disco a libertar: { $size }

summary-nothing = Nada para o Nala fazer.

summary-essential = Os seguintes pacotes são essenciais!

summary-remove-essential = Tentou remover pacotes essenciais

summary-use-switch = Usar { $switch } se tiver a certeza.

summary-reboot = É necessário um reinício.

summary-reboot-packages = Os pacotes seguintes requerem um reinício.

table-package = Pacote:

table-version = Versão:

table-old-version = Versão antiga:

table-new-version = Versão nova:

table-size = Tamanho:

pkg-no-candidate = { $package } não tem candidato à instalação.

pkg-not-installed = { $package } não está instalado

pkg-already-latest = { $package } já está na versão mais recente { $version }

pkg-version-missing = Versão { $version } não encontrada para o pacote { $package }

pkg-cache-missing = { $package } mas não está no cache

pkg-recorded-version-missing = não encontra versão para { $package }

pkg-version-cache-missing = Versão { $version } não encontrada para o pacote { $package }

pkg-virtual-providers = { $package } é um pacote virtual fornecido por:

pkg-virtual-one = Deve selecionar apenas um.

download-complete = Transferência concluída:

download-auth-warning = Os seguintes pacotes não podem ser autenticados!

download-auth-allowed = A configuração está definida para permitir a instalação de pacotes não autenticados.

download-exit = Sair a seu pedido.

progress-working = [Working]

progress-remaining = Tempo restante

progress-fetched = { $size } em { $time } ({ $rate }/s)

tui-pkgs = Pacotes

file-read-defaults = Incapaz de ler o ficheiro de configuração: { $path }. A usar a predefinição

hash-unsupported = Tipo de hash '{ $type }' não é suportado

dpkg-removing = A remover:

dpkg-unpacking = A desempacotar:

dpkg-setting-up = A configurar:

dpkg-processing = A processar:

list-virtual = Pacote virtual

show-local-install = instalado localmente

show-more-records = Existem { $count } registos adicionais. Usar o comutador { $switch } para vê-los.

show-no-description = Sem descrição

show-package = Pacote

show-version = Versão

show-architecture = Arquitetura

show-priority = Prioridade

show-essential = Essencial

show-section = Secção

show-source = Fonte

show-installed-size = Tamanho pós-instalação

show-maintainer = Responsável

show-original-maintainer = Responsável original

show-homepage = Página inicial

show-origin = Origem

show-provides = Fornece

show-description = Descrição

show-apt-sources = Fontes APT

show-depends = Depende

show-pre-depends = Pré-depende

show-suggests = Sugere

show-recommends = Recomenda

show-conflicts = Conflita

show-replaces = Substitui

show-breaks = Quebra

show-enhances = Melhora

show-attr-installed = Installado

show-attr-upgradable-to = está instalado e atualizável para { $version }

show-attr-upgradable-from = é atualizável a partir de { $version }

policy-installed = Installado:

history-empty = Não existe histórico.

history-not-found = A transação { $id } não existe.

fetch-no-mirrors = O Nala não conseguiu encontrar quaisquer espelhos.

fetch-release-detect = Houve um problema ao detectar a versão.

fetch-sources-written = As fontes foram gravadas em { $file }

fetch-score-help = A pontuação é quantos milissegundos são necessários para transferir o ficheiro Release

update-upgradable = { $count } pacotes podem ser atualizados. Executar "{ $command }" para vê-los.

update-no-change = Sem alterações

update-updated = Atualizado

update-ignored = Ignorado

update-downloading = A transferir

update-processing = A processar

install-downloaded = Transferido: { $path }

## Additional messages

unknown = Desconhecido

none = Nenhum

error = Erro

mirrors = Espelhos:

log-info = Informação:

log-verbose = Detalhado:

log-debug = Depuração:

not-implemented = Ainda não implementado.

subcommand-missing = Não foi indicado um subcomando

prompt-invalid = '{ $response }' não é uma resposta válida

prompt-refused = O utilizador recusou a confirmação

prompt-choice-no = [y/N]

release-info-title = Informações do Repositório Alteradas

release-info-explanation =
    As informações de identificação de um repositório foram alteradas no seu ficheiro Release.
    Reveja as alterações antes de continuar.

release-info-repository = Repositório

release-info-distribution = Distribuição

release-info-changes = Alterações

release-info-confirm = Aceitar estas alterações e continuar a atualização?

release-info-help = (y) aceitar | (n/Enter/Esc) rejeitar

release-info-accepted = Alterações às informações do repositório { $repository } aceites.

release-info-rejected = Alterações às informações do repositório { $repository } rejeitadas.

release-info-noninteractive = As alterações às informações Release têm de ser aceites explicitamente. Execute novamente com '-o Acquire::AllowReleaseInfoChange=true' para as aceitar.

reason-excluded = Excluído

reason-manual-hold = Retido manualmente

reason-phased = Faseado

reason-phased-percent = Faseado { $percentage }%

summary-op-count = { $operation } { $count }

summary-op-count-colon = { $operation }: { $count }

summary-protected = Os seguintes pacotes são essenciais ou estão protegidos!

summary-remove-protected = Tentou remover pacotes essenciais ou protegidos

summary-downgrade-assume-yes = As versões dos pacotes seriam reduzidas com --assume-yes. Utilize '-o APT::Get::allow-downgrades=true' se tiver a certeza.

summary-held-assume-yes = Os pacotes retidos seriam alterados com --assume-yes. Utilize '-o APT::Get::allow-change-held-packages=true' se tiver a certeza.

autoremove-config-purge = --remove-config tem de ser utilizado com --purge ou autopurge.

autoremove-assume-yes = A ignorar a remoção automática com --assume-yes. Utilize --autoremove para a ativar.

table-reason = Motivo:

pkg-not-found = '{ $package }' não foi encontrado

pkg-not-found-cache = Alguns pacotes não foram encontrados na cache

pkg-mark-install = Não foi possível marcar '{ $package }' para instalação

pkg-reinstall-missing = { $package } não está instalado, pelo que não pode ser reinstalado

pkg-invalid-name = Nome de pacote inválido: '{ $package }'

pkg-op-conflict = Operações incompatíveis para '{ $package }': { $operation-a } contra { $operation-b }

pkg-version-conflict = Versões fixadas incompatíveis para '{ $package }': { $version-a } contra { $version-b }

pkg-invalid-pin = Versão fixada inválida: '{ $pin }'

pkg-pin-exact = A fixação de versão exige o nome exato do pacote: '{ $pin }'

pkg-exclude-missing = O padrão de exclusão '{ $pattern }' não corresponde a nenhum pacote

pkg-virtual-no-providers = { $package } não tem fornecedores e é puramente virtual

pkg-virtual-select = A selecionar { $provider } em vez do pacote virtual { $package }

pkg-select = A selecionar o pacote '{ $selected }' em vez de '{ $requested }'

pkg-fix-broken = Não foi possível corrigir os pacotes danificados

pkg-required = Tem de indicar um pacote

download-failed = Transferências falhadas

download-some-missing = Alguns pacotes não foram encontrados.

download-auth-required =
    Não foi possível autenticar alguns pacotes.
    Se tiver a certeza, utilize { $switch }

download-items = Itens

download-source-missing = Não foi possível encontrar uma fonte para transferir a versão '{ $version }' de '{ $package }'

download-written = { $package } foi escrito em { $path }

download-hash-missing = Não foi encontrado um hash para '{ $uri }'

download-protocol-missing = Não foi definido um protocolo

download-uri-missing = Não foi definido um URI

download-malformed = '{ $uri }' está mal formado!

download-request-failed = O pedido de transferência para '{ $uri }' falhou

download-stream-failed = Não foi possível receber o fluxo de dados de '{ $uri }'

download-get = Não foi possível enviar o pedido de transferência

download-content-length = content-length não existe em { $headers }

download-content-str = A conversão de content-length numa cadeia de caracteres falhou: { $headers }

download-content-parse = A análise de content-length falhou: { $headers }

download-checksum = A soma de verificação de { $file } não corresponde

download-no-uris = Não foi possível transferir { $file } de nenhum URI

download-no-space = Não há espaço livre suficiente em { $path }: { $required } necessário, { $available } disponível.

download-filename = { $path } não tem um nome de ficheiro válido!

download-error = Erro: { $error }

download-select-domain = A selecionar { $domain } para { $file }

download-start = A iniciar: { $uri }, Novas tentativas: { $retries }

download-finished = Concluído: { $uri }

progress-nothing = Nada a obter

progress-label = Progresso

progress-total = Total

progress-data = Dados

progress-packages = Pacotes

progress-package = Pacote

progress-status = Estado

progress-mirrors = Espelhos

progress-speed = Velocidade

progress-elapsed = Decorrido

progress-finished = Concluído:

progress-update-title = A atualizar listas de pacotes

progress-download-title = A transferir pacotes

progress-install-title = A instalar pacotes

progress-score-title = A testar espelhos

progress-last-completed = Último concluído

progress-connections = Ligações

progress-average = Média

progress-active = Ativo

progress-state = Estado

progress-starting = a iniciar

progress-downloading = a transferir

progress-idle = inativo

progress-more-mirrors = +{ $mirrors } espelhos adicionais · { $connections } ligações ativas

tui-upgrade-title = Atualização Nala

tui-history-title = Histórico Nala

tui-help-move = (↑) subir | (↓) descer

tui-help-tabs = (→) separador seguinte | (←) separador anterior

tui-help-show = (Enter) mostrar registo de alterações | (s) mostrar informações da versão

tui-help-confirm = (q) sair | (y) iniciar atualização

tui-help-quit = (q) sair

tui-changelog-missing = Não foi possível encontrar o URI do registo de alterações

file-remove = Falha ao remover { $path }

file-read = Falha ao ler '{ $path }'

file-parse-defaults = Falha ao analisar { $path }, a utilizar valores predefinidos

config-parse = Falha ao analisar { $path }. O Nala não irá ignorar um ficheiro de configuração existente. Se isto ocorreu após uma atualização de pacote, compare-o com { $path }.dpkg-dist

file-create = Não foi possível criar '{ $path }'

file-write = Não foi possível escrever '{ $path }'

file-replace = Não foi possível substituir '{ $path }'

file-deserialize = Não foi possível desserializar '{ $path }'

fs-failed = Falha na operação { $operation } sobre { $path }

fs-failed-target = Falha na operação { $operation }: { $path } => { $target }

config-option = A opção '{ $option }' não é suportada

color-expected = um nome de cor, um índice de 0 a 255 ou uma cadeia hexadecimal #RRGGBB

color-index = o índice de cor tem de estar entre 0 e 255

color-unknown = cor desconhecida '{ $color }'

color-rgb-components = Rgb requer três componentes

color-rgb-expected = era esperada uma cor Rgb ou Indexed

color-modifier-expected = uma cadeia de modificadores como "BOLD | ITALIC" ou um vetor

color-modifier-unknown = modificador desconhecido '{ $modifier }'

archive-unsupported = O tipo de arquivo não é suportado

deb-control-missing = ficheiro control não encontrado em { $path }

hash-unavailable =
    Não é possível verificar a integridade de { $package } { $version }.
    Não há hashes disponíveis para este pacote.

dpkg-child-failed = O processo filho do dpkg falhou: { $error }

dpkg-status-utf8 = O descritor de estado do dpkg devolveu UTF-8 inválido

dpkg-exit = O dpkg terminou com o código: '{ $code }'

dpkg-poll = Não foi possível consultar o processo filho

dpkg-read-status = Não foi possível ler o descritor de estado

dpkg-read-pty = Não foi possível ler do pseudoterminal

dpkg-write-pty = Não foi possível enviar a entrada padrão para o pseudoterminal

show-no-summary = Sem resumo

show-size = Tamanho

show-sha256 = SHA256

show-archive = Arquivo

show-codename = Nome de código

show-component = Componente

show-attributes = Atributos

show-obsoletes = Torna obsoleto

show-attr-local = Local

show-attr-auto-removable = Removível automaticamente

show-attr-automatic = Automático

policy-none = nenhum

policy-candidate = Candidato:

policy-version-table = Tabela de versões:

policy-no-versions = Sem versões.

policy-origin = origem

policy-package-files = Ficheiros de pacotes:

policy-pinned = Pacotes fixados:

policy-release = lançamento

history-cleared =
    { $count ->
        [one] Foi eliminada { $count } entrada do histórico.
       *[other] Foram eliminadas { $count } entradas do histórico.
    }

history-not-replayable = A entrada do histórico '{ $id }' não pode ser repetida porque não foi registada como aplicada

history-no-changes = A entrada do histórico '{ $id }' não tem alterações de pacotes para repetir

history-cleared-entry = Entrada do histórico { $id } eliminada.

history-id = ID

history-command = Comando

history-date-time = Data e hora

history-requested-by = Pedido por

history-altered = Alterado

history-status = Estado

history-status-applied = Aplicado

history-started = Iniciado

history-finished = Concluído

history-targets = Alvos pedidos

history-undo-version-missing = Não é possível desfazer '{ $package }' porque a versão anterior não foi registada

history-undo-config-only = Não é possível desfazer '{ $package }' porque a restauração do estado com apenas ficheiros de configuração não está implementada

history-undo-installed-version-missing = Não é possível desfazer '{ $package }' porque a versão anteriormente instalada não foi registada

history-undo-reinstall = Não é possível desfazer '{ $package }' porque a reinstalação não tem uma operação inversa registada

history-undo-held = Não é possível desfazer o pacote retido '{ $package }'

history-redo-version-missing = Não é possível refazer '{ $package }' porque a versão resultante não foi registada

history-redo-reinstall-missing = Não é possível refazer '{ $package }' porque a versão reinstalada não foi registada

history-redo-held = Não é possível refazer o pacote retido '{ $package }'

history-clear-target = A limpeza do histórico exige um seletor de entrada ou --all

history-config-undo = Não é possível desfazer o pacote configurado '{ $package }'

history-config-redo = Não é possível refazer o pacote configurado '{ $package }'

history-serialize = Não foi possível serializar a entrada do histórico

fetch-none-selected = Não foram selecionados espelhos.

fetch-unsupported = { $distro } não é suportado.

fetch-title = Seleção de espelhos Nala

fetch-score = Pontuação:

fetch-help = Utilize ↓↑ para navegar, Espaço para selecionar/desselecionar, Home/End para ir ao início/fim e q/Enter para sair.

update-item = { $state }: { $description }

upgrade-exclude-unsafe =
    Os pacotes selecionados não podem ser excluídos da atualização em segurança.
    { $error }

upgrade-protect = A proteger { $package } de { $reason }

upgrade-reason-upgrade = atualização

upgrade-reason-auto-remove = remoção automática

upgrade-config-missing = Não há árvore de configuração!
