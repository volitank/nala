### Ukrainian messages displayed by Nala.

## General labels and errors

unknown = Невідомо
none = Нічого
error = Помилка
mirrors = Дзеркала:

log-error = Помилка:
log-notice = Зауваження:
log-warning = Попередження:
log-info = Інфо:
log-verbose = Деталі:
log-debug = Налагодження:

not-implemented = Ще не реалізовано.
subcommand-missing = Підкоманду не знайдено

# Variables:
#   $response (String) - Invalid confirmation response.
prompt-invalid = '{ $response }' - неочікувана відповідь
prompt-refused = Користувач відхилив дію
prompt-continue = Бажаєте продовжити?
prompt-choice = [Y/n]
prompt-choice-no = [y/N]

release-info-title = Інформацію про репозиторій змінено
release-info-explanation =
    Ідентифікаційні дані репозиторію змінено у файлі випуску.
    Перегляньте зміни, перш ніж продовжити.
release-info-repository = Репозиторій
release-info-distribution = Дистрибутив
release-info-changes = Зміни
release-info-confirm = Підтвердити зміни та продовжити?
release-info-help = (y) підтвердити | (n/Enter/Esc) відхилити
release-info-accepted = Прийнято зміни інформації про репозиторій для { $repository }.
release-info-rejected = Відхилено зміни інформації про репозиторій для { $repository }.
release-info-noninteractive = Зміни інформації про випуск потрібно підтвердити явно. Запустіть повторно з '-o Acquire::AllowReleaseInfoChange=true', щоб прийняти їх.

# Variables:
#   $command (String) - Command that requires elevated privileges.
root-required = Nala вимагає прав суперкористувача для { $command }

## Transaction operations and reasons

op-remove = Вилучення
op-auto-remove = Автовилучення
op-purge = Очищення
op-auto-purge = Автоочищення
op-install = Встановлення
op-reinstall = Перевстановлення
op-upgrade = Оновлення
op-downgrade = Пониження версії
op-configure = Налаштування
op-held = Утримання

reason-excluded = Виключено
reason-manual-hold = Ручне утримання
reason-phased = Поетапно
reason-kept-back = Залишено

# Variables:
#   $percentage (Number) - Phased update percentage.
reason-phased-percent = Поетапно { $percentage }%

## Transaction summary

summary-title = Підсумок
summary-total-download = Загальний розмір завантаження:
summary-disk-required = Потрібно місця на диску:
summary-disk-free = Буде вивільнено місця на диску:

summary-total-download-value = Загальний розмір завантаження: { $size }
summary-disk-required-value = Потрібно місця на диску: { $size }
summary-disk-free-value = Буде вивільнено місця на диску: { $size }
summary-op-count = { $operation } { $count }
summary-op-count-colon = { $operation }: { $count }

summary-nothing = Нічого робити.
summary-essential = Наступні пакунки є важливими!
summary-remove-essential = Ви намагаєтесь вилучити важливі пакунки

# Variables:
#   $switch (String) - Command-line switch that permits essential package removal.
summary-use-switch = Використайте '{ $switch }', якщо ви впевнені.
autoremove-config-purge = --remove-config можна використовувати тільки разом з --purge або autopurge.

summary-reboot = Для завершення цих змін необхідне перезавантаження.
summary-reboot-packages = Перезавантаження необхідне для наступних пакунків:

## Package summary table

table-package = Пакунок:
table-version = Версія:
table-old-version = Стара версія:
table-new-version = Нова версія:
table-reason = Причина:
table-size = Розмір:

## Package selection

# Variables:
#   $package (String) - Package name or selection token.
pkg-not-found = '{ $package }' не знайдено
pkg-not-found-cache = Деякі пакунки не знайдено в кеші
pkg-no-candidate = { $package } не має кандидата на встановлення
pkg-mark-install = Неможливо позначити '{ $package }' для встановлення
pkg-not-installed = { $package } не встановлено
pkg-reinstall-missing = { $package } не встановлено, тому не може бути перевстановлено
pkg-invalid-name = Неправильне ім'я пакунка: '{ $package }'

# Variables:
#   $package (String) - Package name.
#   $operation-a (String) - First requested operation.
#   $operation-b (String) - Conflicting requested operation.
pkg-op-conflict = Конфліктна операція для '{ $package }': { $operation-a } проти { $operation-b }

# Variables:
#   $package (String) - Package name.
#   $version-a (String) - First requested version.
#   $version-b (String) - Conflicting requested version.
pkg-version-conflict = Конфліктна закріплена версія '{ $package }': { $version-a } проти { $version-b }

# Variables:
#   $package (String) - Package name.
#   $version (String) - Package version.
pkg-already-latest = { $package }{ $version } вже встановлено, і це остання версія
pkg-version-missing = Неможливо знайти версію '{ $version }' для '{ $package }'
pkg-cache-missing = Пакунок '{ $package }' не знайдено в кеші
pkg-recorded-version-missing = Немає записаної версії '{ $package }'
pkg-version-cache-missing = Версію '{ $version }' не знайдено для '{ $package }'

# Variables:
#   $pin (String) - Version pin supplied by the user.
pkg-invalid-pin = Неправильне закріплення версії: '{ $pin }'
pkg-pin-exact = Для закріплення версії необхідна точна назва пакунка: '{ $pin }'

# Variables:
#   $pattern (String) - Package exclusion pattern.
pkg-exclude-missing = Шаблон виключення '{ $pattern }' не збігся з жодним пакунком

# Variables:
#   $package (String) - Virtual package name.
pkg-virtual-no-providers = { $package } не має постачальника і є суто віртуальним
pkg-virtual-providers = { $package } - віртуальний пакунок, який надають:

# Variables:
#   $package (String) - Virtual package name.
#   $provider (String) - Selected concrete provider.
pkg-virtual-select = Вибрано { $provider } замість віртуального пакунка { $package }
pkg-virtual-one = Вам слід вибрати тільки один.

# Variables:
#   $selected (String) - Selected package name.
#   $requested (String) - Originally requested package name.
pkg-select = Вибрано пакунок '{ $selected }' замість '{ $requested }'

pkg-fix-broken = Не вдалося виправити пошкоджені пакунки
pkg-required = Потрібно вказати пакунок

## Download output

download-complete = Завантаження завершено:
download-failed = Завантаження не вдалося
download-some-missing = Деякі пакунки не знайдено.
download-auth-warning = Наступні пакунки неможливо автентифікувати!
download-auth-allowed = Налаштування дозволяють встановлення неавтентифікованих пакунків.
download-auth-required =
    Деякі пакунки не вдалося автентифікувати.
      Якщо ви впевнені, використайте { $switch }
download-exit = Скасовано користувачем
download-items = Елементи

# Variables:
#   $package (String) - Package name.
#   $version (String) - Package version.
download-source-missing = Не вдалося знайти джерело для завантаження версії '{ $version }' пакунка '{ $package }'

# Variables:
#   $package (String) - Package name.
#   $path (String) - Destination path.
download-written = { $package } записано до { $path }

# Variables:
#   $uri (String) - Download URI.
download-hash-missing = Не знайдено контрольної суми для '{ $uri }'
download-protocol-missing = Протокол не визначено
download-uri-missing = URI не визначено
download-malformed = '{ $uri }' сформовано некоректно!
download-request-failed = Запит на завантаження для '{ $uri }' не вдався
download-stream-failed = Не вдалося отримати потік даних з '{ $uri }'
download-get = Не вдалося надіслати запит на завантаження
download-content-length = content-length відсутній у { $headers }
download-content-str = Не вдалося перетворити content-length на рядок: { $headers }
download-content-parse = Не вдалося розібрати content-length: { $headers }

# Variables:
#   $file (String) - Downloaded filename.
download-checksum = Контрольна сума не збіглася для { $file }
download-no-uris = Жодного URI не вдалося завантажити для { $file }

# Variables:
#   $path (String) - Path without a valid filename.
download-filename = { $path } не має коректної назви файлу!

download-error = Помилка: { $error }
download-select-domain = Вибір { $domain } для { $file }
download-start = Початок: { $uri }, Повторень: { $retries }
download-finished = Завершення: { $uri }

## Progress

progress-working = Виконання...
progress-nothing = Нічого отримувати
progress-label = Прогрес
progress-total = Всього
progress-data = Дані
progress-packages = Пакунки
progress-package = Пакунок
progress-status = Статус
progress-mirrors = Дзеркала
progress-speed = Швидкість
progress-elapsed = Минуло
progress-remaining = Залишилось
progress-finished = Завершено:
progress-update-title = Оновлення списків пакунків
progress-download-title = Завантаження пакунків
progress-install-title = Встановлення пакунків
progress-score-title = Перевірка дзеркал
progress-last-completed = Востаннє завершено
progress-connections = З'єднання
progress-average = Середнє
progress-active = Активно
progress-state = Стан
progress-starting = запуск
progress-downloading = завантаження
progress-idle = очікування

# Variables:
#   $mirrors (Number) - Number of hidden mirrors.
#   $connections (Number) - Number of active connections across all mirrors.
progress-more-mirrors = +{ $mirrors } дзеркал · { $connections } активних з'єднань

# Variables:
#   $size (String) - Total amount fetched.
#   $time (String) - Elapsed time.
#   $rate (String) - Transfer rate.
progress-fetched = Отримано { $size } за { $time } ({ $rate }/s)

## TUI

tui-upgrade-title = Оновлення Nala
tui-history-title = Історія Nala
tui-pkgs = Пакунки
tui-help-move = (↑) вгору | (↓) вниз
tui-help-tabs = (→) наступна вкладка | (←) попередня вкладка
tui-help-show = (Enter) показати список змін | (s) показати інформацію про версію
tui-help-confirm = (q) вийти | (y) почати оновлення
tui-help-quit = (q) вийти
tui-changelog-missing = Не вдалося знайти URI списку змін

## Files, configuration, and package archives

# Variables:
#   $path (String) - File or directory path.
file-remove = Не вдалося вилучити { $path }
file-read = Не вдалося прочитати '{ $path }'
file-read-defaults = Не вдалося прочитати { $path }, використовуючи типові налаштування
file-parse-defaults = Не вдалося розібрати { $path }, використовуючи типові налаштування
config-parse = Не вдалося розібрати { $path }. Nala не ігноруватиме наявний файл конфігурації. Якщо це сталося після оновлення пакунка, порівняйте його з { $path }.dpkg-dist
file-create = Не вдалося створити '{ $path }'
file-write = Не вдалося записати до '{ $path }'
file-replace = Не вдалося замінити '{ $path }'
file-deserialize = Не вдалося десеріалізувати '{ $path }'

# Variables:
#   $operation (String) - Filesystem operation.
#   $path (String) - Source path.
fs-failed = Не вдалося виконати { $operation } { $path }

# Variables:
#   $operation (String) - Filesystem operation.
#   $path (String) - Source path.
#   $target (String) - Destination path.
fs-failed-target = Не вдалося виконати { $operation } { $path } => { $target }

# Variables:
#   $option (String) - Unsupported APT configuration override.
config-option = Опція '{ $option }' не підтримується
color-expected = назва кольору, індекс 0-255, або шістнадцятковий рядок #RRGGBB
color-index = індекс кольору має бути від 0 до 255
color-unknown = невідомий колір '{ $color }'
color-rgb-components = Rgb очікує три елементи
color-rgb-expected = очікується колір у форматі Rgb або Indexed
color-modifier-expected = рядок модифікатора на кшталт "BOLD | ITALIC" або список
color-modifier-unknown = невідомий модифікатор '{ $modifier }'

archive-unsupported = Тип архіву не підтримується

# Variables:
#   $path (String) - Debian archive path.
deb-control-missing = Файл 'control' не знайдено у { $path }

# Variables:
#   $type (String) - Hash type.
hash-unsupported = Тип контрольної суми '{ $type }' не підтримується

# Variables:
#   $package (String) - Package name.
#   $version (String) - Package version.
hash-unavailable =
    Неможливо перевірити цілісність { $package } { $version }.
    Для цього пакунка немає доступних контрольних сум.

## Dpkg

dpkg-child-failed = Дочірній процес Dpkg завершився з помилкою: { $error }
dpkg-status-utf8 = Дескриптор статусу Dpkg повернув некоректний UTF-8
dpkg-exit = Dpkg завершив роботу з кодом: '{ $code }'
dpkg-poll = Не вдалося опитати дочірній процес
dpkg-read-status = Не вдалося прочитати дескриптор статусу
dpkg-read-pty = Не вдалося прочитати з pty
dpkg-write-pty = Не вдалося надіслати stdin до pty
dpkg-removing = Вилучення:
dpkg-unpacking = Розпакування:
dpkg-setting-up = Налаштування:
dpkg-processing = Обробка:

## List

list-virtual = Віртуальний

## Show: general output

show-local-install = локальне встановлення

# Variables:
#   $count (Number) - Number of additional package records.
#   $switch (String) - Command-line switch that displays every record.
show-more-records = Є ще { $count } додаткових записів. Скористайтеся параметром { $switch }, щоб переглянути їх.

show-no-description = Немає опису
show-no-summary = Немає короткого опису

## Show: package record labels

show-package = Пакунок
show-version = Версія
show-architecture = Архітектура
show-priority = Пріоритет
show-essential = Важливий
show-section = Розділ
show-source = Джерело
show-installed-size = Розмір після встановлення
show-size = Розмір
show-maintainer = Супровідник
show-original-maintainer = Початковий супровідник
show-homepage = Домашня сторінка
show-sha256 = SHA256
show-archive = Архів
show-origin = Походження
show-codename = Кодова назва
show-component = Компонент
show-provides = Надає
show-description = Опис
show-attributes = Атрибути
show-apt-sources = Джерела APT
show-depends = Залежності
show-pre-depends = Попередні залежності
show-suggests = Пропонує
show-recommends = Рекомендує
show-conflicts = Конфліктує
show-replaces = Замінює
show-obsoletes = Застарілі
show-breaks = Порушує
show-enhances = Розширює

## Show: package attributes

show-attr-installed = Встановлено
show-attr-local = Локальний
show-attr-auto-removable = Можна автовилучити
show-attr-automatic = Встановлено автоматично

# Variables:
#   $version (String) - Package version available for upgrade.
show-attr-upgradable-to = Можна оновити до: { $version }

# Variables:
#   $version (String) - Currently installed package version.
show-attr-upgradable-from = Можна оновити з: { $version }

## Policy

policy-none = немає
policy-installed = Встановлено:
policy-candidate = Кандидат:
policy-version-table = Таблиця версій:
policy-no-versions = Немає версій.
policy-origin = походження
policy-package-files = Файли пакунків:
policy-pinned = Закріплені пакунки:
policy-release = випуск

## History: general output

# Variables:
#   $count (Number) - Number of history entries that were cleared.
history-cleared =
    { $count ->
        [one] Очищено { $count } запис історії.
        [few] Очищено { $count } записи історії.
       *[many] Очищено { $count } записів історії.
    }

history-empty = Записів історії не знайдено.

# Variables:
#   $id (Number) - ID of the history entry.
history-not-found = Запис історії з ID '{ $id }' не існує
history-not-replayable = Запис історії '{ $id }' неможливо повторити, оскільки його не було збережено як застосований
history-no-changes = Запис історії '{ $id }' не містить змін пакунків для повторення
history-cleared-entry = Очищено запис історії { $id }.

## History: labels

history-id = ID
history-command = Команда
history-date-time = Дата й час
history-requested-by = Ініціатор
history-altered = Змінено
history-status = Стан
history-status-applied = Застосовано
history-started = Розпочато
history-finished = Завершено
history-targets = Запитані цілі

## History: replay errors

# Variables:
#   $package (String) - Package name.
history-undo-version-missing = Скасування не підтримується для '{ $package }', оскільки попередню версію не було записано
history-undo-config-only = Скасування не підтримується для '{ $package }', оскільки відновлення стану "лише файли конфігурації" не реалізовано
history-undo-installed-version-missing = Скасування не підтримується для '{ $package }', оскільки попередню встановлену версію не було записано
history-undo-reinstall = Скасування не підтримується для '{ $package }', оскільки перевстановлення не має записаної зворотної дії
history-undo-held = Дії над утриманим пакунком '{ $package }' неможливо скасувати
history-redo-version-missing = Повторення не підтримується для '{ $package }', оскільки кінцеву версію не було записано
history-redo-reinstall-missing = Повторення не підтримується для '{ $package }', оскільки перевстановлену версію не було записано
history-redo-held = Дії над утриманим пакунком '{ $package }' неможливо повторити

## History: selector and storage errors

history-clear-target = Для очищення історії потрібен номер запису або --all
history-config-undo = Дії над налаштованим пакунком '{ $package }' неможливо скасувати
history-config-redo = Дії над налаштованим пакунком '{ $package }' неможливо повторити
history-serialize = Не вдалося серіалізувати запис історії

## Fetch

fetch-no-mirrors = Nala не вдалося знайти жодного дзеркала.
fetch-none-selected = Жодного дзеркала не вибрано.
fetch-release-detect = Виникла проблема з визначенням випуску.

# Variables:
#   $distro (String) - Distribution name.
fetch-unsupported = { $distro } не підтримується.

# Variables:
#   $file (String) - Sources file path.
fetch-sources-written = Джерела записано до { $file }

fetch-title = Nala Fetch
fetch-score = Час відповіді:
fetch-score-help = Час у мілісекундах, необхідний для завантаження файлу Release.
fetch-help = Використовуйте ↓↑ для переміщення, Пробіл для вибору, Home/End на початок/кінець, q/Enter для виходу.

## Update

# Variables:
#   $count (Number) - Number of upgradable packages.
#   $command (String) - Command that lists upgradable packages.
update-upgradable =
    { $count ->
        [one] Може бути оновлено { $count } пакунок. Запустіть '{ $command }', щоб переглянути його.
        [few] Може бути оновлено { $count } пакунки. Запустіть '{ $command }', щоб переглянути їх.
       *[many] Може бути оновлено { $count } пакунків. Запустіть '{ $command }', щоб переглянути їх.
    }

update-no-change = Без змін
update-updated = Оновлено
update-ignored = Проігноровано
update-downloading = Завантаження
update-processing = Обробка
update-item = { $state }: { $description }

## Upgrade

upgrade-exclude-unsafe =
    Вибрані пакунки неможливо безпечно виключити з оновлення.
    { $error }
upgrade-protect = Захист { $package } від { $reason }
upgrade-reason-upgrade = оновлення
upgrade-reason-auto-remove = автовилучення
upgrade-config-missing = Дерево конфігурації відсутнє!

## Install

install-downloaded = Завантажено: { $path }

## Additional messages

summary-protected = Наведені нижче пакунки є необхідними або захищеними!

summary-remove-protected = Ви спробували видалити необхідні або захищені пакунки

summary-downgrade-assume-yes = З параметром --assume-yes версії пакунків буде понижено. Якщо ви впевнені, використайте '-o APT::Get::allow-downgrades=true'.

summary-held-assume-yes = З параметром --assume-yes утримувані пакунки буде змінено. Якщо ви впевнені, використайте '-o APT::Get::allow-change-held-packages=true'.

autoremove-assume-yes = Автоматичне видалення з параметром --assume-yes пропущено. Щоб увімкнути його, використайте --autoremove.

download-no-space = Недостатньо вільного місця в { $path }: потрібно { $required }, доступно { $available }.
