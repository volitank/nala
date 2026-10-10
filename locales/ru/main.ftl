### Messages migrated from legacy/python/po/ru.po.
### Missing messages fall back to English.

log-error = Ошибка:

log-notice = Уведомление:

log-warning = Предупреждение:

prompt-continue = Хотите продолжить?

prompt-choice = Д/д Н/н

root-required = Нужен root чтобы { $command }

op-held = Удержать

reason-kept-back = Удержать

summary-title = Итого

summary-total-download = Общий размер загрузки:

summary-disk-required = Требуемое дисковое пространство:

summary-disk-free = Освобождаемое место на диске:

summary-total-download-value = Общий размер загрузки: { $size }

summary-disk-required-value = Требуемое дисковое пространство: { $size }

summary-disk-free-value = Освобождаемое место на диске: { $size }

summary-nothing = Нечего делать.

summary-essential = Следующие пакеты необходимы!

summary-remove-essential = Вы попытались удалить необходимые пакеты

summary-use-switch = Пожалуйста, используйте { $switch } если уверены что хотите этого.

summary-reboot = Необходима перезагрузка.

summary-reboot-packages = Следующие пакеты требуют перезагрузки.

table-package = Пакет:

table-version = Версия:

table-size = Размер:

table-old-version = Старая версия:

table-new-version = Новая версия:

pkg-no-candidate = { $package } не имеет кандидата на установку.

pkg-not-installed = { $package } не установлен

pkg-already-latest = { $package } уже имеет последнюю версию { $version }

pkg-version-missing = Версия { $version } не найдена для пакета { $package }

pkg-cache-missing = { $package }, но его нет в кэше

pkg-recorded-version-missing = не найдена версия для пакета { $package }

pkg-version-cache-missing = Версия { $version } не найдена для пакета { $package }

pkg-virtual-providers = { $package } это виртуальный пакет, предоставляемый:

pkg-virtual-one = Вы должны выбрать только один.

download-complete = Загрузка завершена:

download-auth-warning = Следующие пакеты не могут быть аутентифицированы!

download-auth-allowed = В конфигурации установлено разрешение на установку неаутентифицированных пакетов.

download-exit = Выход по вашему требованию.

progress-remaining = Оставшееся время

tui-pkgs = Пакеты

file-read-defaults = Невозможно прочитать конфигурационный файл: { $path }. Использование по умолчанию

hash-unsupported = Тип хэша '{ $type }' не поддерживается

dpkg-removing = Удаление:

dpkg-unpacking = Распаковка:

dpkg-setting-up = Настройка:

dpkg-processing = Обработка:

list-virtual = Виртуальный пакет

show-local-install = локальная установка

show-more-records = Имеется { $count } дополнительных записей. Используйте { $switch } для просмотра.

show-no-description = Нет описания

show-package = Пакет

show-version = Версия

show-architecture = Архитектура

show-priority = Приоритет

show-essential = Необходимо

show-section = Раздел

show-installed-size = Установленный размер

show-homepage = Домашняя страница

show-provides = Предоставляет

show-description = Описание

show-apt-sources = APT-Источники

show-depends = Зависит от

show-pre-depends = Предварительные зависимости

show-suggests = Предложения

show-recommends = Рекомендуется

show-conflicts = Конфликты

show-replaces = Замены

show-breaks = Брейки

show-enhances = Расширяет

show-attr-installed = Установлено

show-attr-upgradable-to = установлен и может быть обновлен до { $version }

show-attr-upgradable-from = может быть обновлен до { $version }

policy-installed = Установлено:

history-empty = Не существует истории.

history-not-found = Транзакция { $id } не существует.

fetch-no-mirrors = Nala не смогла найти ни одного зеркала.

fetch-release-detect = Возникла проблема с обнаружением релиза.

fetch-sources-written = Источники были записаны в { $file }

fetch-score-help = Score - количество миллисекунд для загрузки файла релиза

update-upgradable = { $count } пакетов могут быть обновлены. Выполните '{ $command }' чтобы посмотреть их.

update-no-change = Без изменений

update-updated = Обновлено

update-ignored = Проигнорировано

update-downloading = Загрузка

update-processing = Обработка

install-downloaded = Загружено: { $path }

## Additional messages

unknown = Неизвестно

none = Нет

error = Ошибка

mirrors = Зеркала:

log-info = Информация:

log-verbose = Подробно:

log-debug = Отладка:

not-implemented = Пока не реализовано.

subcommand-missing = Подкоманда не указана

prompt-invalid = «{ $response }» — недопустимый ответ

prompt-refused = Пользователь отказался от подтверждения

release-info-title = Информация о репозитории изменена

release-info-explanation =
    Идентификационная информация репозитория изменилась в его файле Release.
    Просмотрите изменения, прежде чем продолжить.

release-info-repository = Репозиторий

release-info-distribution = Дистрибутив

release-info-changes = Изменения

release-info-confirm = Принять эти изменения и продолжить обновление?

release-info-help = ({ $yes }) принять | ({ $no }/Enter/Esc) отклонить

release-info-accepted = Изменения информации о репозитории { $repository } приняты.

release-info-rejected = Изменения информации о репозитории { $repository } отклонены.

release-info-noninteractive = Изменения информации Release необходимо принять явно. Для этого запустите команду повторно с параметром «-o Acquire::AllowReleaseInfoChange=true».

op-remove = Удалить

op-auto-remove = Автоматически удалить

op-purge = Удалить с настройками

op-auto-purge = Автоматически удалить с настройками

op-install = Установить

op-reinstall = Переустановить

op-upgrade = Обновить

op-downgrade = Понизить версию

op-configure = Настроить

reason-excluded = Исключено

reason-manual-hold = Удерживается вручную

reason-phased = Поэтапное обновление

reason-phased-percent = Поэтапное обновление { $percentage }%

summary-op-count = { $operation } { $count }

summary-op-count-colon = { $operation }: { $count }

summary-protected = Следующие пакеты являются необходимыми или защищёнными!

summary-remove-protected = Вы попытались удалить необходимые или защищённые пакеты

summary-downgrade-assume-yes = С параметром --assume-yes версии пакетов будут понижены. Если вы уверены, используйте «-o APT::Get::allow-downgrades=true».

summary-held-assume-yes = С параметром --assume-yes удерживаемые пакеты будут изменены. Если вы уверены, используйте «-o APT::Get::allow-change-held-packages=true».

autoremove-config-purge = Параметр --remove-config необходимо использовать с --purge или autopurge.

autoremove-assume-yes = Автоматическое удаление с параметром --assume-yes пропущено. Используйте --autoremove, чтобы включить его.

table-reason = Причина:

pkg-not-found = Пакет «{ $package }» не найден

pkg-not-found-cache = Некоторые пакеты не найдены в кэше

pkg-mark-install = Не удалось отметить «{ $package }» для установки

pkg-reinstall-missing = Пакет { $package } не установлен, поэтому его нельзя переустановить

pkg-invalid-name = Недопустимое имя пакета: «{ $package }»

pkg-op-conflict = Конфликтующие операции для «{ $package }»: { $operation-a } и { $operation-b }

pkg-version-conflict = Конфликтующие закреплённые версии для «{ $package }»: { $version-a } и { $version-b }

pkg-invalid-pin = Недопустимое закрепление версии: «{ $pin }»

pkg-pin-exact = Для закрепления версии требуется точное имя пакета: «{ $pin }»

pkg-exclude-missing = Шаблон исключения «{ $pattern }» не соответствует ни одному пакету

pkg-virtual-no-providers = Пакет { $package } является исключительно виртуальным и не предоставляется другими пакетами

pkg-virtual-select = Выбор { $provider } вместо виртуального пакета { $package }

pkg-select = Выбор пакета «{ $selected }» вместо «{ $requested }»

pkg-fix-broken = Не удалось исправить повреждённые пакеты

pkg-required = Необходимо указать пакет

download-failed = Загрузки завершились неудачей

download-some-missing = Некоторые пакеты не найдены.

download-auth-required =
    Не удалось проверить подлинность некоторых пакетов.
    Если вы уверены, используйте { $switch }

download-items = Элементы

download-source-missing = Не удалось найти источник для загрузки версии «{ $version }» пакета «{ $package }»

download-written = Пакет { $package } записан в { $path }

download-hash-missing = Хеш для «{ $uri }» не найден

download-protocol-missing = Протокол не определён

download-uri-missing = URI не определён

download-malformed = «{ $uri }» имеет неверный формат!

download-request-failed = Запрос на загрузку «{ $uri }» завершился неудачей

download-stream-failed = Не удалось получить поток данных из «{ $uri }»

download-get = Не удалось отправить запрос на загрузку

download-content-length = В { $headers } отсутствует content-length

download-content-str = Не удалось преобразовать content-length в строку: { $headers }

download-content-parse = Не удалось разобрать content-length: { $headers }

download-checksum = Контрольная сумма файла { $file } не совпала

download-no-uris = Не удалось загрузить { $file } ни по одному URI

download-no-space = Недостаточно свободного места в { $path }: требуется { $required }, доступно { $available }.

download-filename = У { $path } нет допустимого имени файла!

download-error = Ошибка: { $error }

download-select-domain = Выбор { $domain } для { $file }

download-start = Начало: { $uri }, Повторные попытки: { $retries }

download-finished = Завершено: { $uri }

progress-working = Выполняется...

progress-nothing = Нечего загружать

progress-label = Ход выполнения

progress-total = Всего

progress-data = Данные

progress-packages = Пакеты

progress-package = Пакет

progress-status = Статус

progress-mirrors = Зеркала

progress-speed = Скорость

progress-elapsed = Прошло

progress-finished = Завершено:

progress-update-title = Обновление списков пакетов

progress-download-title = Загрузка пакетов

progress-install-title = Установка пакетов

progress-score-title = Проверка зеркал

progress-last-completed = Последняя завершённая задача

progress-connections = Соединения

progress-average = Среднее

progress-active = Активно

progress-state = Состояние

progress-starting = запуск

progress-downloading = загрузка

progress-idle = бездействие

progress-more-mirrors = ещё +{ $mirrors } зеркал · { $connections } активных соединений

progress-fetched = Загружено { $size } за { $time } ({ $rate }/s)

tui-upgrade-title = Обновление Nala

tui-history-title = История Nala

tui-help-move = (↑) вверх | (↓) вниз

tui-help-tabs = (→) следующая вкладка | (←) предыдущая вкладка

tui-help-show = (Enter) показать список изменений | (s) показать информацию о версии

tui-help-confirm = (q) выйти | (y) начать обновление

tui-help-quit = (q) выйти

tui-changelog-missing = Не удалось найти URI списка изменений

file-remove = Не удалось удалить { $path }

file-read = Не удалось прочитать «{ $path }»

file-parse-defaults = Не удалось разобрать { $path }, используются значения по умолчанию

config-parse = Не удалось разобрать { $path }. Nala не будет игнорировать существующий файл конфигурации. Если это произошло после обновления пакета, сравните его с { $path }.dpkg-dist

file-create = Не удалось создать «{ $path }»

file-write = Не удалось записать «{ $path }»

file-replace = Не удалось заменить «{ $path }»

file-deserialize = Не удалось десериализовать «{ $path }»

fs-failed = Не удалось выполнить операцию { $operation } для { $path }

fs-failed-target = Не удалось выполнить операцию { $operation }: { $path } => { $target }

config-option = Параметр «{ $option }» не поддерживается

color-expected = название цвета, индекс от 0 до 255 или шестнадцатеричная строка #RRGGBB

color-index = индекс цвета должен быть от 0 до 255

color-unknown = неизвестный цвет «{ $color }»

color-rgb-components = Rgb требует три компонента

color-rgb-expected = ожидался цвет Rgb или Indexed

color-modifier-expected = строка модификаторов, например "BOLD | ITALIC", или массив

color-modifier-unknown = неизвестный модификатор «{ $modifier }»

archive-unsupported = Тип архива не поддерживается

deb-control-missing = Файл control не найден в { $path }

hash-unavailable =
    Невозможно проверить целостность { $package } { $version }.
    Для этого пакета нет доступных хешей.

dpkg-child-failed = Дочерний процесс dpkg завершился неудачей: { $error }

dpkg-status-utf8 = Дескриптор состояния dpkg вернул недопустимый UTF-8

dpkg-exit = Dpkg завершился с кодом: «{ $code }»

dpkg-poll = Не удалось опросить дочерний процесс

dpkg-read-status = Не удалось прочитать дескриптор состояния

dpkg-read-pty = Не удалось прочитать данные из псевдотерминала

dpkg-write-pty = Не удалось передать стандартный ввод в псевдотерминал

show-no-summary = Нет краткого описания

show-source = Источник

show-size = Размер

show-maintainer = Сопровождающий

show-original-maintainer = Первоначальный сопровождающий

show-sha256 = SHA256

show-archive = Архив

show-origin = Происхождение

show-codename = Кодовое имя

show-component = Компонент

show-attributes = Атрибуты

show-obsoletes = Делает устаревшими

show-attr-local = Локальный

show-attr-auto-removable = Можно удалить автоматически

show-attr-automatic = Автоматический

policy-none = нет

policy-candidate = Кандидат:

policy-version-table = Таблица версий:

policy-no-versions = Нет версий.

policy-origin = происхождение

policy-package-files = Файлы пакетов:

policy-pinned = Закреплённые пакеты:

policy-release = выпуск

history-cleared =
    { $count ->
        [one] Удалена { $count } запись истории.
        [few] Удалены { $count } записи истории.
        [many] Удалено { $count } записей истории.
       *[other] Удалено { $count } записи истории.
    }

history-not-replayable = Запись истории «{ $id }» нельзя повторить, поскольку она не была сохранена как применённая

history-no-changes = В записи истории «{ $id }» нет изменений пакетов для повторения

history-cleared-entry = Удалена запись истории { $id }.

history-id = ID

history-command = Команда

history-date-time = Дата и время

history-requested-by = Инициатор

history-altered = Изменено

history-status = Статус

history-status-applied = Применено

history-started = Начато

history-finished = Завершено

history-targets = Запрошенные цели

history-undo-version-missing = Отмена для «{ $package }» не поддерживается, поскольку предыдущая версия не была записана

history-undo-config-only = Отмена для «{ $package }» не поддерживается, поскольку восстановление состояния только с файлами конфигурации не реализовано

history-undo-installed-version-missing = Отмена для «{ $package }» не поддерживается, поскольку ранее установленная версия не была записана

history-undo-reinstall = Отмена для «{ $package }» не поддерживается, поскольку для переустановки не записана обратная операция

history-undo-held = Нельзя отменить удерживаемый пакет «{ $package }»

history-redo-version-missing = Повтор для «{ $package }» не поддерживается, поскольку итоговая версия не была записана

history-redo-reinstall-missing = Повтор для «{ $package }» не поддерживается, поскольку переустановленная версия не была записана

history-redo-held = Нельзя повторить удерживаемый пакет «{ $package }»

history-clear-target = Для очистки истории требуется указать запись или --all

history-config-undo = Нельзя отменить настроенный пакет «{ $package }»

history-config-redo = Нельзя повторить настроенный пакет «{ $package }»

history-serialize = Не удалось сериализовать запись истории

fetch-none-selected = Зеркала не выбраны.

fetch-unsupported = Дистрибутив { $distro } не поддерживается.

fetch-title = Выбор зеркал Nala

fetch-score = Оценка:

fetch-help = Используйте ↓↑ для перемещения, Пробел для выбора/снятия выбора, Home/End для перехода в начало/конец и q/Enter для выхода.

update-item = { $state }: { $description }

upgrade-exclude-unsafe =
    Выбранные пакеты нельзя безопасно исключить из обновления.
    { $error }

upgrade-protect = Защита { $package } от { $reason }

upgrade-reason-upgrade = обновление

upgrade-reason-auto-remove = автоматическое удаление

upgrade-config-missing = Нет дерева конфигурации!
