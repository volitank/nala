### Messages migrated from legacy/python/po/bn.po.
### Missing messages fall back to English.

log-error = ত্রুটি:

log-notice = বিজ্ঞপ্তি:

log-warning = সাবধান:

prompt-continue = আপনি কি চালিয়ে যেতে চান?

summary-nothing = Nalaর কিছুই করার নেই।

summary-reboot = একটি রিবুট প্রয়োজন৷।

download-complete = ডাউনলোড শেষ:

download-exit = আপনার অনুরোধে প্রস্থান করা হচ্ছে

list-virtual = ভার্চুয়াল প্যাকেজ

show-depends = নির্ভর করে

show-conflicts = দ্বন্দ্ব

show-breaks = ভাঙে

update-no-change = পরিবর্তন নেই

update-updated = আপডেটেড

update-ignored = উপেক্ষিত

update-downloading = ডাউনলোডিং

## Additional messages

unknown = অজানা

none = নেই

error = ত্রুটি

mirrors = মিরর:

log-info = তথ্য:

log-verbose = বিস্তারিত:

log-debug = ডিবাগ:

not-implemented = এখনও বাস্তবায়িত হয়নি।

subcommand-missing = উপকমান্ড দেওয়া হয়নি

prompt-invalid = '{ $response }' একটি বৈধ উত্তর নয়

prompt-refused = ব্যবহারকারী নিশ্চিত করতে অস্বীকৃতি জানিয়েছেন

prompt-choice = Y/y N/n

release-info-title = রিপোজিটরির তথ্য পরিবর্তিত হয়েছে

release-info-explanation =
    একটি রিপোজিটরির শনাক্তকারী তথ্য তার Release ফাইলে পরিবর্তিত হয়েছে।
    চালিয়ে যাওয়ার আগে পরিবর্তনগুলো পর্যালোচনা করুন।

release-info-repository = রিপোজিটরি

release-info-distribution = ডিস্ট্রিবিউশন

release-info-changes = পরিবর্তন

release-info-confirm = এই পরিবর্তনগুলো গ্রহণ করে আপডেট চালিয়ে যাবেন?

release-info-help = ({ $yes }) গ্রহণ | ({ $no }/Enter/Esc) প্রত্যাখ্যান

release-info-accepted = { $repository } রিপোজিটরির তথ্যের পরিবর্তনগুলো গ্রহণ করা হয়েছে।

release-info-rejected = { $repository } রিপোজিটরির তথ্যের পরিবর্তনগুলো প্রত্যাখ্যান করা হয়েছে।

release-info-noninteractive = Release তথ্যের পরিবর্তনগুলো স্পষ্টভাবে গ্রহণ করতে হবে। গ্রহণ করার জন্য '-o Acquire::AllowReleaseInfoChange=true' দিয়ে আবার চালান।

root-required = { $command } চালাতে Nala-এর root অনুমতি প্রয়োজন

op-remove = অপসারণ

op-auto-remove = স্বয়ংক্রিয় অপসারণ

op-purge = কনফিগারেশনসহ অপসারণ

op-auto-purge = স্বয়ংক্রিয়ভাবে কনফিগারেশনসহ অপসারণ

op-install = ইনস্টল

op-reinstall = পুনরায় ইনস্টল

op-upgrade = আপগ্রেড

op-downgrade = সংস্করণ নামানো

op-configure = কনফিগার

op-held = আটকে রাখা

reason-excluded = বাদ দেওয়া

reason-manual-hold = ম্যানুয়ালি আটকে রাখা

reason-phased = পর্যায়ক্রমিক

reason-kept-back = আপডেট স্থগিত

reason-phased-percent = পর্যায়ক্রমিক { $percentage }%

summary-title = সারসংক্ষেপ

summary-total-download = মোট ডাউনলোডের আকার:

summary-disk-required = প্রয়োজনীয় ডিস্কের স্থান:

summary-disk-free = খালি হবে এমন ডিস্কের স্থান:

summary-total-download-value = মোট ডাউনলোডের আকার: { $size }

summary-disk-required-value = প্রয়োজনীয় ডিস্কের স্থান: { $size }

summary-disk-free-value = খালি হবে এমন ডিস্কের স্থান: { $size }

summary-op-count = { $operation } { $count }

summary-op-count-colon = { $operation }: { $count }

summary-essential = নিচের প্যাকেজগুলো অপরিহার্য!

summary-remove-essential = আপনি অপরিহার্য প্যাকেজ অপসারণের চেষ্টা করেছেন

summary-protected = নিচের প্যাকেজগুলো অপরিহার্য বা সুরক্ষিত!

summary-remove-protected = আপনি অপরিহার্য বা সুরক্ষিত প্যাকেজ অপসারণের চেষ্টা করেছেন

summary-downgrade-assume-yes = --assume-yes ব্যবহার করলে প্যাকেজগুলোর সংস্করণ নামানো হবে। নিশ্চিত হলে '-o APT::Get::allow-downgrades=true' ব্যবহার করুন।

summary-held-assume-yes = --assume-yes ব্যবহার করলে আটকে রাখা প্যাকেজগুলো পরিবর্তিত হবে। নিশ্চিত হলে '-o APT::Get::allow-change-held-packages=true' ব্যবহার করুন।

summary-use-switch = নিশ্চিত হলে '{ $switch }' ব্যবহার করুন।

autoremove-config-purge = --remove-config অবশ্যই --purge বা autopurge-এর সঙ্গে ব্যবহার করতে হবে।

autoremove-assume-yes = --assume-yes ব্যবহার করলে স্বয়ংক্রিয় অপসারণ এড়িয়ে যাওয়া হচ্ছে। সক্রিয় করতে --autoremove ব্যবহার করুন।

summary-reboot-packages = নিচের প্যাকেজগুলোর জন্য পুনরায় চালু করা প্রয়োজন:

table-package = প্যাকেজ:

table-version = সংস্করণ:

table-old-version = পুরোনো সংস্করণ:

table-new-version = নতুন সংস্করণ:

table-reason = কারণ:

table-size = আকার:

pkg-not-found = '{ $package }' পাওয়া যায়নি

pkg-not-found-cache = কিছু প্যাকেজ ক্যাশে পাওয়া যায়নি

pkg-no-candidate = { $package }-এর ইনস্টল করার উপযোগী কোনো সংস্করণ নেই

pkg-mark-install = '{ $package }' ইনস্টলের জন্য চিহ্নিত করা যায়নি

pkg-not-installed = { $package } ইনস্টল করা নেই

pkg-reinstall-missing = { $package } ইনস্টল করা নেই, তাই পুনরায় ইনস্টল করা যায় না

pkg-invalid-name = অবৈধ প্যাকেজের নাম: '{ $package }'

pkg-op-conflict = '{ $package }'-এর জন্য পরস্পরবিরোধী কাজ: { $operation-a } ও { $operation-b }

pkg-version-conflict = '{ $package }'-এর জন্য পরস্পরবিরোধী নির্দিষ্ট সংস্করণ: { $version-a } ও { $version-b }

pkg-already-latest = { $package }{ $version } ইতিমধ্যে ইনস্টল করা আছে এবং এটিই সর্বশেষ সংস্করণ

pkg-version-missing = '{ $package }'-এর '{ $version }' সংস্করণ পাওয়া যায়নি

pkg-cache-missing = '{ $package }' প্যাকেজ ক্যাশে পাওয়া যায়নি

pkg-recorded-version-missing = '{ $package }'-এর কোনো সংস্করণ নথিভুক্ত নেই

pkg-version-cache-missing = '{ $package }'-এর '{ $version }' সংস্করণ পাওয়া যায়নি

pkg-invalid-pin = অবৈধ সংস্করণ নির্ধারণ: '{ $pin }'

pkg-pin-exact = সংস্করণ নির্ধারণের জন্য সঠিক প্যাকেজের নাম প্রয়োজন: '{ $pin }'

pkg-exclude-missing = '{ $pattern }' বাদ দেওয়ার প্যাটার্নের সঙ্গে কোনো প্যাকেজ মেলেনি

pkg-virtual-no-providers = { $package } সম্পূর্ণ ভার্চুয়াল এবং কোনো প্যাকেজ এটি সরবরাহ করে না

pkg-virtual-providers = { $package } একটি ভার্চুয়াল প্যাকেজ, যা সরবরাহ করে:

pkg-virtual-select = ভার্চুয়াল প্যাকেজ { $package }-এর পরিবর্তে { $provider } নির্বাচন করা হচ্ছে

pkg-virtual-one = শুধু একটি নির্বাচন করুন।

pkg-select = '{ $requested }'-এর পরিবর্তে '{ $selected }' প্যাকেজ নির্বাচন করা হচ্ছে

pkg-fix-broken = ত্রুটিপূর্ণ প্যাকেজগুলো ঠিক করা যায়নি

pkg-required = আপনাকে একটি প্যাকেজ উল্লেখ করতে হবে

download-failed = ডাউনলোড ব্যর্থ

download-some-missing = কিছু প্যাকেজ পাওয়া যায়নি।

download-auth-warning = নিচের প্যাকেজগুলোর সত্যতা যাচাই করা যাচ্ছে না!

download-auth-allowed = কনফিগারেশনে সত্যতা যাচাই না হওয়া প্যাকেজ ইনস্টলের অনুমতি দেওয়া আছে।

download-auth-required =
    কিছু প্যাকেজের সত্যতা যাচাই করা যায়নি।
    নিশ্চিত হলে { $switch } ব্যবহার করুন

download-items = আইটেম

download-source-missing = '{ $package }'-এর '{ $version }' সংস্করণ ডাউনলোড করার উৎস পাওয়া যায়নি

download-written = { $package } লেখা হয়েছে { $path }-এ

download-hash-missing = '{ $uri }'-এর কোনো হ্যাশ পাওয়া যায়নি

download-protocol-missing = কোনো প্রোটোকল নির্ধারিত হয়নি

download-uri-missing = কোনো URI নির্ধারিত হয়নি

download-malformed = '{ $uri }'-এর গঠন সঠিক নয়!

download-request-failed = '{ $uri }'-এর ডাউনলোড অনুরোধ ব্যর্থ হয়েছে

download-stream-failed = '{ $uri }' থেকে ডেটা স্ট্রিম করা যায়নি

download-get = ডাউনলোড অনুরোধ পাঠানো যায়নি

download-content-length = { $headers }-এ content-length নেই

download-content-str = content-length স্ট্রিংয়ে রূপান্তর করা যায়নি: { $headers }

download-content-parse = content-length বিশ্লেষণ করা যায়নি: { $headers }

download-checksum = { $file }-এর চেকসাম মেলেনি

download-no-uris = কোনো URI থেকেই { $file } ডাউনলোড করা যায়নি

download-no-space = { $path }-এ যথেষ্ট খালি স্থান নেই: প্রয়োজন { $required }, উপলব্ধ { $available }।

download-filename = { $path }-এর কোনো বৈধ ফাইলের নাম নেই!

download-error = ত্রুটি: { $error }

download-select-domain = { $file }-এর জন্য { $domain } নির্বাচন করা হচ্ছে

download-start = শুরু হচ্ছে: { $uri }, পুনরায় চেষ্টা: { $retries }

download-finished = সম্পন্ন: { $uri }

progress-working = কাজ চলছে...

progress-nothing = আনার কিছু নেই

progress-label = অগ্রগতি

progress-total = মোট

progress-data = ডেটা

progress-packages = প্যাকেজসমূহ

progress-package = প্যাকেজ

progress-status = অবস্থা

progress-mirrors = মিরর

progress-speed = গতি

progress-elapsed = অতিবাহিত

progress-remaining = বাকি

progress-finished = সম্পন্ন:

progress-update-title = প্যাকেজের তালিকা আপডেট হচ্ছে

progress-download-title = প্যাকেজ ডাউনলোড হচ্ছে

progress-install-title = প্যাকেজ ইনস্টল হচ্ছে

progress-score-title = মিরর পরীক্ষা হচ্ছে

progress-last-completed = সর্বশেষ সম্পন্ন

progress-connections = সংযোগ

progress-average = গড়

progress-active = সক্রিয়

progress-state = অবস্থা

progress-starting = শুরু হচ্ছে

progress-downloading = ডাউনলোড হচ্ছে

progress-idle = নিষ্ক্রিয়

progress-more-mirrors = আরও +{ $mirrors } মিরর · { $connections } সক্রিয় সংযোগ

progress-fetched = { $time } সময়ে { $size } আনা হয়েছে ({ $rate }/s)

tui-upgrade-title = Nala আপগ্রেড

tui-history-title = Nala ইতিহাস

tui-pkgs = প্যাকেজ

tui-help-move = (↑) উপরে | (↓) নিচে

tui-help-tabs = (→) পরের ট্যাব | (←) আগের ট্যাব

tui-help-show = (Enter) পরিবর্তনের তালিকা দেখান | (s) সংস্করণের তথ্য দেখান

tui-help-confirm = (q) প্রস্থান | (y) আপগ্রেড শুরু

tui-help-quit = (q) প্রস্থান

tui-changelog-missing = পরিবর্তনের তালিকার URI পাওয়া যায়নি

file-remove = { $path } অপসারণ করা যায়নি

file-read = '{ $path }' পড়া যায়নি

file-read-defaults = { $path } পড়া যায়নি, ডিফল্ট মান ব্যবহার করা হচ্ছে

file-parse-defaults = { $path } বিশ্লেষণ করা যায়নি, ডিফল্ট মান ব্যবহার করা হচ্ছে

config-parse = { $path } বিশ্লেষণ করা যায়নি। Nala বিদ্যমান কনফিগারেশন ফাইল উপেক্ষা করবে না। প্যাকেজ আপগ্রেডের পরে এটি হয়ে থাকলে, { $path }.dpkg-dist-এর সঙ্গে তুলনা করুন

file-create = '{ $path }' তৈরি করা যায়নি

file-write = '{ $path }' লেখা যায়নি

file-replace = '{ $path }' প্রতিস্থাপন করা যায়নি

file-deserialize = '{ $path }' ডিসিরিয়ালাইজ করা যায়নি

fs-failed = { $path }-এর জন্য { $operation } কাজটি ব্যর্থ হয়েছে

fs-failed-target = { $operation } কাজটি ব্যর্থ হয়েছে: { $path } => { $target }

config-option = '{ $option }' বিকল্পটি সমর্থিত নয়

color-expected = রঙের নাম, 0-255 সূচক, অথবা #RRGGBB হেক্স স্ট্রিং

color-index = রঙের সূচক 0 থেকে 255-এর মধ্যে হতে হবে

color-unknown = অজানা রঙ '{ $color }'

color-rgb-components = Rgb-এর জন্য তিনটি উপাদান প্রয়োজন

color-rgb-expected = Rgb বা Indexed রঙ প্রত্যাশিত

color-modifier-expected = "BOLD | ITALIC"-এর মতো মডিফায়ার স্ট্রিং অথবা অ্যারে

color-modifier-unknown = অজানা মডিফায়ার '{ $modifier }'

archive-unsupported = আর্কাইভের ধরন সমর্থিত নয়

deb-control-missing = { $path }-এ control ফাইল পাওয়া যায়নি

hash-unsupported = { $type } হ্যাশের ধরন সমর্থিত নয়

hash-unavailable =
    { $package } { $version }-এর অখণ্ডতা যাচাই করা যায় না।
    এই প্যাকেজের জন্য কোনো হ্যাশ উপলব্ধ নেই।

dpkg-child-failed = Dpkg-এর চাইল্ড প্রক্রিয়া ব্যর্থ হয়েছে: { $error }

dpkg-status-utf8 = Dpkg-এর স্ট্যাটাস ফাইল ডেসক্রিপ্টর অবৈধ UTF-8 ফেরত দিয়েছে

dpkg-exit = Dpkg প্রস্থান করেছে এই কোড নিয়ে: '{ $code }'

dpkg-poll = চাইল্ড প্রক্রিয়ার অবস্থা পরীক্ষা করা যায়নি

dpkg-read-status = স্ট্যাটাস ফাইল ডেসক্রিপ্টর পড়া যায়নি

dpkg-read-pty = ছদ্মটার্মিনাল থেকে পড়া যায়নি

dpkg-write-pty = ছদ্মটার্মিনালে স্ট্যান্ডার্ড ইনপুট পাঠানো যায়নি

dpkg-removing = অপসারণ হচ্ছে:

dpkg-unpacking = আনপ্যাক হচ্ছে:

dpkg-setting-up = সেটআপ হচ্ছে:

dpkg-processing = প্রক্রিয়াকরণ হচ্ছে:

show-local-install = স্থানীয় ইনস্টল

show-more-records = আরও { $count }টি রেকর্ড রয়েছে। দেখতে { $switch } বিকল্পটি ব্যবহার করুন।

show-no-description = কোনো বিবরণ নেই

show-no-summary = কোনো সারসংক্ষেপ নেই

show-package = প্যাকেজ

show-version = সংস্করণ

show-architecture = আর্কিটেকচার

show-priority = অগ্রাধিকার

show-essential = অপরিহার্য

show-section = বিভাগ

show-source = উৎস

show-installed-size = ইনস্টল করা আকার

show-size = আকার

show-maintainer = রক্ষণাবেক্ষণকারী

show-original-maintainer = মূল রক্ষণাবেক্ষণকারী

show-homepage = হোমপেজ

show-sha256 = SHA256

show-archive = আর্কাইভ

show-origin = উৎস

show-codename = সাংকেতিক নাম

show-component = উপাদান

show-provides = সরবরাহ করে

show-description = বিবরণ

show-attributes = বৈশিষ্ট্য

show-apt-sources = APT উৎস

show-pre-depends = পূর্বনির্ভরতা

show-suggests = প্রস্তাব করে

show-recommends = সুপারিশ করে

show-replaces = প্রতিস্থাপন করে

show-obsoletes = অপ্রচলিত করে

show-enhances = উন্নত করে

show-attr-installed = ইনস্টল করা

show-attr-local = স্থানীয়

show-attr-auto-removable = স্বয়ংক্রিয়ভাবে অপসারণযোগ্য

show-attr-automatic = স্বয়ংক্রিয়

show-attr-upgradable-to = আপগ্রেড করা যায়: { $version }

show-attr-upgradable-from = যে সংস্করণ থেকে আপগ্রেড করা যায়: { $version }

policy-none = নেই

policy-installed = ইনস্টল করা:

policy-candidate = প্রার্থী:

policy-version-table = সংস্করণের সারণি:

policy-no-versions = কোনো সংস্করণ নেই।

policy-origin = উৎস

policy-package-files = প্যাকেজের ফাইল:

policy-pinned = নির্দিষ্ট সংস্করণে রাখা প্যাকেজ:

policy-release = রিলিজ

history-cleared = { $count }টি ইতিহাসের এন্ট্রি মুছে ফেলা হয়েছে।

history-empty = ইতিহাসের কোনো এন্ট্রি পাওয়া যায়নি।

history-not-found = '{ $id }' আইডির ইতিহাসের এন্ট্রি নেই

history-not-replayable = '{ $id }' ইতিহাসের এন্ট্রি প্রয়োগ করা হয়েছে বলে নথিভুক্ত নয়, তাই আবার চালানো যায় না

history-no-changes = '{ $id }' ইতিহাসের এন্ট্রিতে আবার চালানোর মতো প্যাকেজের পরিবর্তন নেই

history-cleared-entry = ইতিহাসের { $id } এন্ট্রি মুছে ফেলা হয়েছে।

history-id = আইডি

history-command = কমান্ড

history-date-time = তারিখ ও সময়

history-requested-by = অনুরোধকারী

history-altered = পরিবর্তিত

history-status = অবস্থা

history-status-applied = প্রয়োগ করা হয়েছে

history-started = শুরু

history-finished = শেষ

history-targets = অনুরোধ করা লক্ষ্যসমূহ

history-undo-version-missing = আগের সংস্করণ নথিভুক্ত না থাকায় '{ $package }'-এর কাজ পূর্বাবস্থায় ফেরানো সমর্থিত নয়

history-undo-config-only = শুধু কনফিগারেশন ফাইল থাকা অবস্থা পুনরুদ্ধার বাস্তবায়িত না হওয়ায় '{ $package }'-এর কাজ পূর্বাবস্থায় ফেরানো সমর্থিত নয়

history-undo-installed-version-missing = আগে ইনস্টল করা সংস্করণ নথিভুক্ত না থাকায় '{ $package }'-এর কাজ পূর্বাবস্থায় ফেরানো সমর্থিত নয়

history-undo-reinstall = পুনরায় ইনস্টলের বিপরীত কাজ নথিভুক্ত না থাকায় '{ $package }'-এর কাজ পূর্বাবস্থায় ফেরানো সমর্থিত নয়

history-undo-held = আটকে রাখা '{ $package }' প্যাকেজের কাজ পূর্বাবস্থায় ফেরানো যায় না

history-redo-version-missing = ফলাফলের সংস্করণ নথিভুক্ত না থাকায় '{ $package }'-এর কাজ আবার করা সমর্থিত নয়

history-redo-reinstall-missing = পুনরায় ইনস্টল করা সংস্করণ নথিভুক্ত না থাকায় '{ $package }'-এর কাজ আবার করা সমর্থিত নয়

history-redo-held = আটকে রাখা '{ $package }' প্যাকেজের কাজ আবার করা যায় না

history-clear-target = ইতিহাস মুছতে একটি এন্ট্রি নির্বাচন বা --all প্রয়োজন

history-config-undo = কনফিগার করা '{ $package }' প্যাকেজের কাজ পূর্বাবস্থায় ফেরানো যায় না

history-config-redo = কনফিগার করা '{ $package }' প্যাকেজের কাজ আবার করা যায় না

history-serialize = ইতিহাসের এন্ট্রি সিরিয়ালাইজ করা যায়নি

fetch-no-mirrors = Nala কোনো মিরর খুঁজে পায়নি।

fetch-none-selected = কোনো মিরর নির্বাচন করা হয়নি।

fetch-release-detect = রিলিজ শনাক্ত করতে সমস্যা হয়েছে।

fetch-unsupported = { $distro } সমর্থিত নয়।

fetch-sources-written = উৎসগুলো লেখা হয়েছে { $file }-এ

fetch-title = Nala মিরর নির্বাচন

fetch-score = স্কোর:

fetch-score-help = Release ফাইল ডাউনলোড করতে যত মিলিসেকেন্ড লাগে, সেটিই স্কোর।

fetch-help = চলাচলের জন্য ↓↑, নির্বাচন/নির্বাচন বাতিলের জন্য Space, উপরে/নিচে যেতে Home/End এবং বের হতে q/Enter ব্যবহার করুন।

update-upgradable = { $count }টি প্যাকেজ আপগ্রেড করা যায়। দেখতে '{ $command }' চালান।

update-processing = প্রক্রিয়াকরণ হচ্ছে

update-item = { $state }: { $description }

upgrade-exclude-unsafe =
    নির্বাচিত প্যাকেজগুলো নিরাপদে আপগ্রেড থেকে বাদ দেওয়া যায় না।
    { $error }

upgrade-protect = { $package }-কে { $reason } থেকে সুরক্ষিত রাখা হচ্ছে

upgrade-reason-upgrade = আপগ্রেড

upgrade-reason-auto-remove = স্বয়ংক্রিয় অপসারণ

upgrade-config-missing = কনফিগারেশন ট্রি নেই!

install-downloaded = ডাউনলোড করা হয়েছে: { $path }
