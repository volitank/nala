### Messages migrated from legacy/python/po/fr.po.
### Missing messages fall back to English.

log-error = Erreur :

log-notice = Avis :

log-warning = Avertissement :

prompt-continue = Voulez-vous continuer ?

prompt-choice = O/o N/n

root-required = Nala à besoin des droits root pour { $command }

op-auto-purge = Auto-Purge

op-auto-remove = Auto-Suppression

op-remove = Supprimer

op-purge = Purge

op-install = Installer

op-reinstall = Réinstaller

op-upgrade = Mettre à niveau

op-downgrade = Rétrograder

op-configure = Configurer

op-held = Retenu

reason-kept-back = Retenu

summary-title = Résumé

summary-total-download = Taille totale du téléchargement:

summary-disk-required = Espace disque requis:

summary-disk-free = Espace disque à libérer:

summary-total-download-value = Taille totale du téléchargement: { $size }

summary-disk-required-value = Espace disque requis: { $size }

summary-disk-free-value = Espace disque à libérer: { $size }

summary-nothing = Rien à faire pour Nala.

summary-essential = Les paquets suivants sont essentiels !

summary-remove-essential = Vous avez tenté de supprimer des paquets essentiels

summary-use-switch = Veuillez utiliser { $switch } si vous êtes sûr de vouloir continuer.

summary-reboot = Un redémarrage est nécessaire.

summary-reboot-packages = Les paquets suivants nécessitent un redémarrage.

table-package = Paquet:

table-version = Version:

table-size = Taille:

table-old-version = Ancienne Version:

table-new-version = Nouvelle Version:

pkg-no-candidate = { $package } n'a pas de candidat à l'installation.

pkg-not-installed = { $package } n'est pas installé

pkg-already-latest = { $package } est déjà à la dernière version { $version }

pkg-version-missing = Version { $version } introuvable pour le paquet { $package }

pkg-cache-missing = { $package } mais il n'est pas dans le cache

pkg-recorded-version-missing = impossible de trouver la version pour { $package }

pkg-version-cache-missing = Version { $version } introuvable pour le paquet { $package }

pkg-virtual-providers = { $package } est un paquet virtuel fourni par :

pkg-virtual-one = Vous devriez en sélectionner un seul.

download-complete = Téléchargement terminé :

download-auth-warning = Les paquets suivants ne peuvent pas être authentifiés !

download-auth-allowed = La configuration permet l'installation de paquets non authentifiés.

download-exit = Arrêt à votre demande.

progress-working = [En cours]

progress-remaining = Temps restant

progress-fetched = { $size } en { $time } ({ $rate }/s)

tui-pkgs = Paquets

file-read-defaults = Impossible de lire le fichier de configuration : { $path }. Utilisation des valeurs par défaut

hash-unsupported = Le type de somme de contrôle '{ $type }' n'est pas pris en charge

dpkg-removing = Suppression :

dpkg-unpacking = Déballage :

dpkg-setting-up = Configuration :

dpkg-processing = Traitement :

list-virtual = Paquet virtuel

show-local-install = installation locale

show-more-records = Il y a { $count } enregistrements supplémentaires. Veuillez utiliser le switch { $switch } pour les voir.

show-no-description = Pas de description

show-package = Paquet

show-version = Version

show-architecture = Architecture

show-priority = Priorité

show-essential = Essentiel

show-section = Section

show-source = Source

show-installed-size = Taille installée

show-original-maintainer = Mainteneur original

show-homepage = Page d'accueil

show-origin = Origine

show-provides = Fournit

show-description = Description

show-apt-sources = Sources APT

show-depends = Dépendances

show-pre-depends = Pré-Dépendances

show-suggests = Suggère

show-recommends = Recommande

show-conflicts = Conflits

show-replaces = Remplace

show-breaks = Casse

show-enhances = Améliore

show-attr-installed = Installé

show-attr-upgradable-to = est installé et peut être mis à niveau vers { $version }

show-attr-upgradable-from = peut être mis à niveau depuis { $version }

policy-installed = Installé :

history-empty = Aucun historique n'existe.

history-not-found = La transaction { $id } n'existe pas.

fetch-no-mirrors = Nala n'a pas pu trouver de miroirs.

fetch-release-detect = Il y a eu un problème lors de la détection de la version.

fetch-sources-written = Les sources ont été écrites dans { $file }

fetch-score-help = Le score est le nombre de millisecondes nécessaires pour télécharger le fichier Release

update-upgradable = { $count } paquets peuvent être mis à jour. Exécutez '{ $command }' pour les voir.

update-no-change = Aucun changement

update-updated = Mis à jour

update-ignored = Ignoré

update-downloading = Téléchargement

update-processing = Traitement

install-downloaded = Téléchargé : { $path }

## Additional messages

unknown = Inconnu

none = Aucun

error = Erreur

mirrors = Miroirs :

log-info = Information :

log-verbose = Détaillé :

log-debug = Débogage :

not-implemented = Pas encore implémenté.

subcommand-missing = Aucune sous-commande fournie

prompt-invalid = « { $response } » n'est pas une réponse valide

prompt-refused = L'utilisateur a refusé la confirmation

release-info-title = Informations du dépôt modifiées

release-info-explanation =
    Les informations d'identification d'un dépôt ont changé dans son fichier Release.
    Examinez les modifications avant de continuer.

release-info-repository = Dépôt

release-info-distribution = Distribution

release-info-changes = Modifications

release-info-confirm = Accepter ces modifications et poursuivre la mise à jour ?

release-info-help = ({ $yes }) accepter | ({ $no }/Enter/Esc) refuser

release-info-accepted = Modifications des informations du dépôt { $repository } acceptées.

release-info-rejected = Modifications des informations du dépôt { $repository } refusées.

release-info-noninteractive = Les modifications des informations Release doivent être explicitement acceptées. Relancez avec « -o Acquire::AllowReleaseInfoChange=true » pour les accepter.

reason-excluded = Exclu

reason-manual-hold = Bloqué manuellement

reason-phased = Échelonné

reason-phased-percent = Échelonné à { $percentage } %

summary-op-count = { $operation } { $count }

summary-op-count-colon = { $operation } : { $count }

summary-protected = Les paquets suivants sont essentiels ou protégés !

summary-remove-protected = Vous avez tenté de supprimer des paquets essentiels ou protégés

summary-downgrade-assume-yes = Des paquets seraient rétrogradés avec --assume-yes. Utilisez « -o APT::Get::allow-downgrades=true » si vous êtes sûr.

summary-held-assume-yes = Des paquets bloqués seraient modifiés avec --assume-yes. Utilisez « -o APT::Get::allow-change-held-packages=true » si vous êtes sûr.

autoremove-config-purge = --remove-config doit être utilisé avec --purge ou autopurge.

autoremove-assume-yes = Suppression automatique ignorée avec --assume-yes. Utilisez --autoremove pour l'activer.

table-reason = Motif :

pkg-not-found = « { $package } » est introuvable

pkg-not-found-cache = Certains paquets sont introuvables dans le cache

pkg-mark-install = Impossible de marquer « { $package } » pour installation

pkg-reinstall-missing = { $package } n'est pas installé et ne peut donc pas être réinstallé

pkg-invalid-name = Nom de paquet invalide : « { $package } »

pkg-op-conflict = Opérations contradictoires pour « { $package } » : { $operation-a } contre { $operation-b }

pkg-version-conflict = Versions épinglées contradictoires pour « { $package } » : { $version-a } contre { $version-b }

pkg-invalid-pin = Version épinglée invalide : « { $pin } »

pkg-pin-exact = L'épinglage d'une version exige un nom de paquet exact : « { $pin } »

pkg-exclude-missing = Le motif d'exclusion « { $pattern } » ne correspond à aucun paquet

pkg-virtual-no-providers = { $package } n'a aucun fournisseur et est purement virtuel

pkg-virtual-select = Sélection de { $provider } à la place du paquet virtuel { $package }

pkg-select = Sélection du paquet « { $selected } » à la place de « { $requested } »

pkg-fix-broken = Impossible de corriger les paquets cassés

pkg-required = Vous devez spécifier un paquet

download-failed = Téléchargements échoués

download-some-missing = Certains paquets sont introuvables.

download-auth-required =
    Certains paquets n'ont pas pu être authentifiés.
    Si vous êtes sûr, utilisez { $switch }

download-items = Éléments

download-source-missing = Impossible de trouver une source pour télécharger la version « { $version } » de « { $package } »

download-written = { $package } a été écrit dans { $path }

download-hash-missing = Aucun hachage trouvé pour « { $uri } »

download-protocol-missing = Aucun protocole défini

download-uri-missing = Aucune URI définie

download-malformed = « { $uri } » est mal formée !

download-request-failed = Échec de la requête de téléchargement pour « { $uri } »

download-stream-failed = Impossible de recevoir le flux de données de « { $uri } »

download-get = Impossible d'envoyer la requête de téléchargement

download-content-length = content-length est absent de { $headers }

download-content-str = Échec de la conversion de content-length en chaîne : { $headers }

download-content-parse = Échec de l'analyse de content-length : { $headers }

download-checksum = La somme de contrôle ne correspond pas pour { $file }

download-no-uris = Aucune URI n'a permis de télécharger { $file }

download-no-space = Espace libre insuffisant dans { $path } : { $required } requis, { $available } disponible.

download-filename = { $path } n'a pas de nom de fichier valide !

download-error = Erreur : { $error }

download-select-domain = Sélection de { $domain } pour { $file }

download-start = Démarrage : { $uri }, Nouvelles tentatives : { $retries }

download-finished = Terminé : { $uri }

progress-nothing = Rien à récupérer

progress-label = Progression

progress-total = Total

progress-data = Données

progress-packages = Paquets

progress-package = Paquet

progress-status = Statut

progress-mirrors = Miroirs

progress-speed = Vitesse

progress-elapsed = Écoulé

progress-finished = Terminé :

progress-update-title = Mise à jour des listes de paquets

progress-download-title = Téléchargement des paquets

progress-install-title = Installation des paquets

progress-score-title = Test des miroirs

progress-last-completed = Dernier terminé

progress-connections = Connexions

progress-average = Moyenne

progress-active = Actif

progress-state = État

progress-starting = démarrage

progress-downloading = téléchargement

progress-idle = inactif

progress-more-mirrors = +{ $mirrors } miroirs supplémentaires · { $connections } connexions actives

tui-upgrade-title = Mise à niveau Nala

tui-history-title = Historique Nala

tui-help-move = (↑) monter | (↓) descendre

tui-help-tabs = (→) onglet suivant | (←) onglet précédent

tui-help-show = (Enter) afficher le journal des modifications | (s) afficher les informations de version

tui-help-confirm = (q) quitter | (y) démarrer la mise à niveau

tui-help-quit = (q) quitter

tui-changelog-missing = Impossible de trouver l'URI du journal des modifications

file-remove = Échec de la suppression de { $path }

file-read = Échec de la lecture de « { $path } »

file-parse-defaults = Échec de l'analyse de { $path }, utilisation des valeurs par défaut

config-parse = Échec de l'analyse de { $path }. Nala n'ignorera pas un fichier de configuration existant. Si cela suit une mise à jour de paquet, comparez-le à { $path }.dpkg-dist

file-create = Impossible de créer « { $path } »

file-write = Impossible d'écrire « { $path } »

file-replace = Impossible de remplacer « { $path } »

file-deserialize = Impossible de désérialiser « { $path } »

fs-failed = Échec de l'opération { $operation } sur { $path }

fs-failed-target = Échec de l'opération { $operation } : { $path } => { $target }

config-option = L'option « { $option } » n'est pas prise en charge

color-expected = un nom de couleur, un indice de 0 à 255 ou une chaîne hexadécimale #RRGGBB

color-index = l'indice de couleur doit être compris entre 0 et 255

color-unknown = couleur inconnue « { $color } »

color-rgb-components = Rgb attend trois composantes

color-rgb-expected = une couleur Rgb ou Indexed était attendue

color-modifier-expected = une chaîne de modificateurs comme "BOLD | ITALIC" ou un tableau

color-modifier-unknown = modificateur inconnu « { $modifier } »

archive-unsupported = Le type d'archive n'est pas pris en charge

deb-control-missing = fichier control introuvable dans { $path }

hash-unavailable =
    L'intégrité de { $package } { $version } ne peut pas être vérifiée.
    Aucun hachage n'est disponible pour ce paquet.

dpkg-child-failed = Échec du processus fils de dpkg : { $error }

dpkg-status-utf8 = Le descripteur de statut de dpkg a renvoyé du texte UTF-8 invalide

dpkg-exit = Dpkg s'est terminé avec le code : « { $code } »

dpkg-poll = Impossible d'interroger le processus fils

dpkg-read-status = Impossible de lire le descripteur de statut

dpkg-read-pty = Impossible de lire le pseudo-terminal

dpkg-write-pty = Impossible d'envoyer l'entrée standard au pseudo-terminal

show-no-summary = Aucun résumé

show-size = Taille

show-maintainer = Responsable

show-sha256 = SHA256

show-archive = Archive

show-codename = Nom de code

show-component = Composant

show-attributes = Attributs

show-obsoletes = Rend obsolète

show-attr-local = Local

show-attr-auto-removable = Supprimable automatiquement

show-attr-automatic = Automatique

policy-none = aucun

policy-candidate = Candidat :

policy-version-table = Table des versions :

policy-no-versions = Aucune version.

policy-origin = origine

policy-package-files = Fichiers de paquets :

policy-pinned = Paquets épinglés :

policy-release = publication

history-cleared =
    { $count ->
        [one] { $count } entrée d'historique effacée.
       *[other] { $count } entrées d'historique effacées.
    }

history-not-replayable = L'entrée d'historique « { $id } » ne peut pas être rejouée car elle n'a pas été enregistrée comme appliquée

history-no-changes = L'entrée d'historique « { $id } » ne contient aucune modification de paquet à rejouer

history-cleared-entry = Entrée d'historique { $id } effacée.

history-id = ID

history-command = Commande

history-date-time = Date et heure

history-requested-by = Demandé par

history-altered = Modifié

history-status = Statut

history-status-applied = Appliqué

history-started = Démarré

history-finished = Terminé

history-targets = Cibles demandées

history-undo-version-missing = L'annulation n'est pas prise en charge pour « { $package } » car la version précédente n'a pas été enregistrée

history-undo-config-only = L'annulation n'est pas prise en charge pour « { $package } » car la restauration de l'état contenant uniquement les fichiers de configuration n'est pas implémentée

history-undo-installed-version-missing = L'annulation n'est pas prise en charge pour « { $package } » car la version précédemment installée n'a pas été enregistrée

history-undo-reinstall = L'annulation n'est pas prise en charge pour « { $package } » car la réinstallation n'a pas d'opération inverse enregistrée

history-undo-held = Le paquet bloqué « { $package } » ne peut pas être annulé

history-redo-version-missing = Le rétablissement n'est pas pris en charge pour « { $package } » car la version résultante n'a pas été enregistrée

history-redo-reinstall-missing = Le rétablissement n'est pas pris en charge pour « { $package } » car la version réinstallée n'a pas été enregistrée

history-redo-held = Le paquet bloqué « { $package } » ne peut pas être rétabli

history-clear-target = L'effacement de l'historique exige un sélecteur d'entrée ou --all

history-config-undo = Le paquet configuré « { $package } » ne peut pas être annulé

history-config-redo = Le paquet configuré « { $package } » ne peut pas être rétabli

history-serialize = Impossible de sérialiser l'entrée d'historique

fetch-none-selected = Aucun miroir sélectionné.

fetch-unsupported = { $distro } n'est pas pris en charge.

fetch-title = Sélection des miroirs Nala

fetch-score = Score :

fetch-help = Utilisez ↓↑ pour naviguer, Espace pour sélectionner/désélectionner, Home/End pour aller au début/à la fin et q/Enter pour quitter.

update-item = { $state } : { $description }

upgrade-exclude-unsafe =
    Les paquets sélectionnés ne peuvent pas être exclus de la mise à niveau en toute sécurité.
    { $error }

upgrade-protect = Protection de { $package } contre { $reason }

upgrade-reason-upgrade = mise à niveau

upgrade-reason-auto-remove = suppression automatique

upgrade-config-missing = Aucun arbre de configuration !
