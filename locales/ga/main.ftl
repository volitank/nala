### Irish messages displayed by Nala.

## General labels and errors

unknown = Anaithnid
none = Dada
error = Earráid
mirrors = Scátháin:

log-error = Earráid:
log-notice = Fógra:
log-warning = Rabhadh:
log-info = Eolas:
log-verbose = Fadóch:
log-debug = Dífhabhtú:

not-implemented = Níor cuireadh i bhfeidhm fós.
subcommand-missing = Níor soláthraíodh fo-ordú

# Variables:
#   $response (String) - Invalid confirmation response.
prompt-invalid = Ní freagra bailí é '{ $response }'
prompt-refused = Udhiúltaigh an ser deimhniú
prompt-continue = Ar mhaith leat leanúint ar aghaidh?
prompt-choice = [T/n]

# Variables:
#   $command (String) - Command that requires elevated privileges.
root-required = Teastaíonn fréamh ó Nala le haghaidh { $command }

## Transaction operations and reasons

op-remove = Bain
op-auto-remove = Bain Uathoibríoch
op-purge = Glanadh
op-auto-purge = Glanadh Uathoibríoch
op-install = Suiteáil
op-reinstall = Athshuiteáil
op-upgrade = Uasghrádú
op-downgrade = Íosghrádú
op-configure = Cumraigh
op-held = Coinnithe

reason-excluded = Eisiata
reason-manual-hold = Sealbhú láimhe
reason-phased = Céimnithe
reason-kept-back = Coinnithe siar

# Variables:
#   $percentage (Number) - Phased update percentage.
reason-phased-percent = Céimnithe { $percentage }%

## Transaction summary

summary-title = Achoimre
summary-total-download = Méid iomlán íoslódála:
summary-disk-required = Spás diosca ag teastáil:
summary-disk-free = Spás diosca le saoradh:

summary-total-download-value = Méid iomlán íoslódála: { $size }
summary-disk-required-value = Spás diosca ag teastáil: { $size }
summary-disk-free-value = Spás diosca le saoradh: { $size }
summary-op-count = { $operation } { $count }
summary-op-count-colon = { $operation }: { $count }

summary-nothing = Ní raibh aon rud le déanamh.
summary-essential = Tá na pacáistí seo a leanas riachtanach!
summary-remove-essential = Rinne tú iarracht pacáistí riachtanacha a bhaint

# Variables:
#   $switch (String) - Lasc líne ordaithe a cheadaíonn baint pacáiste riachtanach.
summary-use-switch = Úsáid '{ $switch }' má tá tú cinnte.

summary-reboot = Tá atosú ag teastáil chun na hathruithe seo a chur i gcrích.
summary-reboot-packages = Éilíonn na pacáistí seo a leanas athbhútáil:

## Package summary table

table-package = Pacáiste:
table-version = Leagan:
table-old-version = Seanleagan:
table-new-version = Leagan Nua:
table-reason = Cúis:
table-size = Méid:

## Package selection

# Variables:
#   $package (String) - Package name or selection token.
pkg-not-found = Níor aimsíodh '{ $package }'
pkg-not-found-cache = Níor aimsíodh roinnt pacáistí sa taisce
pkg-no-candidate = Níl aon iarrthóir suiteála ag { $package }
pkg-mark-install = Ní féidir '{ $package }' a mharcáil le haghaidh suiteála
pkg-not-installed = Níl { $package } suiteáilte
pkg-reinstall-missing = Níl { $package } suiteáilte, mar sin ní féidir é a athshuiteáil
pkg-invalid-name = Ainm pacáiste neamhbhailí: '{ $package }'

# Variables:
#   $package (String) - Package name.
#   $operation-a (String) - First requested operation.
#   $operation-b (String) - Conflicting requested operation.
pkg-op-conflict = Oibríochtaí contrártha le haghaidh '{ $package }': { $operation-a } vs { $operation-b }

# Variables:
#   $package (String) - Package name.
#   $version-a (String) - First requested version.
#   $version-b (String) - Conflicting requested version.
pkg-version-conflict = Leaganacha bioráilte contrártha do '{ $package }': { $version-a } vs { $version-b }

# Variables:
#   $package (String) - Package name.
#   $version (String) - Package version.
pkg-already-latest = Tá { $package }{ $version } suiteáilte cheana féin agus ag an leagan is déanaí
pkg-version-missing = Ní féidir leagan '{ $version }' a aimsiú le haghaidh '{ $package }'
pkg-cache-missing = Níor aimsíodh an pacáiste '{ $package }' sa taisce
pkg-recorded-version-missing = Gan aon leagan taifeadta do '{ $package }'
pkg-version-cache-missing = Níor aimsíodh leagan '{ $version }' le haghaidh '{ $package }'

# Variables:
#   $pin (String) - Version pin supplied by the user.
pkg-invalid-pin = Biorán leagan neamhbhailí: '{ $pin }'
pkg-pin-exact = Éilíonn biorán leagan ainm pacáiste cruinn: '{ $pin }'

# Variables:
#   $pattern (String) - Package exclusion pattern.
pkg-exclude-missing = Níor mheaitseáil an patrún eisiata '{ $pattern }' le haon phacáistí

# Variables:
#   $package (String) - Virtual package name.
pkg-virtual-no-providers = Níl aon soláthraithe ag { $package } agus is pacáiste fíorúil amháin é
pkg-virtual-providers = Is pacáiste fíorúil é { $package } arna sholáthar ag:

# Variables:
#   $package (String) - Virtual package name.
#   $provider (String) - Selected concrete provider.
pkg-virtual-select = Ag roghnú { $provider } in ionad an phacáiste fhíorúil { $package }
pkg-virtual-one = Níor cheart duit ach ceann amháin a roghnú.

# Variables:
#   $selected (String) - Selected package name.
#   $requested (String) - Originally requested package name.
pkg-select = Ag roghnú Pacáiste '{ $selected }' in ionad '{ $requested }'

pkg-fix-broken = Ní féidir pacáistí briste a cheartú
pkg-required = Ní mór duit pacáiste a shonrú

## Download output

download-complete = Íoslódálacha Críochnaithe:
download-failed = Theip ar Íoslódálacha
download-some-missing = Níor aimsíodh roinnt pacáistí.
download-auth-warning = Ní féidir na pacáistí seo a leanas a fhíordheimhniú!
download-auth-allowed = Tá an chumraíocht socraithe chun suiteáil pacáistí neamhfhíordheimhnithe a cheadú.
download-auth-required =
    Níorbh fhéidir roinnt pacáistí a fhíordheimhniú.
      Más cinnte atá tú, bain úsáid as { $switch }
download-exit = Ag imeacht ar iarratas úsáideora
download-items = Míreanna

# Variables:
#   $package (String) - Package name.
#   $version (String) - Package version.
download-source-missing = Ní féidir foinse a aimsiú chun leagan '{ $version }' de '{ $package }' a íoslódáil

# Variables:
#   $package (String) - Package name.
#   $path (String) - Destination path.
download-written = Scríobhadh { $package } chuig { $path }

# Variables:
#   $uri (String) - Download URI.
download-hash-missing = Níor aimsíodh aon hais do '{ $uri }'
download-protocol-missing = Níor sainmhíníodh aon phrótacal
download-uri-missing = Níor sainmhíníodh aon uri
download-malformed = Tá '{ $uri }' mífhoirmithe!
download-request-failed = Theip ar iarratas íoslódála do '{ $uri }'
download-stream-failed = Ní féidir sonraí a shruthú ó '{ $uri }'
download-get = Ní féidir iarratas íoslódála a sheoladh
download-content-length = níl fad an ábhair ann i { $headers }
download-content-str = Theip ar fhad an ábhair a thiontú go teaghrán: { $headers }
download-content-parse = Theip ar fhad an ábhair a pharsáil: { $headers }

# Variables:
#   $file (String) - Downloaded filename.
download-checksum = Níorbh ionann an tsuim sheiceála do { $file }
download-no-uris = Níorbh fhéidir aon URIanna a íoslódáil do { $file }

# Variables:
#   $path (String) - Path without a valid filename.
download-filename = Níl ainm comhaid bailí ar { $path }!

download-error = Earráid: { $error }
download-select-domain = Ag roghnú { $domain } do { $file }
download-start = Ag tosú: { $uri }, Iarrachtaí: { $retries }
download-finished = Críochnaithe: { $uri }

## Progress

progress-working = Ag obair...
progress-nothing = Ní rud ar bith le faigh
progress-label = Dul chun cinn
progress-total = Iomlán
progress-speed = Luas
progress-elapsed = Imithe
progress-remaining = Fágtha
progress-finished = Críochnaithe:

# Variables:
#   $size (String) - Total amount fetched.
#   $time (String) - Elapsed time.
#   $rate (String) - Transfer rate.
progress-fetched = Fuarthas { $size } i { $time } ({ $rate }/s)

## TUI

tui-upgrade-title = Uasghrádú Nala
tui-history-title = Stair Nala
tui-pkgs = Pacáistí
tui-help-move = (↑) bog suas | (↓) bog síos
tui-help-tabs = (→) an chéad chluaisín eile | (←) an cluaisín roimhe seo
tui-help-show = (Iontráil) taispeáin loga athruithe | (s) taispeáin eolas faoin leagan
tui-help-confirm = (q) scoir | (y) tosaigh an t-uasghrádú
tui-help-quit = (q) scoir
tui-changelog-missing =  Ní féidir URI an Loga Athraithe a aimsiú

## Files, configuration, and package archives

# Variables:
#   $path (String) - File or directory path.
file-remove = Theip ar bhaint { $path }
file-read = Theip ar léamh '{ $path }'
file-read-defaults = Theip ar léamh { $path }, ag baint úsáide as réamhshocruithe
file-parse-defaults = Theip ar pharsáil { $path }, ag baint úsáide as réamhshocruithe
file-create = Ní féidir '{ $path }' a chruthú
file-write = Ní féidir scríobh chuig '{ $path }'
file-replace = Ní féidir '{ $path }' a athsholáthar
file-deserialize = Ní féidir '{ $path }' a dhíshraithiú

# Variables:
#   $operation (String) - Filesystem operation.
#   $path (String) - Source path.
fs-failed = Theip ar { $operation } { $path }

# Variables:
#   $operation (String) - Filesystem operation.
#   $path (String) - Source path.
#   $target (String) - Destination path.
fs-failed-target = Theip ar { $operation } { $path } => { $target }

# Variables:
#   $option (String) - Unsupported APT configuration override.
config-option = Ní thacaítear leis an rogha '{ $option }'
color-expected = ainm datha, innéacs 0-255, nó teaghrán heicsidheachúlach #RRGGBB
color-index = ní mór innéacs datha a bheith idir 0 agus 255
color-unknown = dath anaithnid '{ $color }'
color-rgb-components = Tá súil ag Rgb le trí chomhpháirt
color-rgb-expected = dath Rgb nó Innéacsaithe a bhíothas ag súil leis
color-modifier-expected = teaghrán modhnóra cosúil le "BOLD | ITALIC" nó eagar
color-modifier-unknown = modhnóir anaithnid '{ $modifier }'

archive-unsupported = Ní thacaítear leis an gcineál cartlainne

# Variables:
#   $path (String) - Debian archive path.
deb-control-missing = comhad rialaithe gan aimsiú i { $path }

# Variables:
#   $type (String) - Hash type.
hash-unsupported = Ní thacaítear le Cineál Haise: { $type }

# Variables:
#   $package (String) - Package name.
#   $version (String) - Package version.
hash-unavailable =
    Ní féidir sláine { $package } { $version } a sheiceáil.
    Níl aon haiseanna ar fáil don phacáiste seo.

## Dpkg

dpkg-child-failed = Theip ar leanbh dpkg: { $error }
dpkg-status-utf8 = D'fhill stádas fd dpkg UTF-8 neamhbhailí
dpkg-exit = Scoir dpkg leis an gcód: '{ $code }'
dpkg-poll = Ní féidir an leanbh a vótáil
dpkg-read-status = Ní féidir Stádas Fd a léamh
dpkg-read-pty = Ní féidir léamh ó pty
dpkg-write-pty = Ní féidir stdin a sheoladh chuig pty
dpkg-removing = Ag baint:
dpkg-unpacking = Ag díphacáil:
dpkg-setting-up = Ag socrú:
dpkg-processing = Ag próiseáil:

## List

list-virtual = Fíorúil

## Show: general output

show-local-install = suiteáil áitiúil

# Variables:
#   $count (Number) - Number of additional package records.
#   $switch (String) - Command-line switch that displays every record.
show-more-records = Tá { $count } taifead breise ann. Bain úsáid as an lasc { $switch } le do thoil chun iad a fheiceáil.

show-no-description = Gan Cur Síos
show-no-summary = Gan Achoimre

## Show: package record labels

show-package = Pacáiste
show-version = Leagan
show-architecture = Ailtireacht
show-priority = Tosaíocht
show-essential = Riachtanach
show-section = Roinn
show-source = Foinse
show-installed-size = Méid Suiteáilte
show-size = Méid
show-maintainer = Cothaitheoir
show-original-maintainer = Cothaitheoir Bunaidh
show-homepage = Leathanach Baile
show-sha256 = SHA256
show-archive = Cartlann
show-origin = Bunús
show-codename = Ainm Cód
show-component = Comhpháirt
show-provides = Soláthraíonn
show-description = Cur Síos
show-attributes = Tréithe
show-apt-sources = Foinsí APT
show-depends = Braitheann
show-pre-depends = Réamh-Braitheann
show-suggests = Molann
show-recommends = Molann
show-conflicts = Coimhlintí
show-replaces = Ionadaíonn
show-obsoletes = As feidhm
show-breaks = Sosanna
show-enhances = Feabhsaíonn

## Show: package attributes

show-attr-installed = Suiteáilte
show-attr-local = Áitiúil
show-attr-auto-removable = Uathoibríoch-Inbhainte
show-attr-automatic = Uathoibríoch

# Variables:
#   $version (String) - Package version available for upgrade.
show-attr-upgradable-to = In-uasghrádaithe go: { $version }

# Variables:
#   $version (String) - Currently installed package version.
show-attr-upgradable-from = In-uasghrádaithe ó: { $version }

## Policy

policy-none = aon cheann
policy-installed = Suiteáilte:
policy-candidate = Iarrthóir:
policy-version-table = Tábla leaganacha:
policy-no-versions = Gan leaganacha.
policy-origin = bunús
policy-package-files = Comhaid phacáiste:
policy-pinned = Pacáistí bioráilte:
policy-release = scaoileadh

## History: general output

# Variables:
#   $count (Number) - Number of history entries that were cleared.
history-cleared =
    { $count ->
        [one] Glanadh { $count } iontráil staire.
       *[other] Glanadh { $count } iontrálacha staire.
    }

history-empty = Níor aimsíodh aon iontrálacha staire.

# Variables:
#   $id (Number) - ID of the history entry.
history-not-found = Níl an iontráil staire leis an ID '{ $id }' ann
history-not-replayable = Ní féidir an iontráil staire '{ $id }' a athsheinm mar nach ndearnadh í a thaifeadadh mar a cuireadh i bhfeidhm
history-no-changes = Níl aon athruithe pacáiste le hathsheinm ag an iontráil staire '{ $id }'
history-cleared-entry = Glanadh an iontráil staire { $id }.

## History: labels

history-id = ID
history-command = Ordú
history-date-time = Dáta agus Am
history-requested-by = Iarrtha-Ag
history-altered = Athraithe
history-status = Stádas
history-status-applied = Curtha i bhFeidhm
history-started = Tosaithe
history-finished = Críochnaithe
history-targets = Spriocanna Iarrtha

## History: replay errors

# Variables:
#   $package (String) - Package name.
history-undo-version-missing = Ní thacaítear le cealú do '{ $package }' mar nach ndearnadh an leagan roimhe seo a thaifeadadh
history-undo-config-only = Ní thacaítear le cealú do '{ $package }' mar nach bhfuil athchóiriú stáit na gcomhad cumraíochta amháin curtha i bhfeidhm
history-undo-installed-version-missing = Ní thacaítear le cealú do '{ $package }' mar nach ndearnadh an leagan suiteáilte roimhe seo a thaifeadadh
history-undo-reinstall = Ní thacaítear le cealú do '{ $package }' mar nach bhfuil aon aisiompú taifeadta ag an athshuiteáil
history-undo-held = Ní féidir an pacáiste coinnithe '{ $package }' a neamhdhéanta
history-redo-version-missing = Ní thacaítear le hathdhéanamh do '{ $package }' mar nach ndearnadh an leagan a lean as a thaifeadadh
history-redo-reinstall-missing = Ní thacaítear le hathdhéanamh do '{ $package }' mar nach ndearnadh an leagan athshuiteáilte a thaifeadadh
history-redo-held = Ní féidir an pacáiste coinnithe '{ $package }' a athdhéanamh

## History: selector and storage errors

history-clear-target = Éilíonn glanadh staire roghnóir iontrála nó --all
history-config-undo = Ní féidir an pacáiste cumraithe '{ $package }' a chealú
history-config-redo = Ní féidir an pacáiste cumraithe '{ $package }' a athdhéanamh
history-serialize = Ní féidir an iontráil staire a shrathú

## Fetch

fetch-no-mirrors = Ní raibh Nala in ann aon scátháin a aimsiú.
fetch-none-selected = Níor roghnaíodh aon scátháin.
fetch-release-detect = Bhí fadhb ann an scaoileadh a bhrath.

# Variables:
#   $distro (String) - Distribution name.
fetch-unsupported = Ní thacaítear le { $distro }.

# Variables:
#   $file (String) - Sources file path.
fetch-sources-written = Scríobhadh foinsí chuig { $file }

fetch-title = Nala Fetch
fetch-score = Scór:
fetch-score-help = Is é an scór cé mhéad milleasoicind a thógann sé chun an comhad Eisiúna a íoslódáil.
fetch-help = Úsáid ↓↑ chun bogadh, Spás chun roghnú/díroghnú, Baile/Deireadh chun dul barr/bun, q/Iontráil chun imeacht.

## Update

# Variables:
#   $count (Number) - Number of upgradable packages.
#   $command (String) - Command that lists upgradable packages.
update-upgradable =
    { $count ->        
        [one] Is féidir { $count } pacáiste a uasghrádú. Rith '{ $command }' chun é a fheiceáil.       
       *[other] Is féidir { $count } pacáistí a uasghrádú. Rith '{ $command }' chun iad a fheiceáil.
    }

update-no-change = Gan Athrú
update-updated = Nuashonraithe
update-ignored = Neamhaird
update-downloading = Ag Íoslódáil
update-processing = Ag Próiseáil
update-item = { $state }: { $description }

## Upgrade

upgrade-exclude-unsafe =
    Ní féidir pacáistí roghnaithe a eisiamh ón uasghrádú go sábháilte.
    { $error }
upgrade-protect = Ag cosaint { $package } ó { $reason }
upgrade-reason-upgrade = uasghrádú
upgrade-reason-auto-remove = uath-bhaint
upgrade-config-missing = Gan crann cumraíochta!

## Install

install-downloaded = Íoslódáilte: { $path }
