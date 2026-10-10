### Messages migrated from legacy/python/po/pl.po.
### Missing messages fall back to English.

log-error = Błąd:

log-notice = Informacja:

log-warning = Ostrzeżenie:

prompt-continue = Czy chcesz kontynuować?

prompt-choice = Y/y N/n

root-required = Nala potrzebuje uprawnień roota by wykonać { $command }

op-auto-purge = Automatycznie wyczyść

op-auto-remove = Automatycznie usuń

op-remove = Usuń

op-purge = Wyczyść

op-install = Zainstaluj

op-reinstall = Przeinstaluj

op-upgrade = Zaktualizuj

op-downgrade = Starsza wersja

op-configure = Konfiguruj

summary-title = Podsumowanie

summary-total-download = Ilość pobranych danych:

summary-disk-required = Wymagane miejsce na dysku:

summary-disk-free = Zostanie zwolnione miejsce na dysku:

summary-total-download-value = Ilość pobranych danych: { $size }

summary-disk-required-value = Wymagane miejsce na dysku: { $size }

summary-disk-free-value = Zostanie zwolnione miejsce na dysku: { $size }

summary-nothing = Nala nie ma nic do roboty.

summary-essential = Te pakiety są kluczowe!

summary-remove-essential = Próbowałeś usunąć kluczowe pakiety

summary-use-switch = Użyj { $switch } jeśli masz pewność co robisz.

summary-reboot = Wymagane jest ponowne uruchomienie

summary-reboot-packages = Następujące pakiety wymagają ponownego uruchomienia systemu.

table-package = Pakiet:

table-version = Wersja:

table-size = Rozmiar:

table-old-version = Stara wersja:

table-new-version = Nowa wersja:

pkg-no-candidate = { $package } nie ma kandydata do instalacji

pkg-not-installed = { $package } nie jest zainstalowany

pkg-already-latest = { $package } jest już w najnowszej wersji { $version }

pkg-version-missing = Nie znaleziono wersji { $version } pakietu { $package }

pkg-cache-missing = { $package } ale nie ma go w buforze

pkg-recorded-version-missing = Nie można znaleźć wersji pakietu { $package }

pkg-version-cache-missing = Nie znaleziono wersji { $version } pakietu { $package }

pkg-virtual-providers = { $package } to pakiet wirtualny, dostarczany przez:

pkg-virtual-one = Musisz wybrać tylko jeden.

download-complete = Pobieranie zakończone:

download-auth-warning = Następujące pakiety nie mogą zostać uwierzytelnione!

download-auth-allowed = Konfiguracja zezwala na instalację nieuwierzytelnionych pakietów.

download-exit = Zakończone na twoje życzenie.

progress-remaining = Pozostały czas

tui-pkgs = Pakiety

file-read-defaults = Nie można odczytać pliku konfiguracyjnego: { $path }. Używam ustawień domyślnych

dpkg-removing = Usuwane:

dpkg-unpacking = Rozpakowywanie:

dpkg-setting-up = Konfigurowanie:

dpkg-processing = Przetwarzanie:

list-virtual = Pakiet wirtualny

show-local-install = Instalacja lokalna

show-more-records = Jest jeszcze { $count } dodatkowych wpisów. Użyj { $switch }, żeby zobaczyć wszystkie.

show-no-description = Brak opisu

show-package = Pakiet

show-version = Wersja

show-architecture = Architektura

show-priority = Priorytet

show-essential = Kluczowe

show-section = Sekcja

show-source = Źródło

show-installed-size = Rozmiar po zainstalowaniu

show-maintainer = Opiekun

show-original-maintainer = Opiekun oryginału

show-homepage = Strona domowa

show-origin = Pochodzenie

show-provides = Dostarcza

show-description = Opis

show-apt-sources = APT-Sources

show-depends = Zależności

show-pre-depends = Zależy od

show-suggests = Sugerowane

show-recommends = Rekomendowane

show-conflicts = Konflikty

show-replaces = Zamienia

show-breaks = Koliduje z

show-enhances = Rozszerza

show-attr-installed = Zainstalowane

show-attr-upgradable-to = Jest zainstalowany, możliwy do aktualizacji do wersji { $version }

show-attr-upgradable-from = można aktualizować z wersji { $version }

policy-installed = Zainstalowane:

history-empty = Brak historii.

history-not-found = Operacja { $id } nie istnieje.

fetch-release-detect = Wystąpił problem z wykryciem numeru wydania

fetch-sources-written = Źródła zostały zapisane w pliku { $file }

update-no-change = Bez zmian

update-updated = Zaktualizowane

update-ignored = Pominięte

update-downloading = Pobieranie

update-processing = Przetwarzanie

install-downloaded = Pobrane: { $path }

## Additional messages

unknown = Nieznane

none = Brak

error = Błąd

mirrors = Serwery lustrzane:

log-info = Informacja:

log-verbose = Szczegóły:

log-debug = Debugowanie:

not-implemented = Jeszcze nie zaimplementowano.

subcommand-missing = Nie podano podpolecenia

prompt-invalid = „{ $response }” nie jest prawidłową odpowiedzią

prompt-refused = Użytkownik odmówił potwierdzenia

prompt-choice-no = [y/N]

release-info-title = Zmieniono informacje o repozytorium

release-info-explanation =
    Informacje identyfikujące repozytorium zmieniły się w jego pliku Release.
    Przejrzyj zmiany przed kontynuowaniem.

release-info-repository = Repozytorium

release-info-distribution = Dystrybucja

release-info-changes = Zmiany

release-info-confirm = Zaakceptować te zmiany i kontynuować aktualizację?

release-info-help = (y) zaakceptuj | (n/Enter/Esc) odrzuć

release-info-accepted = Zaakceptowano zmiany informacji o repozytorium { $repository }.

release-info-rejected = Odrzucono zmiany informacji o repozytorium { $repository }.

release-info-noninteractive = Zmiany informacji Release wymagają jawnej akceptacji. Uruchom ponownie z opcją „-o Acquire::AllowReleaseInfoChange=true”, aby je zaakceptować.

op-held = Wstrzymane

reason-excluded = Wykluczone

reason-manual-hold = Ręcznie wstrzymane

reason-phased = Etapowe

reason-kept-back = Pozostawione bez aktualizacji

reason-phased-percent = Etapowe { $percentage }%

summary-op-count = { $operation } { $count }

summary-op-count-colon = { $operation }: { $count }

summary-protected = Następujące pakiety są niezbędne lub chronione!

summary-remove-protected = Próbowano usunąć niezbędne lub chronione pakiety

summary-downgrade-assume-yes = Opcja --assume-yes spowodowałaby obniżenie wersji pakietów. Użyj „-o APT::Get::allow-downgrades=true”, jeśli masz pewność.

summary-held-assume-yes = Opcja --assume-yes spowodowałaby zmianę wstrzymanych pakietów. Użyj „-o APT::Get::allow-change-held-packages=true”, jeśli masz pewność.

autoremove-config-purge = Opcji --remove-config należy używać z --purge lub autopurge.

autoremove-assume-yes = Pomijanie automatycznego usuwania przy --assume-yes. Użyj --autoremove, aby je włączyć.

table-reason = Powód:

pkg-not-found = Nie znaleziono „{ $package }”

pkg-not-found-cache = Nie znaleziono niektórych pakietów w pamięci podręcznej

pkg-mark-install = Nie można oznaczyć „{ $package }” do instalacji

pkg-reinstall-missing = Pakiet { $package } nie jest zainstalowany, więc nie można go przeinstalować

pkg-invalid-name = Nieprawidłowa nazwa pakietu: „{ $package }”

pkg-op-conflict = Sprzeczne operacje dla „{ $package }”: { $operation-a } i { $operation-b }

pkg-version-conflict = Sprzeczne przypięte wersje dla „{ $package }”: { $version-a } i { $version-b }

pkg-invalid-pin = Nieprawidłowe przypięcie wersji: „{ $pin }”

pkg-pin-exact = Przypięcie wersji wymaga dokładnej nazwy pakietu: „{ $pin }”

pkg-exclude-missing = Wzorzec wykluczenia „{ $pattern }” nie pasuje do żadnego pakietu

pkg-virtual-no-providers = Pakiet { $package } jest wyłącznie wirtualny i żaden pakiet go nie dostarcza

pkg-virtual-select = Wybieranie { $provider } zamiast pakietu wirtualnego { $package }

pkg-select = Wybieranie pakietu „{ $selected }” zamiast „{ $requested }”

pkg-fix-broken = Nie można naprawić uszkodzonych pakietów

pkg-required = Należy podać pakiet

download-failed = Pobieranie nie powiodło się

download-some-missing = Nie znaleziono niektórych pakietów.

download-auth-required =
    Nie można było uwierzytelnić niektórych pakietów.
    Jeśli masz pewność, użyj { $switch }

download-items = Elementy

download-source-missing = Nie można znaleźć źródła do pobrania wersji „{ $version }” pakietu „{ $package }”

download-written = Pakiet { $package } zapisano w { $path }

download-hash-missing = Nie znaleziono skrótu dla „{ $uri }”

download-protocol-missing = Nie zdefiniowano protokołu

download-uri-missing = Nie zdefiniowano URI

download-malformed = „{ $uri }” ma nieprawidłowy format!

download-request-failed = Żądanie pobrania „{ $uri }” nie powiodło się

download-stream-failed = Nie można strumieniować danych z „{ $uri }”

download-get = Nie można wysłać żądania pobrania

download-content-length = W { $headers } nie ma content-length

download-content-str = Nie można przekształcić content-length w ciąg znaków: { $headers }

download-content-parse = Nie można przeanalizować content-length: { $headers }

download-checksum = Suma kontrolna pliku { $file } nie zgadza się

download-no-uris = Nie można pobrać { $file } z żadnego URI

download-no-space = Za mało wolnego miejsca w { $path }: potrzeba { $required }, dostępne { $available }.

download-filename = Ścieżka { $path } nie ma prawidłowej nazwy pliku!

download-error = Błąd: { $error }

download-select-domain = Wybieranie { $domain } dla { $file }

download-start = Rozpoczynanie: { $uri }, Ponowienia: { $retries }

download-finished = Zakończono: { $uri }

progress-working = Praca w toku...

progress-nothing = Nic do pobrania

progress-label = Postęp

progress-total = Łącznie

progress-data = Dane

progress-packages = Pakiety

progress-package = Pakiet

progress-status = Status

progress-mirrors = Serwery lustrzane

progress-speed = Prędkość

progress-elapsed = Upłynęło

progress-finished = Zakończono:

progress-update-title = Aktualizowanie list pakietów

progress-download-title = Pobieranie pakietów

progress-install-title = Instalowanie pakietów

progress-score-title = Testowanie serwerów lustrzanych

progress-last-completed = Ostatnio ukończone

progress-connections = Połączenia

progress-average = Średnia

progress-active = Aktywne

progress-state = Stan

progress-starting = rozpoczynanie

progress-downloading = pobieranie

progress-idle = bezczynność

progress-more-mirrors = +{ $mirrors } dodatkowych serwerów lustrzanych · { $connections } aktywnych połączeń

progress-fetched = Pobrano { $size } w { $time } ({ $rate }/s)

tui-upgrade-title = Aktualizacja Nala

tui-history-title = Historia Nala

tui-help-move = (↑) w górę | (↓) w dół

tui-help-tabs = (→) następna karta | (←) poprzednia karta

tui-help-show = (Enter) pokaż dziennik zmian | (s) pokaż informacje o wersji

tui-help-confirm = (q) wyjdź | (y) rozpocznij aktualizację

tui-help-quit = (q) wyjdź

tui-changelog-missing = Nie można znaleźć URI dziennika zmian

file-remove = Nie można usunąć { $path }

file-read = Nie można odczytać „{ $path }”

file-parse-defaults = Nie można przeanalizować { $path }, używanie wartości domyślnych

config-parse = Nie można przeanalizować { $path }. Nala nie zignoruje istniejącego pliku konfiguracji. Jeśli nastąpiło to po aktualizacji pakietu, porównaj go z { $path }.dpkg-dist

file-create = Nie można utworzyć „{ $path }”

file-write = Nie można zapisać „{ $path }”

file-replace = Nie można zastąpić „{ $path }”

file-deserialize = Nie można zdeserializować „{ $path }”

fs-failed = Operacja { $operation } dla { $path } nie powiodła się

fs-failed-target = Operacja { $operation } nie powiodła się: { $path } => { $target }

config-option = Opcja „{ $option }” nie jest obsługiwana

color-expected = nazwa koloru, indeks od 0 do 255 lub ciąg szesnastkowy #RRGGBB

color-index = indeks koloru musi być pomiędzy 0 a 255

color-unknown = nieznany kolor „{ $color }”

color-rgb-components = Rgb wymaga trzech składowych

color-rgb-expected = oczekiwano koloru Rgb lub Indexed

color-modifier-expected = ciąg modyfikatorów, taki jak "BOLD | ITALIC", lub tablica

color-modifier-unknown = nieznany modyfikator „{ $modifier }”

archive-unsupported = Typ archiwum nie jest obsługiwany

deb-control-missing = Nie znaleziono pliku control w { $path }

hash-unsupported = Typ skrótu { $type } nie jest obsługiwany

hash-unavailable =
    Nie można sprawdzić integralności { $package } { $version }.
    Dla tego pakietu nie ma dostępnych skrótów.

dpkg-child-failed = Proces potomny dpkg zakończył się niepowodzeniem: { $error }

dpkg-status-utf8 = Deskryptor stanu dpkg zwrócił nieprawidłowy UTF-8

dpkg-exit = Dpkg zakończył się z kodem: „{ $code }”

dpkg-poll = Nie można sprawdzić procesu potomnego

dpkg-read-status = Nie można odczytać deskryptora stanu

dpkg-read-pty = Nie można odczytać pseudoterminala

dpkg-write-pty = Nie można wysłać standardowego wejścia do pseudoterminala

show-no-summary = Brak podsumowania

show-size = Rozmiar

show-sha256 = SHA256

show-archive = Archiwum

show-codename = Nazwa kodowa

show-component = Komponent

show-attributes = Atrybuty

show-obsoletes = Zastępuje przestarzałe

show-attr-local = Lokalny

show-attr-auto-removable = Do automatycznego usunięcia

show-attr-automatic = Automatyczny

policy-none = brak

policy-candidate = Kandydat:

policy-version-table = Tabela wersji:

policy-no-versions = Brak wersji.

policy-origin = pochodzenie

policy-package-files = Pliki pakietów:

policy-pinned = Przypięte pakiety:

policy-release = wydanie

history-cleared =
    { $count ->
        [one] Usunięto { $count } wpis historii.
        [few] Usunięto { $count } wpisy historii.
        [many] Usunięto { $count } wpisów historii.
       *[other] Usunięto { $count } wpisu historii.
    }

history-not-replayable = Wpis historii „{ $id }” nie może być powtórzony, ponieważ nie został zapisany jako zastosowany

history-no-changes = Wpis historii „{ $id }” nie zawiera zmian pakietów do powtórzenia

history-cleared-entry = Usunięto wpis historii { $id }.

history-id = ID

history-command = Polecenie

history-date-time = Data i godzina

history-requested-by = Zażądane przez

history-altered = Zmienione

history-status = Status

history-status-applied = Zastosowane

history-started = Rozpoczęto

history-finished = Zakończono

history-targets = Żądane cele

history-undo-version-missing = Nie można cofnąć „{ $package }”, ponieważ poprzednia wersja nie została zapisana

history-undo-config-only = Nie można cofnąć „{ $package }”, ponieważ odtwarzanie stanu zawierającego tylko pliki konfiguracji nie jest zaimplementowane

history-undo-installed-version-missing = Nie można cofnąć „{ $package }”, ponieważ poprzednio zainstalowana wersja nie została zapisana

history-undo-reinstall = Nie można cofnąć „{ $package }”, ponieważ reinstalacja nie ma zapisanej operacji odwrotnej

history-undo-held = Nie można cofnąć wstrzymanego pakietu „{ $package }”

history-redo-version-missing = Nie można ponowić „{ $package }”, ponieważ wynikowa wersja nie została zapisana

history-redo-reinstall-missing = Nie można ponowić „{ $package }”, ponieważ przeinstalowana wersja nie została zapisana

history-redo-held = Nie można ponowić wstrzymanego pakietu „{ $package }”

history-clear-target = Czyszczenie historii wymaga wyboru wpisu lub --all

history-config-undo = Nie można cofnąć skonfigurowanego pakietu „{ $package }”

history-config-redo = Nie można ponowić skonfigurowanego pakietu „{ $package }”

history-serialize = Nie można zserializować wpisu historii

fetch-no-mirrors = Nala nie znalazła żadnych serwerów lustrzanych.

fetch-none-selected = Nie wybrano żadnych serwerów lustrzanych.

fetch-unsupported = Dystrybucja { $distro } nie jest obsługiwana.

fetch-title = Wybór serwerów lustrzanych Nala

fetch-score = Ocena:

fetch-score-help = Ocena to liczba milisekund potrzebnych na pobranie pliku Release.

fetch-help = Użyj ↓↑ do poruszania, Spacji do zaznaczania/odznaczania, Home/End do przejścia na początek/koniec oraz q/Enter do wyjścia.

update-upgradable =
    { $count ->
        [one] Można zaktualizować { $count } pakiet. Uruchom „{ $command }”, aby go zobaczyć.
        [few] Można zaktualizować { $count } pakiety. Uruchom „{ $command }”, aby je zobaczyć.
        [many] Można zaktualizować { $count } pakietów. Uruchom „{ $command }”, aby je zobaczyć.
       *[other] Można zaktualizować { $count } pakietu. Uruchom „{ $command }”, aby go zobaczyć.
    }

update-item = { $state }: { $description }

upgrade-exclude-unsafe =
    Wybranych pakietów nie można bezpiecznie wykluczyć z aktualizacji.
    { $error }

upgrade-protect = Ochrona { $package } przed { $reason }

upgrade-reason-upgrade = aktualizacja

upgrade-reason-auto-remove = automatyczne usunięcie

upgrade-config-missing = Brak drzewa konfiguracji!
