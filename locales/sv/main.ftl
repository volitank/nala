### Messages migrated from legacy/python/po/sv.po.
### Missing messages fall back to English.

log-error = Fel:

log-notice = Notis:

log-warning = Varning:

prompt-continue = Vill du fortsätta?

prompt-choice = J/j N/n

root-required = Nala behöver root för att utföra { $command }

op-auto-purge = Automatisk rensa

op-auto-remove = Ta bort automatiskt

op-remove = Ta bort

op-purge = Rensa

op-install = Installera

op-reinstall = Ominstallerar

op-upgrade = Uppgradera

op-downgrade = Nergradera

op-configure = Konfigurerar

op-held = Hålls tillbaka

reason-kept-back = Hålls tillbaka

summary-title = Sammanfattning

summary-total-download = Total nedladdningsstorlek:

summary-disk-required = Diskutrymme som krävs:

summary-disk-free = Diskutrymme att frigöra:

summary-total-download-value = Total nedladdningsstorlek: { $size }

summary-disk-required-value = Diskutrymme som krävs: { $size }

summary-disk-free-value = Diskutrymme att frigöra: { $size }

summary-nothing = Inget för Nala att göra.

summary-essential = Följande paket är viktiga!

summary-remove-essential = Du har försökt ta bort viktiga paket

summary-use-switch = Använd { $switch } om du är säker på att du vill.

summary-reboot = En omstart krävs.

summary-reboot-packages = Följande paket kräver en omstart.

table-package = Paket:

table-version = Version:

table-size = Storlek:

table-old-version = Gammal version:

table-new-version = Ny version:

pkg-no-candidate = { $package } har ingen installationskandidat.

pkg-not-installed = { $package } är inte installerat

pkg-already-latest = { $package } är redan den senaste versionen { $version }

pkg-version-missing = Version { $version } hittades inte för paketet { $package }

pkg-cache-missing = { $package } men det finns inte i cachen

pkg-recorded-version-missing = kan inte hitta version för { $package }

pkg-version-cache-missing = Version { $version } hittades inte för paketet { $package }

pkg-virtual-providers = { $package } är ett virtuellt paket som tillhandahålls av:

pkg-virtual-one = Du ska välja bara ett.

download-complete = Nedladdning Klar:

download-auth-warning = Följande paket kan inte autentiseras!

download-auth-allowed = Konfigurationen är inställd att tillåta installation av oautentiserade paket.

download-exit = Avslutar på din begäran.

progress-working = [Arbetar]

progress-remaining = Återstående tid

progress-fetched = { $size } på { $time } ({ $rate }/s)

tui-pkgs = Paket

file-read-defaults = Det gick inte att läsa konfigurationsfilen: { $path }. Använder standardinställningar

hash-unsupported = Hashtypen '{ $type }' stöds inte

dpkg-removing = Tar bort:

dpkg-unpacking = Packar upp:

dpkg-setting-up = Ställer in:

dpkg-processing = Bearbetar:

list-virtual = Virtuellt paket

show-local-install = lokal installation

show-more-records = Det finns { $count } ytterligare poster. Använd växla { $switch } för att se dem.

show-no-description = Ingen beskrivning

show-package = Paket

show-version = Version

show-architecture = Arkitektur

show-priority = Prioritet

show-essential = Viktig

show-section = Sektion

show-source = Källa

show-installed-size = Installerad storlek

show-maintainer = Underhållare

show-original-maintainer = Original-Underhållare

show-homepage = Hemsida

show-origin = Ursprung

show-provides = Ger

show-description = Beskrivning

show-apt-sources = APT-källor

show-depends = Beroenden

show-pre-depends = För-beroenden

show-suggests = Föreslår

show-recommends = Rekommenderar

show-conflicts = Konflikter

show-replaces = Ersätter

show-breaks = Har sönder

show-enhances = Förbättrar

show-attr-installed = Installerad

show-attr-upgradable-to = är installerat och kan uppgraderas till { $version }

show-attr-upgradable-from = kan uppgraderas från { $version }

policy-installed = Installerad:

history-empty = Ingen historik existerar.

history-not-found = Transaktion { $id } existerar inte.

fetch-no-mirrors = Nala kunde inte hitta några speglar.

fetch-release-detect = Det uppstod ett problem med att identifiera release.

fetch-sources-written = Källor har skrivits till { $file }

fetch-score-help = Poäng är hur många millisekunder det tar att ladda ner Release-filen

update-upgradable = { $count } paket kan uppgraderas. Kör '{ $command }' för att se dem.

update-no-change = Ingen ändring

update-updated = Uppdaterad

update-ignored = Ignorerad

update-downloading = Laddar ner

update-processing = Bearbetar

install-downloaded = Nedladdat: { $path }

## Additional messages

unknown = Okänd

none = Ingen

error = Fel

mirrors = Speglar:

log-info = Information:

log-verbose = Utförligt:

log-debug = Felsökning:

not-implemented = Inte implementerat ännu.

subcommand-missing = Inget underkommando angivet

prompt-invalid = '{ $response }' är inte ett giltigt svar

prompt-refused = Användaren avböjde bekräftelsen

release-info-title = Förrådsinformationen har ändrats

release-info-explanation =
    Ett förråds identifierande information har ändrats i dess Release-fil.
    Granska ändringarna innan du fortsätter.

release-info-repository = Förråd

release-info-distribution = Distribution

release-info-changes = Ändringar

release-info-confirm = Godkänna ändringarna och fortsätta uppdateringen?

release-info-help = ({ $yes }) godkänn | ({ $no }/Enter/Esc) avvisa

release-info-accepted = Ändringarna av förrådsinformationen för { $repository } godkändes.

release-info-rejected = Ändringarna av förrådsinformationen för { $repository } avvisades.

release-info-noninteractive = Ändringar av Release-information måste godkännas uttryckligen. Kör igen med '-o Acquire::AllowReleaseInfoChange=true' för att godkänna dem.

reason-excluded = Utesluten

reason-manual-hold = Manuellt tillbakahållen

reason-phased = Stegvis

reason-phased-percent = Stegvis { $percentage }%

summary-op-count = { $operation } { $count }

summary-op-count-colon = { $operation }: { $count }

summary-protected = Följande paket är nödvändiga eller skyddade!

summary-remove-protected = Du försökte ta bort nödvändiga eller skyddade paket

summary-downgrade-assume-yes = Paket skulle nedgraderas med --assume-yes. Använd '-o APT::Get::allow-downgrades=true' om du är säker.

summary-held-assume-yes = Tillbakahållna paket skulle ändras med --assume-yes. Använd '-o APT::Get::allow-change-held-packages=true' om du är säker.

autoremove-config-purge = --remove-config måste användas med --purge eller autopurge.

autoremove-assume-yes = Hoppar över automatisk borttagning med --assume-yes. Använd --autoremove för att aktivera den.

table-reason = Orsak:

pkg-not-found = '{ $package }' hittades inte

pkg-not-found-cache = Vissa paket hittades inte i cachen

pkg-mark-install = Kunde inte markera '{ $package }' för installation

pkg-reinstall-missing = { $package } är inte installerat och kan därför inte installeras om

pkg-invalid-name = Ogiltigt paketnamn: '{ $package }'

pkg-op-conflict = Motstridiga åtgärder för '{ $package }': { $operation-a } mot { $operation-b }

pkg-version-conflict = Motstridiga låsta versioner för '{ $package }': { $version-a } mot { $version-b }

pkg-invalid-pin = Ogiltig versionslåsning: '{ $pin }'

pkg-pin-exact = Versionslåsning kräver ett exakt paketnamn: '{ $pin }'

pkg-exclude-missing = Uteslutningsmönstret '{ $pattern }' matchade inga paket

pkg-virtual-no-providers = { $package } har inga leverantörer och är helt virtuellt

pkg-virtual-select = Väljer { $provider } i stället för det virtuella paketet { $package }

pkg-select = Väljer paketet '{ $selected }' i stället för '{ $requested }'

pkg-fix-broken = Kunde inte korrigera trasiga paket

pkg-required = Du måste ange ett paket

download-failed = Hämtningarna misslyckades

download-some-missing = Vissa paket hittades inte.

download-auth-required =
    Vissa paket kunde inte autentiseras.
    Använd { $switch } om du är säker

download-items = Objekt

download-source-missing = Kan inte hitta en källa för att hämta version '{ $version }' av '{ $package }'

download-written = { $package } skrevs till { $path }

download-hash-missing = Ingen hash hittades för '{ $uri }'

download-protocol-missing = Inget protokoll definierades

download-uri-missing = Ingen URI definierades

download-malformed = '{ $uri }' är felaktigt utformad!

download-request-failed = Hämtningsbegäran för '{ $uri }' misslyckades

download-stream-failed = Kunde inte strömma data från '{ $uri }'

download-get = Kunde inte skicka hämtningsbegäran

download-content-length = content-length saknas i { $headers }

download-content-str = Kunde inte konvertera content-length till en sträng: { $headers }

download-content-parse = Kunde inte tolka content-length: { $headers }

download-checksum = Kontrollsumman stämde inte för { $file }

download-no-uris = Kunde inte hämta { $file } från någon URI

download-no-space = Inte tillräckligt med ledigt utrymme i { $path }: { $required } krävs, { $available } tillgängligt.

download-filename = { $path } har inget giltigt filnamn!

download-error = Fel: { $error }

download-select-domain = Väljer { $domain } för { $file }

download-start = Startar: { $uri }, Nya försök: { $retries }

download-finished = Slutfört: { $uri }

progress-nothing = Inget att hämta

progress-label = Förlopp

progress-total = Totalt

progress-data = Data

progress-packages = Paket

progress-package = Paket

progress-status = Status

progress-mirrors = Speglar

progress-speed = Hastighet

progress-elapsed = Förfluten tid

progress-finished = Slutfört:

progress-update-title = Uppdaterar paketlistor

progress-download-title = Hämtar paket

progress-install-title = Installerar paket

progress-score-title = Testar speglar

progress-last-completed = Senast slutfört

progress-connections = Anslutningar

progress-average = Genomsnitt

progress-active = Aktiv

progress-state = Tillstånd

progress-starting = startar

progress-downloading = hämtar

progress-idle = inaktiv

progress-more-mirrors = +{ $mirrors } ytterligare speglar · { $connections } aktiva anslutningar

tui-upgrade-title = Nala-uppgradering

tui-history-title = Nala-historik

tui-help-move = (↑) flytta upp | (↓) flytta ned

tui-help-tabs = (→) nästa flik | (←) föregående flik

tui-help-show = (Enter) visa ändringslogg | (s) visa versionsinformation

tui-help-confirm = (q) avsluta | (y) starta uppgraderingen

tui-help-quit = (q) avsluta

tui-changelog-missing = Kunde inte hitta ändringsloggens URI

file-remove = Kunde inte ta bort { $path }

file-read = Kunde inte läsa '{ $path }'

file-parse-defaults = Kunde inte tolka { $path }, använder standardvärden

config-parse = Kunde inte tolka { $path }. Nala ignorerar inte en befintlig konfigurationsfil. Om detta inträffade efter en paketuppdatering, jämför den med { $path }.dpkg-dist

file-create = Kunde inte skapa '{ $path }'

file-write = Kunde inte skriva '{ $path }'

file-replace = Kunde inte ersätta '{ $path }'

file-deserialize = Kunde inte deserialisera '{ $path }'

fs-failed = Åtgärden { $operation } för { $path } misslyckades

fs-failed-target = Åtgärden { $operation } misslyckades: { $path } => { $target }

config-option = Alternativet '{ $option }' stöds inte

color-expected = ett färgnamn, ett index mellan 0 och 255 eller en hexadecimal sträng #RRGGBB

color-index = färgindex måste vara mellan 0 och 255

color-unknown = okänd färg '{ $color }'

color-rgb-components = Rgb kräver tre komponenter

color-rgb-expected = en Rgb- eller Indexed-färg förväntades

color-modifier-expected = en modifierarsträng som "BOLD | ITALIC" eller en array

color-modifier-unknown = okänd modifierare '{ $modifier }'

archive-unsupported = Arkivtypen stöds inte

deb-control-missing = control-filen hittades inte i { $path }

hash-unavailable =
    Integriteten för { $package } { $version } kan inte kontrolleras.
    Det finns inga hashar för detta paket.

dpkg-child-failed = Dpkg:s barnprocess misslyckades: { $error }

dpkg-status-utf8 = Dpkg:s statusfildeskriptor returnerade ogiltig UTF-8

dpkg-exit = Dpkg avslutades med koden: '{ $code }'

dpkg-poll = Kunde inte avfråga barnprocessen

dpkg-read-status = Kunde inte läsa statusfildeskriptorn

dpkg-read-pty = Kunde inte läsa från pseudoterminalen

dpkg-write-pty = Kunde inte skicka standardindata till pseudoterminalen

show-no-summary = Ingen sammanfattning

show-size = Storlek

show-sha256 = SHA256

show-archive = Arkiv

show-codename = Kodnamn

show-component = Komponent

show-attributes = Attribut

show-obsoletes = Gör föråldrad

show-attr-local = Lokal

show-attr-auto-removable = Kan tas bort automatiskt

show-attr-automatic = Automatisk

policy-none = ingen

policy-candidate = Kandidat:

policy-version-table = Versionstabell:

policy-no-versions = Inga versioner.

policy-origin = ursprung

policy-package-files = Paketfiler:

policy-pinned = Låsta paket:

policy-release = utgåva

history-cleared =
    { $count ->
        [one] Rensade { $count } historikpost.
       *[other] Rensade { $count } historikposter.
    }

history-not-replayable = Historikposten '{ $id }' kan inte upprepas eftersom den inte registrerades som tillämpad

history-no-changes = Historikposten '{ $id }' har inga paketändringar att upprepa

history-cleared-entry = Rensade historikpost { $id }.

history-id = ID

history-command = Kommando

history-date-time = Datum och tid

history-requested-by = Begärt av

history-altered = Ändrad

history-status = Status

history-status-applied = Tillämpad

history-started = Startad

history-finished = Slutförd

history-targets = Begärda mål

history-undo-version-missing = Ångra stöds inte för '{ $package }' eftersom den föregående versionen inte registrerades

history-undo-config-only = Ångra stöds inte för '{ $package }' eftersom återställning till ett tillstånd med endast konfigurationsfiler inte är implementerad

history-undo-installed-version-missing = Ångra stöds inte för '{ $package }' eftersom den tidigare installerade versionen inte registrerades

history-undo-reinstall = Ångra stöds inte för '{ $package }' eftersom ominstallationen inte har någon registrerad omvänd åtgärd

history-undo-held = Det tillbakahållna paketet '{ $package }' kan inte ångras

history-redo-version-missing = Gör om stöds inte för '{ $package }' eftersom den resulterande versionen inte registrerades

history-redo-reinstall-missing = Gör om stöds inte för '{ $package }' eftersom den ominstallerade versionen inte registrerades

history-redo-held = Det tillbakahållna paketet '{ $package }' kan inte göras om

history-clear-target = Rensning av historiken kräver en postväljare eller --all

history-config-undo = Det konfigurerade paketet '{ $package }' kan inte ångras

history-config-redo = Det konfigurerade paketet '{ $package }' kan inte göras om

history-serialize = Kunde inte serialisera historikposten

fetch-none-selected = Inga speglar valdes.

fetch-unsupported = { $distro } stöds inte.

fetch-title = Nala-spegelval

fetch-score = Poäng:

fetch-help = Använd ↓↑ för att flytta, mellanslag för att välja/avmarkera, Home/End för att gå till början/slutet och q/Enter för att avsluta.

update-item = { $state }: { $description }

upgrade-exclude-unsafe =
    Valda paket kan inte uteslutas från uppgraderingen på ett säkert sätt.
    { $error }

upgrade-protect = Skyddar { $package } från { $reason }

upgrade-reason-upgrade = uppgradering

upgrade-reason-auto-remove = automatisk borttagning

upgrade-config-missing = Inget konfigurationsträd!
