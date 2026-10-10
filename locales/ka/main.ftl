### Georgian messages displayed by Nala.

## General labels and errors

unknown = უცნობი
none = არცერთი
error = შეცდომა
mirrors = სარკეები:

log-error = შეცდომა:
log-notice = შეტყობინება:
log-warning = გაფრთხილება:
log-info = ინფორმაცია:
log-verbose = დიაგნოსტიკა:
log-debug = გამართვა:

not-implemented = ჯერ განხორციელებული არაა.
subcommand-missing = ქვებრძანება მოწოდებული არაა

# Variables:
#   $response (String) - Invalid confirmation response.
prompt-invalid = '{ $response }' სწორი პასუხი არაა
prompt-refused = მომხმარებელმა უარი თქვა თანხმობაზე
prompt-continue = გააგრძელებთ?
prompt-choice = Y/y N/n

# Variables:
#   $command (String) - Command that requires elevated privileges.
root-required = Nala-ს სჭირდება root-ის უფლებები ბრძანებისთვის { $command }

## Transaction operations and reasons

op-remove = წაშლა
op-auto-remove = ავტოწაშლა
op-purge = გასუფთავება
op-auto-purge = ავტოგასუფთავება
op-install = დაყენება
op-reinstall = თავიდან დაყენება
op-upgrade = განახლება
op-downgrade = ვერსიის დაწევა
op-configure = მორგება
op-held = ვერსიის შეჩერება

reason-excluded = ამოღებულია
reason-manual-hold = ხელითაა შეჩერებული
reason-phased = ეტაპობრივი
reason-kept-back = შეჩერებულია

# Variables:
#   $percentage (Number) - Phased update percentage.
reason-phased-percent = ეტაპობრივი { $percentage }%

## Transaction summary

summary-title = შეჯამება
summary-total-download = ჩამოტვირთვის სრული ზომა:
summary-disk-required = აუცილებელი ადგილი დისკზე:
summary-disk-free = გასათავისუფლებელი ადგილი დისკზე:

summary-total-download-value = ჩამოტვირთვის სრული ზომა: { $size }
summary-disk-required-value = აუცილებელი ადგილი დისკზე: { $size }
summary-disk-free-value = გასათავისუფლებელი ადგილი დისკზე: { $size }
summary-op-count = { $operation } { $count }
summary-op-count-colon = { $operation }: { $count }

summary-nothing = გასაკეთებელი არაფერია.
summary-essential = შემდეგი პაკეტები ძირითადია!
summary-remove-essential = თქვენ დააპირეთ, წაშალოთ ძირითადი პაკეტი

# Variables:
#   $switch (String) - Command-line switch that permits essential package removal.
summary-use-switch = თუ დარწმუნებული ბრძანდებით, გამოიყენეთ '{ $switch }'.

summary-reboot = ამ ცვლილებების დასასრულებლად საჭიროა გადატვირთვა.
summary-reboot-packages = გადატვირთვა სჭირდებათ შემდეგ პაკეტებს:

## Package summary table

table-package = პაკეტი:
table-version = ვერსია:
table-old-version = ძველი ვერსია:
table-new-version = ახალი ვერსია:
table-reason = მიზეზი:
table-size = ზომა:

## Package selection

# Variables:
#   $package (String) - Package name or selection token.
pkg-not-found = '{ $package }' აღმოჩენილი არაა
pkg-not-found-cache = ზოგიერთი პაკეტი კეშში აღმოჩენილი არაა
pkg-no-candidate = { $package }-ს დასაყენებელი კანდიდატი არ აქვს
pkg-mark-install = '{ $package }'-ის დასაყენებლად მონიშვნა შეუძლებელია
pkg-not-installed = { $package } დაყენებული არაა
pkg-reinstall-missing = { $package } დაყენებული არაა, ასე რომ მას თავიდან ვერ დააყენებთ
pkg-invalid-name = არასწორი პაკეტის სახელი: '{ $package }'

# Variables:
#   $package (String) - Package name.
#   $operation-a (String) - First requested operation.
#   $operation-b (String) - Conflicting requested operation.
pkg-op-conflict = კონფლიქტური ოპერაციები პაკეტისთვის '{ $package }': { $operation-a } vs { $operation-b }

# Variables:
#   $package (String) - Package name.
#   $version-a (String) - First requested version.
#   $version-b (String) - Conflicting requested version.
pkg-version-conflict = კონფლიქტური ამაგრებული ვერსიები პაკეტისთვის '{ $package }': { $version-a } vs { $version-b }

# Variables:
#   $package (String) - Package name.
#   $version (String) - Package version.
pkg-already-latest = { $package }{ $version } უკვე დაყენებულია და მისი ვერსია უკანასკნელია
pkg-version-missing = ვერ ვიპოვე ვერსია '{ $version }' პაკეტისთვის '{ $package }'
pkg-cache-missing = პაკეტი '{ $package }' კეშში აღმოჩენილი არაა
pkg-recorded-version-missing = პაკეტისთვის '{ $package }' ვერსია ჩაწერილი არაა
pkg-version-cache-missing = ვერსია '{ $version }' პაკეტისთვის '{ $package }' აღმოჩენილი არაა

# Variables:
#   $pin (String) - Version pin supplied by the user.
pkg-invalid-pin = არასწორი ვერსიის ამაგრება: '{ $pin }'
pkg-pin-exact = ვერსიის ამაგრებას პაკეტის ზუსტის სახელი სჭირდება: '{ $pin }'

# Variables:
#   $pattern (String) - Package exclusion pattern.
pkg-exclude-missing = ამოღების ნიმუში '{ $pattern }' არც ერთ პაკეტს ემთხვევა

# Variables:
#   $package (String) - Virtual package name.
pkg-virtual-no-providers = პაკეტს { $package } მომწოდებელი არ ჰყავს და ის წმინდად ვირტუალურია
pkg-virtual-providers = { $package } ვირტუალური პაკეტია, რომელსაც მოგაწვდით:

# Variables:
#   $package (String) - Virtual package name.
#   $provider (String) - Selected concrete provider.
pkg-virtual-select = არჩეულია პაკეტი { $provider } ვირტუალური პაკეტის { $package } მაგიერ
pkg-virtual-one = ზუსტად ერთი უნდა აირჩიოთ.

# Variables:
#   $selected (String) - Selected package name.
#   $requested (String) - Originally requested package name.
pkg-select = არჩეულია პაკეტი '{ $selected }' პაკეტის '{ $requested }' მაგიერ

pkg-fix-broken = გაფუჭებული პაკეტების შეკეთება შეუძლებელია
pkg-required = პაკეტის მითითება აუცილებელია

## Download output

download-complete = გადმოწერები დასრულდა:
download-failed = გადმოწერები ჩავარდა
download-some-missing = ზოგიერთი პაკეტი აღმოჩენილი არაა.
download-auth-warning = შემდეგი პაკეტების ავთენტიკაცია შეუძლებელია!
download-auth-allowed = კონფიგურაციაში აყენია, რომ ავთენტიკაცია არგავლილი პაკეტების დაყენება მაინც მოხდება.
download-auth-required =
    ზოგიერთი პაკეტის ავთენტიკაცია შეუძლებელია.
      თუ დარწმუნებული ბრძანდებით, გამოიყენეთ { $switch }
download-exit = გასვლა მომხმარებლის მოთხოვნის გამო
download-items = ელემენტები

# Variables:
#   $package (String) - Package name.
#   $version (String) - Package version.
download-source-missing = შეუძლებელია წყაროს აღმოჩენა, რომ გადმოვწერო ვერსია '{ $version }' პაკეტისთვის '{ $package }'

# Variables:
#   $package (String) - Package name.
#   $path (String) - Destination path.
download-written = { $package } ჩაწერილია საქაღალდეში { $path }

# Variables:
#   $uri (String) - Download URI.
download-hash-missing = ბმულისთვის '{ $uri }' ჰეში აღმოჩენილი არაა
download-protocol-missing = პროტოკოლი მითითებული არაა
download-uri-missing = uri მითითებული არაა
download-malformed = '{ $uri }' არასწორადაა ჩამოყალიბებული!
download-request-failed = გადმოწერის მოთხოვნა ჩავარდა uri-სთვის '{ $uri }'
download-stream-failed = მონაცემების გადმოწერის შეცდომა uri-დან '{ $uri }'
download-get = გადმოწერის მოთხოვნის გაგზავნა შეუძლებელია
download-content-length = content-length არ არსებობს { $headers }-ში
download-content-str = content-length-ის სტრიქონში გადაყვანა ჩავარდა: { $headers }
download-content-parse = content-length-ის დამუშავება ჩავარდა: { $headers }

# Variables:
#   $file (String) - Downloaded filename.
download-checksum = საკონტროლო ჯამი არ ემთხვევა ფაილისთვის { $file }
download-no-uris = ფაილისთვის { $file } URI-ების გადმოწერა შეუძლებელია

# Variables:
#   $path (String) - Path without a valid filename.
download-filename = ბილიკს { $path } სწორი ფაილის სახელი არ აქვს!

download-error = შეცდომა: { $error }
download-select-domain = ვირჩევ { $domain } ფაილისთვის { $file }
download-start = ვიწყებ: { $uri }, თავიდან ცდები: { $retries }
download-finished = დასრულდა: { $uri }

## Progress

progress-working = ვმუშაობ...
progress-nothing = გამოსათხოვი არაფერია
progress-label = მიმდინარეობა
progress-total = ჯამში
progress-speed = სიჩქარე
progress-elapsed = გასულია
progress-remaining = დარჩენილია
progress-finished = დასრულდა:

# Variables:
#   $size (String) - Total amount fetched.
#   $time (String) - Elapsed time.
#   $rate (String) - Transfer rate.
progress-fetched = გამოთხოვილია { $size } დროში { $time } ({ $rate }/წმ)

## TUI

tui-upgrade-title = Nala-ის განახლება
tui-history-title = Nala-ის ისტორია
tui-pkgs = პაკეტები
tui-help-move = (↑) ასვლა | (↓) ჩამოსვლა
tui-help-tabs = (→) შემდეგი ჩანართი | (←) წინა ჩანართი
tui-help-show = (Enter) ცვლილებების ჟურნალის ჩვნებეა | (s) ვერსიის ინფორმაციის ჩვენება
tui-help-confirm = (q) გასვლა | (y) განახლების დაწყება
tui-help-quit = (q) გასვლა
tui-changelog-missing = ცვლილებების ჟურნალის URI-ის აღმოჩენა შეუძლებელია

## Files, configuration, and package archives

# Variables:
#   $path (String) - File or directory path.
file-remove = ვერ წავშალე { $path }
file-read = ვერ წავიკითხე '{ $path }'
file-read-defaults = ვერ წავიკითხე { $path }, გამოიყენება ნაგულისხმევი მნიშვნელობები
file-parse-defaults = ვერ დავამუშავე { $path }, გამოიყენება ნაგულისხმევი მნიშვნელობები
file-create = ვერ შევქმენი '{ $path }'
file-write = ვერ ჩავწერე ბილიკზე '{ $path }'
file-replace = ვერ ჩავანაცვლე '{ $path }'
file-deserialize = ბილიკის '{ $path }' დესერიალიზაცია შეუძლებელია

# Variables:
#   $operation (String) - Filesystem operation.
#   $path (String) - Source path.
fs-failed =  { $operation } ჩავარდა ბილიკისთვის { $path }

# Variables:
#   $operation (String) - Filesystem operation.
#   $path (String) - Source path.
#   $target (String) - Destination path.
fs-failed-target = { $operation } ჩავარდა ბილიკისთვის { $path } => { $target }

# Variables:
#   $option (String) - Unsupported APT configuration override.
config-option = პარამეტრი '{ $option }' მხარდაჭერილი არაა
color-expected = ფერის სახელი, 0-255 ინდექსი, ან #RRGGBB თექვსმეტობითი სტრიქონი
color-index = ფერის ინდექსი უნდა იყოს შუალედიდან 0 - 255
color-unknown = უცნობი ფერი '{ $color }'
color-rgb-components = Rgb მოელის სამ კომპონენტს
color-rgb-expected = მოველოდი Rgb, ან დაინდექსებულ ფერს
color-modifier-expected = ისეთ მოდიფიკატორ ტექსტს, როგორიცაა "BOLD | ITALIC" ან მასივს
color-modifier-unknown = უცნობი მოდიფიკატორი '{ $modifier }'

archive-unsupported = Archive type is not supported

# Variables:
#   $path (String) - Debian archive path.
deb-control-missing = ბილიკზე { $path } ფაილი control აღმოჩენილი არაა

# Variables:
#   $type (String) - Hash type.
hash-unsupported = ჰეშის ტიპი: { $type } მხარდაჭერილი არაა

# Variables:
#   $package (String) - Package name.
#   $version (String) - Package version.
hash-unavailable =
    { $package } { $version } მთლიანობის შემოწმება შეუძლებელია.
    ამ პაკეტისტვის ჰეშები ხელმისაწვდომი არაა.

## Dpkg

dpkg-child-failed = Dpkg-ის შვილის შეცდომა: { $error }
dpkg-status-utf8 = Dpkg-ის სტატუსის fd-მა დააბრუნა არასწორი UTF-8
dpkg-exit = Dpkg გავიდა კოდით: '{ $code }'
dpkg-poll = შვილის გამოკითხვა შეუძლებელია
dpkg-read-status = სტატუსის Fd-ის წაკითხვა შეუძლებელია
dpkg-read-pty = pty-დან წაკითხვა შეუძლებელია
dpkg-write-pty = stdin-ის pty-ზე გაგზავნა შეუძლებელია
dpkg-removing = მიმდინარეობს წაშლა:
dpkg-unpacking = მიმდინარეობს ამოღება:
dpkg-setting-up = მიმდინარეობს მორგება:
dpkg-processing = მიმდინარეობს დამუშავება:

## List

list-virtual = ვირტუალური

## Show: general output

show-local-install = ლოკალურად დაყენება

# Variables:
#   $count (Number) - Number of additional package records.
#   $switch (String) - Command-line switch that displays every record.
show-more-records = არსებობს { $count } დამატებიტი ჩანაწერი. მათ სანახავად გამოიყენეთ პარამეტრი { $switch }.

show-no-description = აღწერის გარეშე
show-no-summary = შეჯამების გარეშე

## Show: package record labels

show-package = პაკეტი
show-version = ვერსია
show-architecture = არქიტექტურა
show-priority = პრიორიტეტი
show-essential = ძირითადი
show-section = განყოფილება
show-source = წყარო
show-installed-size = დაყენებული-ზომა
show-size = ზომა
show-maintainer = წამყვანი პროგრამისტი
show-original-maintainer = ორიგინალი-წამყვანი-პროგრამისტი
show-homepage = ვებგვერდი
show-sha256 = SHA256
show-archive = არქივი
show-origin = წყარო
show-codename = კოდური სახელი
show-component = კომპონენტი
show-provides = მოგაწვდით
show-description = აღწერა
show-attributes = ატრიბუტები
show-apt-sources = APT-წყაროები
show-depends = დამოკიდებულია
show-pre-depends = პრედამოკიდებულია
show-suggests = გირჩევთ
show-recommends = რეკომენდაციას უწევს
show-conflicts = კონფლიქტშია
show-replaces = ჩაანაცვლებს
show-obsoletes = მოძველებულად მონიშნავს
show-breaks = გააფუჭებს
show-enhances = გააფართოვებს

## Show: package attributes

show-attr-installed = დაყენებულია
show-attr-local = ლოკალური
show-attr-auto-removable = ავტოწაშლადი
show-attr-automatic = ავტომატური

# Variables:
#   $version (String) - Package version available for upgrade.
show-attr-upgradable-to = განახლებადია ვერსიამდე: { $version }

# Variables:
#   $version (String) - Currently installed package version.
show-attr-upgradable-from = განახლებადია ვერსიიდან: { $version }

## Policy

policy-none = არცერთი
policy-installed = დაყენებულია:
policy-candidate = კანდიდატი:
policy-version-table = ვერსიების ცხრილი:
policy-no-versions = ვერსიების გარეშე.
policy-origin = წყარო
policy-package-files = პაკეტის ფაილები:
policy-pinned = ამაგრებული პაკეტები:
policy-release = გამოცემა

## History: general output

# Variables:
#   $count (Number) - Number of history entries that were cleared.
history-cleared =
    { $count ->
        [one] გასუფტავდა { $count } ისტორიის ჩანაწერი.
       *[other] გასუფთავდა { $count } ისტორიის ჩანაწერი.
    }

history-empty = ისტორიის ჩანაწერები აღმოჩენილი არაა.

# Variables:
#   $id (Number) - ID of the history entry.
history-not-found = ისტორიის ჩანაწერი ID-ით '{ $id }' არ არსებობს
history-not-replayable = ისტორიის ჩანაწერი '{ $id }' გამეორებადი არაა, რადგან ის არაა ჩაწერილი, როგორც გადატარებული
history-no-changes = ისტორიის ჩანაწერს '{ $id }' პაკეტის ცვლილებები გასამეორებლად არ აქვს
history-cleared-entry = გასუფთავდა ისტორიის ჩანაწერი { $id }.

## History: labels

history-id = ID
history-command = ბრძანება
history-date-time = თარიღი და დრო
history-requested-by = მოთხოვნის-ავტორი
history-altered = შეცვალა
history-status = სტატუსი
history-status-applied = გადატარებულია
history-started = დაწყებულია
history-finished = დასრულდა
history-targets = მოთხოვნილი სამიზნეები

## History: replay errors

# Variables:
#   $package (String) - Package name.
history-undo-version-missing = პაკეტისთვის '{ $package }' ბოლო ბრძანების გაუქმება შეუძლებელია, რადგან წინა ბრძანება ჩაწერილი არაა
history-undo-config-only = პაკეტისთვის '{ $package }' ბოლო ბრძანების გაუქმება შეუძლებელია, რადგან მხოლოდ კონფიგურაციის ფაილების მდგომარეობის აღდგენა ჯერ განხორციელებული არაა
history-undo-installed-version-missing = პაკეტისთვის '{ $package }' ბოლო ბრძანების გაუქმება შეუძლებელია, რადგან წინა დაყენებული ვერსი ჩაწერილი არ იყო
history-undo-reinstall = პაკეტისთვის '{ $package }' ბოლო ბრძანების გაუქმება შეუძლებელია, რადგან თავიდან დაყენებას წინა ვერსია ჩაწერილი არ ჰქონდა
history-undo-held = შეკავებული პაკეტისთვის '{ $package }' ბოლო ბრძანებას ვერ გააუქმებთ
history-redo-version-missing = პაკეტისთვის '{ $package }' ბრძანების გამეორება შეუძლებელია, რადგან მიღებული ვერსია ჩაწერილი არ იყო
history-redo-reinstall-missing = პაკეტისთვის '{ $package }' ბრძანების გამეორება შეუძლებელია, რადგან თავიდან დაყენებული ვერსია ჩაწერილი არ იყო
history-redo-held = შეკავებული პაკეტისთვის '{ $package }' ბრძანების გამეორება შეუძლებელია

## History: selector and storage errors

history-clear-target = ისტორიის გასუფთავებას სჭირდება ან ჩანაწერის ამრჩევი, ან --all
history-config-undo = მორგებული პაკეტის '{ $package }' გაუქმება შეუძლებელია
history-config-redo = მორგებული პაკეტის '{ $package }' გამეორება შეუძლებელია
history-serialize = ისტორიის ჩანაწერის სერიალიზაცია შეუძლებელია

## Fetch

fetch-no-mirrors = Nala-მა სარკეები ვერ იპოვა.
fetch-none-selected = სარკეები არჩეული არ ყოფილა.
fetch-release-detect = გამოცემის დადგენის შეცდომა.

# Variables:
#   $distro (String) - Distribution name.
fetch-unsupported = { $distro } მხარდაჭერილი არაა.

# Variables:
#   $file (String) - Sources file path.
fetch-sources-written = წყარო ჩაიწერა ფაილსი { $file }

fetch-title = Nala გამოთხოვა
fetch-score = ქულა:
fetch-score-help = ქულა არის რიცხვი, რამდენი მილიწამი დასწირდა ფაილის Release გადმოწერას.
fetch-help = გამოიყენეთ ↓↑ გადასაადგილებლად, Space მოსანიშნად/მონიშვნის გასაუქმებლად, Home/End თავში/ბოლოში გადასასვლელად, q/Enter გასასვლელად.

## Update

# Variables:
#   $count (Number) - Number of upgradable packages.
#   $command (String) - Command that lists upgradable packages.
update-upgradable =
    { $count ->
        [one] { $count } პაკეტის განახლება შეუძლებელია. მათ სანახავად გამოიყენეთ ბრძანება '{ $command }'.
       *[other] { $count } პაკეტის განახლება შეუძლებელია. მათ სანახავად გამოიყენეთ ბრძანება '{ $command }'.
    }

update-no-change = ცვლილების გარეშე
update-updated = განახლდა
update-ignored = გამოტოვებულია
update-downloading = მიმდინარეობს გადმოწერა
update-processing = მიმდინარეობს დამუშავება
update-item = { $state }: { $description }

## Upgrade

upgrade-exclude-unsafe =
    მონიშნული პაკეტების განახლებიდან უსაფრთხოდ ამოღება შეუძლებელია.
    { $error }
upgrade-protect = პაკეტის { $package } დაცვა ბრძანებისგან { $reason }
upgrade-reason-upgrade = განახლება
upgrade-reason-auto-remove = ავტოწაშლა
upgrade-config-missing = კონფიგურაციის ხის გარეშე!

## Install

install-downloaded = გადმოწერილია: { $path }

## Additional messages

release-info-title = რეპოზიტორიის ინფორმაცია შეიცვალა

release-info-explanation =
    რეპოზიტორიის საიდენტიფიკაციო ინფორმაცია მის Release ფაილში შეიცვალა.
    გაგრძელებამდე გადახედეთ ცვლილებებს.

release-info-repository = რეპოზიტორია

release-info-distribution = დისტრიბუტივი

release-info-changes = ცვლილებები

release-info-confirm = მივიღოთ ეს ცვლილებები და გავაგრძელოთ განახლება?

release-info-help = ({ $yes }) მიღება | ({ $no }/Enter/Esc) უარყოფა

release-info-accepted = მიღებულია რეპოზიტორიის ინფორმაციის ცვლილებები: { $repository }.

release-info-rejected = უარყოფილია რეპოზიტორიის ინფორმაციის ცვლილებები: { $repository }.

release-info-noninteractive = Release ინფორმაციის ცვლილებები აშკარად უნდა მიიღოთ. მათ მისაღებად თავიდან გაუშვით პარამეტრით '-o Acquire::AllowReleaseInfoChange=true'.

summary-protected = შემდეგი პაკეტები აუცილებელია ან დაცულია!

summary-remove-protected = თქვენ სცადეთ აუცილებელი ან დაცული პაკეტების წაშლა

summary-downgrade-assume-yes = --assume-yes პარამეტრით პაკეტების ვერსიები დაქვეითდება. თუ დარწმუნებული ხართ, გამოიყენეთ '-o APT::Get::allow-downgrades=true'.

summary-held-assume-yes = --assume-yes პარამეტრით შეჩერებული პაკეტები შეიცვლება. თუ დარწმუნებული ხართ, გამოიყენეთ '-o APT::Get::allow-change-held-packages=true'.

autoremove-config-purge = --remove-config უნდა გამოიყენოთ --purge ან autopurge პარამეტრთან ერთად.

autoremove-assume-yes = --assume-yes პარამეტრით ავტომატური წაშლა გამოტოვებულია. ჩასართავად გამოიყენეთ --autoremove.

download-no-space = { $path }-ში საკმარისი თავისუფალი ადგილი არაა: საჭიროა { $required }, ხელმისაწვდომია { $available }.

progress-data = მონაცემები

progress-packages = პაკეტები

progress-package = პაკეტი

progress-status = სტატუსი

progress-mirrors = სარკეები

progress-update-title = პაკეტების სიების განახლება

progress-download-title = პაკეტების გადმოწერა

progress-install-title = პაკეტების დაყენება

progress-score-title = სარკეების შემოწმება

progress-last-completed = ბოლო დასრულებული

progress-connections = კავშირები

progress-average = საშუალო

progress-active = აქტიური

progress-state = მდგომარეობა

progress-starting = იწყება

progress-downloading = მიმდინარეობს გადმოწერა

progress-idle = უმოქმედო

progress-more-mirrors = კიდევ +{ $mirrors } სარკე · { $connections } აქტიური კავშირი

config-parse = { $path }-ის დამუშავება ვერ მოხერხდა. Nala არსებული კონფიგურაციის ფაილს არ უგულებელყოფს. თუ ეს პაკეტის განახლების შემდეგ მოხდა, შეადარეთ ის { $path }.dpkg-dist ფაილს
