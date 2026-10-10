### Messages migrated from legacy/python/po/es.po.
### Missing messages fall back to English.

log-error = Error:

log-notice = Aviso:

log-warning = Advertencia:

prompt-continue = ¿Desea continuar?

prompt-choice = S/s N/n

root-required = Nala requiere acceso de root para ejecutar { $command }

op-auto-purge = Purgar automáticamente

op-auto-remove = Desinstalar automáticamente

op-remove = Desinstalar

op-purge = Purgar

op-install = Instalar

op-reinstall = Reinstalar

op-upgrade = Actualizar

op-downgrade = Degradar

op-configure = Configurar

op-held = Retenido

reason-kept-back = Retenido

summary-title = Resumen

summary-total-download = Tamaño total de descarga:

summary-disk-required = Espacio en disco requerido:

summary-disk-free = Espacio en disco a liberar:

summary-total-download-value = Tamaño total de descarga: { $size }

summary-disk-required-value = Espacio en disco requerido: { $size }

summary-disk-free-value = Espacio en disco a liberar: { $size }

summary-nothing = No hay nada que Nala pueda hacer.

summary-essential = ¡Los siguientes paquetes son esenciales!

summary-remove-essential = Ha intentado desinstalar paquetes esenciales

summary-use-switch = Utilice { $switch } si está seguro que quiere hacerlo.

summary-reboot = Se requiere reiniciar.

summary-reboot-packages = Los siguientes paquetes requieren reiniciar.

table-package = Paquete:

table-version = Versión:

table-size = Tamaño:

table-old-version = Versión anterior:

table-new-version = Versión nueva:

pkg-no-candidate = { $package } no tiene candidato para la instalación.

pkg-not-installed = { $package } no está instalado

pkg-already-latest = { $package } ya está en la versión más reciente { $version }

pkg-version-missing = La versión { $version } no se encontró para el paquete { $package }

pkg-cache-missing = { $package } pero no está en el caché

pkg-recorded-version-missing = no se pudo encontrar la versión para { $package }

pkg-version-cache-missing = La versión { $version } no se encontró para el paquete { $package }

pkg-virtual-providers = { $package } es un paquete virtual proporcionado por:

pkg-virtual-one = Debe seleccionar solo uno.

download-complete = Descarga completa:

download-auth-warning = ¡Los siguientes paquetes no pudieron ser autenticados!

download-auth-allowed = Se estableció la configuración para permitir la instalación de paquetes no autenticados.

download-exit = Saliendo a petición suya.

progress-working = [Trabajando]

progress-remaining = Tiempo restante

progress-fetched = { $size } en { $time } ({ $rate }/s)

tui-pkgs = Paquetes

file-read-defaults = No se puede leer el archivo de configuración: { $path }. Usando los valores predeterminados

hash-unsupported = El tipo de hash '{ $type }' es incompatible

dpkg-removing = Desinstalando:

dpkg-unpacking = Desempaquetando:

dpkg-setting-up = Configurando:

dpkg-processing = Procesando:

list-virtual = Paquete virtual

show-local-install = instalación local

show-more-records = Hay { $count } registros adicionales. Utilice la opción { $switch } para verlos.

show-no-description = Sin descripción

show-package = Paquete

show-version = Versión

show-architecture = Arquitectura

show-priority = Prioridad

show-essential = Esencial

show-section = Sección

show-source = Fuente

show-installed-size = Tamaño instalado

show-maintainer = Responsable

show-original-maintainer = Responsable original

show-homepage = Sitio web

show-origin = Origen

show-provides = Proporciona

show-description = Descripción

show-apt-sources = Fuentes APT

show-depends = Depende

show-pre-depends = Pre-Depende

show-suggests = Sugiere

show-recommends = Recomienda

show-conflicts = Entra en conflicto

show-replaces = Reemplaza

show-breaks = Rompe

show-enhances = Mejora

show-attr-installed = Instalado

show-attr-upgradable-to = está instalado y es actualizable a { $version }

show-attr-upgradable-from = es actualizable desde { $version }

policy-installed = Instalado:

history-empty = No existe historial.

history-not-found = La transacción { $id } no existe.

fetch-no-mirrors = Nala no pudo encontrar ningún servidor espejo.

fetch-release-detect = Hubo un problema al detectar la versión.

fetch-sources-written = Las fuentes se han escrito en { $file }

fetch-score-help = La puntuación son los milisegundos que tarda en descargar el archivo Release

update-upgradable = { $count } paquetes pueden ser actualizados. Ejecute '{ $command }' para verlos.

update-no-change = Sin cambios

update-updated = Actualizado

update-ignored = Ignorado

update-downloading = Descargando

update-processing = Procesando

install-downloaded = Descargado: { $path }

## Additional messages

unknown = Desconocido

none = Ninguno

error = Error

mirrors = Réplicas:

log-info = Información:

log-verbose = Detallado:

log-debug = Depuración:

not-implemented = Aún no implementado.

subcommand-missing = No se ha indicado ningún subcomando

prompt-invalid = '{ $response }' no es una respuesta válida

prompt-refused = El usuario rechazó la confirmación

release-info-title = La información del repositorio ha cambiado

release-info-explanation =
    La información que identifica a un repositorio ha cambiado en su archivo Release.
    Revise los cambios antes de continuar.

release-info-repository = Repositorio

release-info-distribution = Distribución

release-info-changes = Cambios

release-info-confirm = ¿Aceptar estos cambios y continuar la actualización?

release-info-help = ({ $yes }) aceptar | ({ $no }/Enter/Esc) rechazar

release-info-accepted = Se aceptaron los cambios de información del repositorio { $repository }.

release-info-rejected = Se rechazaron los cambios de información del repositorio { $repository }.

release-info-noninteractive = Los cambios en la información de Release deben aceptarse explícitamente. Ejecute de nuevo con '-o Acquire::AllowReleaseInfoChange=true' para aceptarlos.

reason-excluded = Excluido

reason-manual-hold = Retenido manualmente

reason-phased = Escalonado

reason-phased-percent = Escalonado { $percentage }%

summary-op-count = { $operation } { $count }

summary-op-count-colon = { $operation }: { $count }

summary-protected = ¡Los siguientes paquetes son esenciales o están protegidos!

summary-remove-protected = Ha intentado eliminar paquetes esenciales o protegidos

summary-downgrade-assume-yes = Con --assume-yes se instalarían versiones anteriores de los paquetes. Use '-o APT::Get::allow-downgrades=true' si está seguro.

summary-held-assume-yes = Con --assume-yes se modificarían paquetes retenidos. Use '-o APT::Get::allow-change-held-packages=true' si está seguro.

autoremove-config-purge = --remove-config debe usarse con --purge o autopurge.

autoremove-assume-yes = Se omite la eliminación automática con --assume-yes. Use --autoremove para activarla.

table-reason = Motivo:

pkg-not-found = No se encontró '{ $package }'

pkg-not-found-cache = Algunos paquetes no se encontraron en la caché

pkg-mark-install = No se puede marcar '{ $package }' para su instalación

pkg-reinstall-missing = { $package } no está instalado, por lo que no se puede reinstalar

pkg-invalid-name = Nombre de paquete no válido: '{ $package }'

pkg-op-conflict = Operaciones incompatibles para '{ $package }': { $operation-a } frente a { $operation-b }

pkg-version-conflict = Versiones fijadas incompatibles para '{ $package }': { $version-a } frente a { $version-b }

pkg-invalid-pin = Versión fijada no válida: '{ $pin }'

pkg-pin-exact = Para fijar una versión se necesita el nombre exacto del paquete: '{ $pin }'

pkg-exclude-missing = El patrón de exclusión '{ $pattern }' no coincide con ningún paquete

pkg-virtual-no-providers = { $package } no tiene proveedores y es puramente virtual

pkg-virtual-select = Se selecciona { $provider } en lugar del paquete virtual { $package }

pkg-select = Se selecciona el paquete '{ $selected }' en lugar de '{ $requested }'

pkg-fix-broken = No se pueden corregir los paquetes rotos

pkg-required = Debe indicar un paquete

download-failed = Descargas fallidas

download-some-missing = Algunos paquetes no se encontraron.

download-auth-required =
    No se pudieron autenticar algunos paquetes.
    Si está seguro, use { $switch }

download-items = Elementos

download-source-missing = No se encuentra una fuente para descargar la versión '{ $version }' de '{ $package }'

download-written = { $package } se escribió en { $path }

download-hash-missing = No se encontró un hash para '{ $uri }'

download-protocol-missing = No se definió ningún protocolo

download-uri-missing = No se definió ninguna URI

download-malformed = ¡'{ $uri }' tiene un formato incorrecto!

download-request-failed = Falló la solicitud de descarga de '{ $uri }'

download-stream-failed = No se pueden recibir datos de '{ $uri }'

download-get = No se puede enviar la solicitud de descarga

download-content-length = content-length no existe en { $headers }

download-content-str = Falló la conversión de content-length a una cadena: { $headers }

download-content-parse = Falló el análisis de content-length: { $headers }

download-checksum = La suma de comprobación de { $file } no coincide

download-no-uris = No se pudo descargar { $file } desde ninguna URI

download-no-space = No hay suficiente espacio libre en { $path }: se necesitan { $required }, hay { $available } disponibles.

download-filename = ¡{ $path } no tiene un nombre de archivo válido!

download-error = Error: { $error }

download-select-domain = Se selecciona { $domain } para { $file }

download-start = Iniciando: { $uri }, Reintentos: { $retries }

download-finished = Finalizado: { $uri }

progress-nothing = No hay nada que obtener

progress-label = Progreso

progress-total = Total

progress-data = Datos

progress-packages = Paquetes

progress-package = Paquete

progress-status = Estado

progress-mirrors = Réplicas

progress-speed = Velocidad

progress-elapsed = Transcurrido

progress-finished = Finalizado:

progress-update-title = Actualizando las listas de paquetes

progress-download-title = Descargando paquetes

progress-install-title = Instalando paquetes

progress-score-title = Probando réplicas

progress-last-completed = Último completado

progress-connections = Conexiones

progress-average = Promedio

progress-active = Activo

progress-state = Estado

progress-starting = iniciando

progress-downloading = descargando

progress-idle = inactivo

progress-more-mirrors = +{ $mirrors } réplicas más · { $connections } conexiones activas

tui-upgrade-title = Actualización de Nala

tui-history-title = Historial de Nala

tui-help-move = (↑) subir | (↓) bajar

tui-help-tabs = (→) pestaña siguiente | (←) pestaña anterior

tui-help-show = (Enter) mostrar registro de cambios | (s) mostrar información de la versión

tui-help-confirm = (q) salir | (y) iniciar actualización

tui-help-quit = (q) salir

tui-changelog-missing = No se encuentra la URI del registro de cambios

file-remove = No se pudo eliminar { $path }

file-read = No se pudo leer '{ $path }'

file-parse-defaults = No se pudo analizar { $path }, se usarán los valores predeterminados

config-parse = No se pudo analizar { $path }. Nala no ignorará un archivo de configuración existente. Si ocurrió después de actualizar un paquete, compárelo con { $path }.dpkg-dist

file-create = No se puede crear '{ $path }'

file-write = No se puede escribir '{ $path }'

file-replace = No se puede reemplazar '{ $path }'

file-deserialize = No se puede deserializar '{ $path }'

fs-failed = Falló la operación { $operation } sobre { $path }

fs-failed-target = Falló la operación { $operation }: { $path } => { $target }

config-option = La opción '{ $option }' no es compatible

color-expected = un nombre de color, un índice de 0 a 255 o una cadena hexadecimal #RRGGBB

color-index = el índice de color debe estar entre 0 y 255

color-unknown = color desconocido '{ $color }'

color-rgb-components = Rgb requiere tres componentes

color-rgb-expected = se esperaba un color Rgb o Indexed

color-modifier-expected = una cadena de modificadores como "BOLD | ITALIC" o un arreglo

color-modifier-unknown = modificador desconocido '{ $modifier }'

archive-unsupported = El tipo de archivo comprimido no es compatible

deb-control-missing = No se encuentra el archivo control en { $path }

hash-unavailable =
    No se puede comprobar la integridad de { $package } { $version }.
    No hay hashes disponibles para este paquete.

dpkg-child-failed = Falló el proceso hijo de dpkg: { $error }

dpkg-status-utf8 = El descriptor de estado de dpkg devolvió UTF-8 no válido

dpkg-exit = Dpkg terminó con el código: '{ $code }'

dpkg-poll = No se puede consultar el proceso hijo

dpkg-read-status = No se puede leer el descriptor de estado

dpkg-read-pty = No se puede leer del pseudoterminal

dpkg-write-pty = No se puede enviar la entrada estándar al pseudoterminal

show-no-summary = Sin resumen

show-size = Tamaño

show-sha256 = SHA256

show-archive = Archivo

show-codename = Nombre en clave

show-component = Componente

show-attributes = Atributos

show-obsoletes = Deja obsoleto

show-attr-local = Local

show-attr-auto-removable = Eliminable automáticamente

show-attr-automatic = Automático

policy-none = ninguno

policy-candidate = Candidato:

policy-version-table = Tabla de versiones:

policy-no-versions = No hay versiones.

policy-origin = origen

policy-package-files = Archivos de paquetes:

policy-pinned = Paquetes fijados:

policy-release = publicación

history-cleared =
    { $count ->
        [one] Se borró { $count } entrada del historial.
       *[other] Se borraron { $count } entradas del historial.
    }

history-not-replayable = No se puede repetir la entrada '{ $id }' del historial porque no se registró como aplicada

history-no-changes = La entrada '{ $id }' del historial no tiene cambios de paquetes que repetir

history-cleared-entry = Se borró la entrada { $id } del historial.

history-id = ID

history-command = Comando

history-date-time = Fecha y hora

history-requested-by = Solicitado por

history-altered = Modificado

history-status = Estado

history-status-applied = Aplicado

history-started = Iniciado

history-finished = Finalizado

history-targets = Objetivos solicitados

history-undo-version-missing = No se puede deshacer '{ $package }' porque no se registró la versión anterior

history-undo-config-only = No se puede deshacer '{ $package }' porque no se ha implementado la restauración del estado de solo archivos de configuración

history-undo-installed-version-missing = No se puede deshacer '{ $package }' porque no se registró la versión instalada anteriormente

history-undo-reinstall = No se puede deshacer '{ $package }' porque la reinstalación no tiene una operación inversa registrada

history-undo-held = No se puede deshacer el paquete retenido '{ $package }'

history-redo-version-missing = No se puede rehacer '{ $package }' porque no se registró la versión resultante

history-redo-reinstall-missing = No se puede rehacer '{ $package }' porque no se registró la versión reinstalada

history-redo-held = No se puede rehacer el paquete retenido '{ $package }'

history-clear-target = Para borrar el historial debe seleccionar una entrada o usar --all

history-config-undo = No se puede deshacer el paquete configurado '{ $package }'

history-config-redo = No se puede rehacer el paquete configurado '{ $package }'

history-serialize = No se puede serializar la entrada del historial

fetch-none-selected = No se seleccionó ninguna réplica.

fetch-unsupported = { $distro } no es compatible.

fetch-title = Selección de réplicas de Nala

fetch-score = Puntuación:

fetch-help = Use ↓↑ para desplazarse, Espacio para seleccionar/deseleccionar, Home/End para ir al principio/final y q/Enter para salir.

update-item = { $state }: { $description }

upgrade-exclude-unsafe =
    Los paquetes seleccionados no pueden excluirse de la actualización de forma segura.
    { $error }

upgrade-protect = Se protege { $package } de { $reason }

upgrade-reason-upgrade = actualización

upgrade-reason-auto-remove = eliminación automática

upgrade-config-missing = ¡No hay árbol de configuración!
