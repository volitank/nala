### Messages migrated from legacy/python/po/de.po.
### Missing messages fall back to English.

log-error = Fehler:

log-notice = Notiz:

log-warning = Warnung:

prompt-continue = Möchten Sie fortfahren?

prompt-choice = J/j N/n

root-required = Nala braucht root für { $command }

op-auto-purge = automatische Bereinigung

op-auto-remove = Automatisch entfernen

op-remove = Entfernen

op-purge = Bereinigen

op-install = Installieren

op-reinstall = Neu installieren

op-upgrade = Aktualisieren

op-downgrade = Herabstufung

op-configure = Konfigurieren

op-held = Zurückgehalten

reason-kept-back = Zurückgehalten

summary-title = Zusammenfassung

summary-total-download = Gesamtgröße des Downloads:

summary-disk-required = Speicherplatz erforderlich:

summary-disk-free = Speicherplatz freizugeben:

summary-total-download-value = Gesamtgröße des Downloads: { $size }

summary-disk-required-value = Speicherplatz erforderlich: { $size }

summary-disk-free-value = Speicherplatz freizugeben: { $size }

summary-nothing = Für Nala gibt es nichts zu Nala tun.

summary-essential = Die folgenden Pakete sind essenziell!

summary-remove-essential = Sie haben versucht, essenzielle Pakete zu entfernen

summary-use-switch = Bitte verwenden Sie { $switch }, wenn Sie sicher sind, dass Sie das möchten.

summary-reboot = Ein Neustart ist erforderlich.

summary-reboot-packages = Die folgenden Pakete erfordern einen Neustart.

table-package = Paket:

table-version = Version:

table-size = Größe:

table-old-version = Alte Version:

table-new-version = Neue Version:

pkg-no-candidate = { $package } hat keinen Installationskandidaten.

pkg-not-installed = { $package } ist nicht installiert

pkg-already-latest = { $package } ist bereits in der neusten Version { $version }

pkg-version-missing = Version { $version } für Paket { $package } wurde nicht gefunden

pkg-cache-missing = { $package }, aber es ist nicht im Cache

pkg-recorded-version-missing = Version für { $package } wurde nicht gefunden

pkg-version-cache-missing = Version { $version } für Paket { $package } wurde nicht gefunden

pkg-virtual-providers = { $package } ist ein virtuelles Paket, bereitgestellt von:

download-complete = Download abgeschlossen:

download-auth-warning = Die folgenden Pakete können nicht authentifiziert werden!

download-auth-allowed = Die Konfiguration erlaubt aktuell die Installation von nicht authentifizierten Paketen.

download-exit = Beenden auf Ihren Wunsch.

progress-working = [Wird verarbeitet]

progress-remaining = Verbleibende Zeit

progress-fetched = { $size } in { $time } ({ $rate }/s)

tui-pkgs = Pakete

file-read-defaults = Konfigurationsdatei konnte nicht gelesen werden: { $path }. Es werden Standardwerte verwendet

hash-unsupported = Prüfsummentyp '{ $type }' wird nicht unterstützt

dpkg-removing = Entfernen:

dpkg-unpacking = Entpacken:

dpkg-setting-up = Konfigurieren:

dpkg-processing = Verarbeiten:

list-virtual = Virtuelles Paket

show-local-install = lokale Installation

show-more-records = Es gibt { $count } zusätzliche Einträge. Bitte verwenden Sie den Schalter { $switch }, um sie anzuzeigen.

show-no-description = Keine Beschreibung

show-package = Paket

show-version = Version

show-architecture = Architektur

show-priority = Priorität

show-essential = Essenziell

show-section = Abschnitt

show-source = Quelle

show-installed-size = Installierte Größe

show-maintainer = Betreuer

show-original-maintainer = Original-Betreuer

show-homepage = Homepage

show-origin = Herkunft

show-provides = Bietet

show-description = Beschreibung

show-apt-sources = APT-Quellen

show-depends = Hängt ab von

show-pre-depends = Vorab abhängig

show-suggests = Schlägt vor

show-recommends = empfiehlt

show-conflicts = Konflikte

show-replaces = Ersetzt

show-breaks = Beschädigt

show-enhances = Verbessert

show-attr-installed = Installiert

show-attr-upgradable-to = ist installiert und aktualisierbar auf { $version }

show-attr-upgradable-from = ist aktualisierbar von { $version }

policy-installed = Installiert:

history-empty = Es existiert kein Verlauf.

history-not-found = Transaktion { $id } existiert nicht.

fetch-no-mirrors = Nala konnte keine Spiegelserver finden.

fetch-release-detect = Bei der Erkennung der Freigabe ist ein Problem aufgetreten.

fetch-sources-written = Quellen wurden in { $file } geschrieben

fetch-score-help = Punktzahl gibt an, wie viele Millisekunden das Herunterladen der Release-Datei dauert

update-upgradable = { $count } Pakete können aktualisiert werden. Führen Sie '{ $command }' aus, um sie zu sehen.

update-no-change = Keine Änderung

update-updated = Aktualisiert

update-ignored = Ignoriert

update-downloading = Wird heruntergeladen

update-processing = Verarbeiten

install-downloaded = Heruntergeladen: { $path }

## Additional messages

unknown = Unbekannt

none = Keine

error = Fehler

mirrors = Spiegelserver:

log-info = Info:

log-verbose = Ausführlich:

log-debug = Debug:

not-implemented = Noch nicht implementiert.

subcommand-missing = Kein Unterbefehl angegeben

prompt-invalid = '{ $response }' ist keine gültige Antwort

prompt-refused = Bestätigung durch den Benutzer verweigert

prompt-choice-no = [y/N]

release-info-title = Repository-Informationen geändert

release-info-explanation =
    Die Identifikationsdaten eines Repositorys haben sich in seiner Release-Datei geändert.
    Prüfen Sie die Änderungen, bevor Sie fortfahren.

release-info-repository = Repository

release-info-distribution = Distribution

release-info-changes = Änderungen

release-info-confirm = Diese Änderungen akzeptieren und die Aktualisierung fortsetzen?

release-info-help = (y) akzeptieren | (n/Enter/Esc) ablehnen

release-info-accepted = Änderungen an den Repository-Informationen für { $repository } akzeptiert.

release-info-rejected = Änderungen an den Repository-Informationen für { $repository } abgelehnt.

release-info-noninteractive = Änderungen an Release-Informationen müssen ausdrücklich akzeptiert werden. Führen Sie den Befehl erneut mit '-o Acquire::AllowReleaseInfoChange=true' aus, um sie zu akzeptieren.

reason-excluded = Ausgeschlossen

reason-manual-hold = Manuell zurückgehalten

reason-phased = Gestaffelt

reason-phased-percent = Gestaffelt { $percentage }%

summary-op-count = { $operation } { $count }

summary-op-count-colon = { $operation }: { $count }

summary-protected = Die folgenden Pakete sind essenziell oder geschützt!

summary-remove-protected = Sie haben versucht, essenzielle oder geschützte Pakete zu entfernen

summary-downgrade-assume-yes = Mit --assume-yes würden Paketversionen herabgestuft. Verwenden Sie '-o APT::Get::allow-downgrades=true', wenn Sie sicher sind.

summary-held-assume-yes = Mit --assume-yes würden zurückgehaltene Pakete geändert. Verwenden Sie '-o APT::Get::allow-change-held-packages=true', wenn Sie sicher sind.

autoremove-config-purge = --remove-config muss mit --purge oder autopurge verwendet werden.

autoremove-assume-yes = Automatisches Entfernen mit --assume-yes wird übersprungen. Verwenden Sie --autoremove, um es zu aktivieren.

table-reason = Grund:

pkg-not-found = '{ $package }' wurde nicht gefunden

pkg-not-found-cache = Einige Pakete wurden nicht im Cache gefunden

pkg-mark-install = '{ $package }' kann nicht zur Installation vorgemerkt werden

pkg-reinstall-missing = { $package } ist nicht installiert und kann deshalb nicht erneut installiert werden

pkg-invalid-name = Ungültiger Paketname: '{ $package }'

pkg-op-conflict = Widersprüchliche Vorgänge für '{ $package }': { $operation-a } gegenüber { $operation-b }

pkg-version-conflict = Widersprüchliche festgelegte Versionen für '{ $package }': { $version-a } gegenüber { $version-b }

pkg-invalid-pin = Ungültige Versionsfestlegung: '{ $pin }'

pkg-pin-exact = Die Versionsfestlegung erfordert einen exakten Paketnamen: '{ $pin }'

pkg-exclude-missing = Das Ausschlussmuster '{ $pattern }' passt zu keinem Paket

pkg-virtual-no-providers = { $package } hat keine Anbieter und ist rein virtuell

pkg-virtual-select = { $provider } wird statt des virtuellen Pakets { $package } ausgewählt

pkg-virtual-one = Sie sollten nur eines auswählen.

pkg-select = Paket '{ $selected }' wird statt '{ $requested }' ausgewählt

pkg-fix-broken = Beschädigte Pakete können nicht korrigiert werden

pkg-required = Sie müssen ein Paket angeben

download-failed = Downloads fehlgeschlagen

download-some-missing = Einige Pakete wurden nicht gefunden.

download-auth-required =
    Einige Pakete konnten nicht authentifiziert werden.
    Wenn Sie sicher sind, verwenden Sie { $switch }

download-items = Einträge

download-source-missing = Keine Quelle zum Herunterladen von Version '{ $version }' von '{ $package }' gefunden

download-written = { $package } wurde nach { $path } geschrieben

download-hash-missing = Kein Hash für '{ $uri }' gefunden

download-protocol-missing = Kein Protokoll definiert

download-uri-missing = Keine URI definiert

download-malformed = '{ $uri }' ist fehlerhaft!

download-request-failed = Download-Anfrage für '{ $uri }' fehlgeschlagen

download-stream-failed = Daten von '{ $uri }' können nicht gestreamt werden

download-get = Download-Anfrage kann nicht gesendet werden

download-content-length = content-length fehlt in { $headers }

download-content-str = Umwandlung von content-length in eine Zeichenkette fehlgeschlagen: { $headers }

download-content-parse = Auswertung von content-length fehlgeschlagen: { $headers }

download-checksum = Prüfsumme stimmt für { $file } nicht überein

download-no-uris = Für { $file } konnte von keiner URI heruntergeladen werden

download-no-space = Nicht genügend freier Speicherplatz in { $path }: { $required } benötigt, { $available } verfügbar.

download-filename = { $path } hat keinen gültigen Dateinamen!

download-error = Fehler: { $error }

download-select-domain = { $domain } wird für { $file } ausgewählt

download-start = Start: { $uri }, Wiederholungen: { $retries }

download-finished = Abgeschlossen: { $uri }

progress-nothing = Nichts abzurufen

progress-label = Fortschritt

progress-total = Gesamt

progress-data = Daten

progress-packages = Pakete

progress-package = Paket

progress-status = Status

progress-mirrors = Spiegelserver

progress-speed = Geschwindigkeit

progress-elapsed = Vergangen

progress-finished = Abgeschlossen:

progress-update-title = Paketlisten werden aktualisiert

progress-download-title = Pakete werden heruntergeladen

progress-install-title = Pakete werden installiert

progress-score-title = Spiegelserver werden getestet

progress-last-completed = Zuletzt abgeschlossen

progress-connections = Verbindungen

progress-average = Durchschnitt

progress-active = Aktiv

progress-state = Zustand

progress-starting = startet

progress-downloading = lädt herunter

progress-idle = inaktiv

progress-more-mirrors = +{ $mirrors } weitere Spiegelserver · { $connections } aktive Verbindungen

tui-upgrade-title = Nala-Aktualisierung

tui-history-title = Nala-Verlauf

tui-help-move = (↑) nach oben | (↓) nach unten

tui-help-tabs = (→) nächster Reiter | (←) vorheriger Reiter

tui-help-show = (Enter) Änderungsprotokoll anzeigen | (s) Versionsinformationen anzeigen

tui-help-confirm = (q) beenden | (y) Aktualisierung starten

tui-help-quit = (q) beenden

tui-changelog-missing = URI des Änderungsprotokolls nicht gefunden

file-remove = Entfernen von { $path } fehlgeschlagen

file-read = Lesen von '{ $path }' fehlgeschlagen

file-parse-defaults = Auswertung von { $path } fehlgeschlagen, Standardwerte werden verwendet

config-parse = Auswertung von { $path } fehlgeschlagen. Nala ignoriert keine vorhandene Konfigurationsdatei. Falls dies nach einer Paketaktualisierung auftrat, vergleichen Sie sie mit { $path }.dpkg-dist

file-create = '{ $path }' kann nicht erstellt werden

file-write = '{ $path }' kann nicht geschrieben werden

file-replace = '{ $path }' kann nicht ersetzt werden

file-deserialize = '{ $path }' kann nicht deserialisiert werden

fs-failed = Vorgang { $operation } für { $path } fehlgeschlagen

fs-failed-target = Vorgang { $operation } fehlgeschlagen: { $path } => { $target }

config-option = Option '{ $option }' wird nicht unterstützt

color-expected = ein Farbname, ein Index von 0 bis 255 oder eine Hex-Zeichenkette #RRGGBB

color-index = Farbindex muss zwischen 0 und 255 liegen

color-unknown = unbekannte Farbe '{ $color }'

color-rgb-components = Rgb erwartet drei Komponenten

color-rgb-expected = Rgb oder Indexed als Farbe erwartet

color-modifier-expected = eine Modifikator-Zeichenkette wie "BOLD | ITALIC" oder ein Array

color-modifier-unknown = unbekannter Modifikator '{ $modifier }'

archive-unsupported = Archivtyp wird nicht unterstützt

deb-control-missing = control-Datei in { $path } nicht gefunden

hash-unavailable =
    Die Integrität von { $package } { $version } kann nicht geprüft werden.
    Für dieses Paket sind keine Hashes verfügbar.

dpkg-child-failed = Dpkg-Unterprozess fehlgeschlagen: { $error }

dpkg-status-utf8 = Dpkg-Status-Dateideskriptor lieferte ungültiges UTF-8

dpkg-exit = Dpkg wurde mit Code '{ $code }' beendet

dpkg-poll = Unterprozess kann nicht abgefragt werden

dpkg-read-status = Status-Dateideskriptor kann nicht gelesen werden

dpkg-read-pty = Lesen aus dem Pseudoterminal nicht möglich

dpkg-write-pty = Standardeingabe kann nicht an das Pseudoterminal gesendet werden

show-no-summary = Keine Zusammenfassung

show-size = Größe

show-sha256 = SHA256

show-archive = Archiv

show-codename = Codename

show-component = Komponente

show-attributes = Attribute

show-obsoletes = Ersetzt veraltete Pakete

show-attr-local = Lokal

show-attr-auto-removable = Automatisch entfernbar

show-attr-automatic = Automatisch

policy-none = keine

policy-candidate = Kandidat:

policy-version-table = Versionstabelle:

policy-no-versions = Keine Versionen.

policy-origin = Herkunft

policy-package-files = Paketdateien:

policy-pinned = Festgelegte Pakete:

policy-release = Veröffentlichung

history-cleared =
    { $count ->
        [one] { $count } Verlaufseintrag gelöscht.
       *[other] { $count } Verlaufseinträge gelöscht.
    }

history-not-replayable = Verlaufseintrag '{ $id }' kann nicht wiederholt werden, da er nicht als angewendet aufgezeichnet wurde

history-no-changes = Verlaufseintrag '{ $id }' enthält keine Paketänderungen zum Wiederholen

history-cleared-entry = Verlaufseintrag { $id } gelöscht.

history-id = ID

history-command = Befehl

history-date-time = Datum und Uhrzeit

history-requested-by = Angefordert von

history-altered = Geändert

history-status = Status

history-status-applied = Angewendet

history-started = Gestartet

history-finished = Abgeschlossen

history-targets = Angeforderte Ziele

history-undo-version-missing = Rückgängigmachen für '{ $package }' wird nicht unterstützt, da die vorherige Version nicht aufgezeichnet wurde

history-undo-config-only = Rückgängigmachen für '{ $package }' wird nicht unterstützt, da die Wiederherstellung des Zustands mit ausschließlich Konfigurationsdateien nicht implementiert ist

history-undo-installed-version-missing = Rückgängigmachen für '{ $package }' wird nicht unterstützt, da die zuvor installierte Version nicht aufgezeichnet wurde

history-undo-reinstall = Rückgängigmachen für '{ $package }' wird nicht unterstützt, da für die Neuinstallation kein umgekehrter Vorgang aufgezeichnet wurde

history-undo-held = Zurückgehaltenes Paket '{ $package }' kann nicht rückgängig gemacht werden

history-redo-version-missing = Wiederholen für '{ $package }' wird nicht unterstützt, da die resultierende Version nicht aufgezeichnet wurde

history-redo-reinstall-missing = Wiederholen für '{ $package }' wird nicht unterstützt, da die neu installierte Version nicht aufgezeichnet wurde

history-redo-held = Zurückgehaltenes Paket '{ $package }' kann nicht wiederholt werden

history-clear-target = Zum Löschen des Verlaufs ist eine Eintragsauswahl oder --all erforderlich

history-config-undo = Konfiguriertes Paket '{ $package }' kann nicht rückgängig gemacht werden

history-config-redo = Konfiguriertes Paket '{ $package }' kann nicht wiederholt werden

history-serialize = Verlaufseintrag kann nicht serialisiert werden

fetch-none-selected = Keine Spiegelserver ausgewählt.

fetch-unsupported = { $distro } wird nicht unterstützt.

fetch-title = Nala-Spiegelauswahl

fetch-score = Bewertung:

fetch-help = Mit ↓↑ navigieren, mit der Leertaste auswählen/abwählen, mit Home/End zum Anfang/Ende springen, mit q/Enter beenden.

update-item = { $state }: { $description }

upgrade-exclude-unsafe =
    Ausgewählte Pakete können nicht sicher von der Aktualisierung ausgeschlossen werden.
    { $error }

upgrade-protect = { $package } wird vor { $reason } geschützt

upgrade-reason-upgrade = Aktualisierung

upgrade-reason-auto-remove = automatisches Entfernen

upgrade-config-missing = Kein Konfigurationsbaum!
