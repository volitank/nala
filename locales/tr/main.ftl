### Messages migrated from legacy/python/po/tr.po.
### Missing messages fall back to English.

log-error = Hata:

log-notice = Dikkat:

log-warning = Uyarı:

prompt-continue = Devam etmek istiyor musunuz?

prompt-choice = E/e H/h

root-required = Nala'nın { $command } için root'a ihtiyacı var

op-auto-purge = Otomatik Temizle

op-auto-remove = Otomatik Kaldır

op-remove = Sil

op-purge = Temizle

op-install = Yükle

op-reinstall = Y.Yükle

op-upgrade = Güncelle

op-downgrade = Düşür

op-configure = Ayarla

summary-title = Özet

summary-total-download = Toplam indirme boyutu:

summary-disk-required = Gerekecek disk alanı.:

summary-disk-free = Boşalacak disk alanı.:

summary-total-download-value = Toplam indirme boyutu: { $size }

summary-disk-required-value = Gerekecek disk alanı.: { $size }

summary-disk-free-value = Boşalacak disk alanı.: { $size }

summary-nothing = Nala'nın yapacak bir şeyi yok.

summary-essential = Aşağıdaki paketler gereklidir!

summary-remove-essential = Temel paketleri kaldırmayı denediniz

summary-use-switch = İstediğinizden eminseniz lütfen { $switch } kullanın.

summary-reboot = Yeniden başlatma gerekli.

summary-reboot-packages = Aşağıdaki paketler yeniden başlatma gerektiriyor.

table-package = Paket:

table-version = Sürüm:

table-size = Boyut:

table-old-version = Eski Sürüm:

table-new-version = Yeni Sürüm:

pkg-no-candidate = { $package } yükleme adayına sahip değil.

pkg-not-installed = { $package } kurulu değil

pkg-already-latest = { $package } { $version } zaten en son sürümde

pkg-version-missing = { $version } sürümü, { $package } paketi için bulunamadı

pkg-cache-missing = { $package } ancak önbellekte değil

pkg-recorded-version-missing = { $package } için sürüm bulunamıyor

pkg-version-cache-missing = { $version } sürümü, { $package } paketi için bulunamadı

pkg-virtual-providers = { $package } şunun tarafından sağlanan sanal bir pakettir:

pkg-virtual-one = Yalnızca birini seçmelisiniz.

download-complete = İndirme Tamamlandı:

download-auth-warning = Aşağıdaki paketler doğrulanamadı!

download-auth-allowed = Yapılandırma, kimliği doğrulanmamış paketlerin kurulumuna izin verecek şekilde ayarlandı.

download-exit = İsteğiniz üzerine çıkılıyor.

progress-working = [Çalışıyor]

progress-remaining = Kalan Zaman

progress-fetched = { $time }'de { $size } ({ $rate }/s)

tui-pkgs = Paket

file-read-defaults = Yapılandırma dosyası okunamıyor: { $path }. Varsayılan kullanılıyor

dpkg-removing = Kaldırılıyor:

dpkg-unpacking = Açılıyor....:

dpkg-setting-up = Ayarlanıyor.:

dpkg-processing = İşleniyor...:

list-virtual = Sanal Paket

show-local-install = yerel kurulum

show-more-records = { $count } ek kayıt var. Bunları görmek için lütfen { $switch } anahtarını kullanın.

show-no-description = Açıklama Yok

show-package = Paket

show-version = Sürüm

show-architecture = Mimari

show-priority = Öncelik

show-essential = Temel

show-section = Bölüm

show-source = Kaynak

show-installed-size = Kurulu Boyut

show-maintainer = Bakıcı

show-original-maintainer = Esas-Bakıcı

show-homepage = Anasayfa

show-origin = Köken

show-provides = Sağladıkları

show-description = Açıklama

show-apt-sources = APT-Kaynakları

show-depends = Bağımlılıklar

show-pre-depends = Ön-Bağımlı

show-suggests = Öneriler

show-recommends = Tavsiyeler

show-conflicts = Çakışmalar

show-replaces = Değiştirir

show-breaks = Bozdukları

show-enhances = Geliştirmeler

show-attr-installed = Kurulu

show-attr-upgradable-to = kurulu ve { $version } sürümüne yükseltilebilir

show-attr-upgradable-from = { $version }'dan yükseltilebilir

policy-installed = Kurulu:

history-empty = Geçmiş yok.

history-not-found = İşlem { $id } mevcut değil.

fetch-no-mirrors = Nala herhangi bir yansı bulamadı.

fetch-release-detect = Sürüm tespit edilirken bir sorun oluştu.

fetch-sources-written = Kaynaklar { $file } dosyasına yazıldı

fetch-score-help = Sonuç, Sürüm dosyasını indirmek için geçen milisaniye sayısıdır.

update-upgradable = { $count } paket yükseltilebilir. Bunları görmek için '{ $command }' komutunu çalıştırın.

update-no-change = Değişmedi..

update-updated = Güncellendi

update-ignored = Yoksayıldı.

update-downloading = İndiriliyor

update-processing = İşleniyor...

install-downloaded = İndirildi..: { $path }

## Additional messages

unknown = Bilinmiyor

none = Yok

error = Hata

mirrors = Yansılar:

log-info = Bilgi:

log-verbose = Ayrıntılı:

log-debug = Hata ayıklama:

not-implemented = Henüz uygulanmadı.

subcommand-missing = Alt komut belirtilmedi

prompt-invalid = '{ $response }' geçerli bir yanıt değil

prompt-refused = Kullanıcı onay vermedi

release-info-title = Depo Bilgileri Değişti

release-info-explanation =
    Bir deponun tanımlayıcı bilgileri Release dosyasında değişti.
    Devam etmeden önce değişiklikleri gözden geçirin.

release-info-repository = Depo

release-info-distribution = Dağıtım

release-info-changes = Değişiklikler

release-info-confirm = Bu değişiklikler kabul edilip güncellemeye devam edilsin mi?

release-info-help = ({ $yes }) kabul et | ({ $no }/Enter/Esc) reddet

release-info-accepted = { $repository } deposunun bilgi değişiklikleri kabul edildi.

release-info-rejected = { $repository } deposunun bilgi değişiklikleri reddedildi.

release-info-noninteractive = Release bilgisi değişiklikleri açıkça kabul edilmelidir. Kabul etmek için '-o Acquire::AllowReleaseInfoChange=true' ile yeniden çalıştırın.

op-held = Tutulan

reason-excluded = Hariç tutulan

reason-manual-hold = Elle tutulan

reason-phased = Aşamalı

reason-kept-back = Geri tutulan

reason-phased-percent = Aşamalı %{ $percentage }

summary-op-count = { $operation } { $count }

summary-op-count-colon = { $operation }: { $count }

summary-protected = Aşağıdaki paketler temel veya korumalıdır!

summary-remove-protected = Temel veya korumalı paketleri kaldırmayı denediniz

summary-downgrade-assume-yes = --assume-yes ile paketlerin sürümleri düşürülecek. Eminseniz '-o APT::Get::allow-downgrades=true' kullanın.

summary-held-assume-yes = --assume-yes ile tutulan paketler değiştirilecek. Eminseniz '-o APT::Get::allow-change-held-packages=true' kullanın.

autoremove-config-purge = --remove-config, --purge veya autopurge ile kullanılmalıdır.

autoremove-assume-yes = --assume-yes ile otomatik kaldırma atlanıyor. Etkinleştirmek için --autoremove kullanın.

table-reason = Neden:

pkg-not-found = '{ $package }' bulunamadı

pkg-not-found-cache = Bazı paketler önbellekte bulunamadı

pkg-mark-install = '{ $package }' kurulum için işaretlenemedi

pkg-reinstall-missing = { $package } kurulu değil, bu nedenle yeniden kurulamaz

pkg-invalid-name = Geçersiz paket adı: '{ $package }'

pkg-op-conflict = '{ $package }' için çakışan işlemler: { $operation-a } ve { $operation-b }

pkg-version-conflict = '{ $package }' için çakışan sabitlenmiş sürümler: { $version-a } ve { $version-b }

pkg-invalid-pin = Geçersiz sürüm sabitlemesi: '{ $pin }'

pkg-pin-exact = Sürüm sabitlemesi tam paket adı gerektirir: '{ $pin }'

pkg-exclude-missing = '{ $pattern }' hariç tutma deseni hiçbir paketle eşleşmedi

pkg-virtual-no-providers = { $package } yalnızca sanaldır ve hiçbir sağlayıcısı yoktur

pkg-virtual-select = Sanal { $package } paketi yerine { $provider } seçiliyor

pkg-select = '{ $requested }' yerine '{ $selected }' paketi seçiliyor

pkg-fix-broken = Bozuk paketler düzeltilemedi

pkg-required = Bir paket belirtmelisiniz

download-failed = İndirmeler Başarısız

download-some-missing = Bazı paketler bulunamadı.

download-auth-required =
    Bazı paketlerin kimliği doğrulanamadı.
    Eminseniz { $switch } kullanın

download-items = Öğeler

download-source-missing = '{ $package }' paketinin '{ $version }' sürümünü indirmek için kaynak bulunamadı

download-written = { $package }, { $path } konumuna yazıldı

download-hash-missing = '{ $uri }' için özet bulunamadı

download-protocol-missing = Protokol tanımlanmadı

download-uri-missing = URI tanımlanmadı

download-malformed = '{ $uri }' hatalı biçimlendirilmiş!

download-request-failed = '{ $uri }' için indirme isteği başarısız oldu

download-stream-failed = '{ $uri }' kaynağından veri akışı alınamadı

download-get = İndirme isteği gönderilemedi

download-content-length = { $headers } içinde content-length yok

download-content-str = content-length metne dönüştürülemedi: { $headers }

download-content-parse = content-length ayrıştırılamadı: { $headers }

download-checksum = { $file } için sağlama toplamı eşleşmedi

download-no-uris = { $file } hiçbir URI'den indirilemedi

download-no-space = { $path } konumunda yeterli boş alan yok: { $required } gerekli, { $available } kullanılabilir.

download-filename = { $path } geçerli bir dosya adına sahip değil!

download-error = Hata: { $error }

download-select-domain = { $file } için { $domain } seçiliyor

download-start = Başlatılıyor: { $uri }, Yeniden denemeler: { $retries }

download-finished = Tamamlandı: { $uri }

progress-nothing = Alınacak bir şey yok

progress-label = İlerleme

progress-total = Toplam

progress-data = Veri

progress-packages = Paketler

progress-package = Paket

progress-status = Durum

progress-mirrors = Yansılar

progress-speed = Hız

progress-elapsed = Geçen süre

progress-finished = Tamamlandı:

progress-update-title = Paket Listeleri Güncelleniyor

progress-download-title = Paketler İndiriliyor

progress-install-title = Paketler Kuruluyor

progress-score-title = Yansılar Test Ediliyor

progress-last-completed = Son tamamlanan

progress-connections = Bağlantılar

progress-average = Ortalama

progress-active = Etkin

progress-state = Durum

progress-starting = başlatılıyor

progress-downloading = indiriliyor

progress-idle = boşta

progress-more-mirrors = +{ $mirrors } yansı daha · { $connections } etkin bağlantı

tui-upgrade-title = Nala Yükseltme

tui-history-title = Nala Geçmişi

tui-help-move = (↑) yukarı | (↓) aşağı

tui-help-tabs = (→) sonraki sekme | (←) önceki sekme

tui-help-show = (Enter) değişiklik günlüğünü göster | (s) sürüm bilgisini göster

tui-help-confirm = (q) çık | (y) yükseltmeyi başlat

tui-help-quit = (q) çık

tui-changelog-missing = Değişiklik günlüğü URI'si bulunamadı

file-remove = { $path } kaldırılamadı

file-read = '{ $path }' okunamadı

file-parse-defaults = { $path } ayrıştırılamadı, varsayılanlar kullanılıyor

config-parse = { $path } ayrıştırılamadı. Nala mevcut bir yapılandırma dosyasını yok saymaz. Bu durum bir paket güncellemesinden sonra oluştuysa dosyayı { $path }.dpkg-dist ile karşılaştırın

file-create = '{ $path }' oluşturulamadı

file-write = '{ $path }' yazılamadı

file-replace = '{ $path }' değiştirilemedi

file-deserialize = '{ $path }' seriden çıkarılamadı

fs-failed = { $path } üzerinde { $operation } işlemi başarısız oldu

fs-failed-target = { $operation } işlemi başarısız oldu: { $path } => { $target }

config-option = '{ $option }' seçeneği desteklenmiyor

color-expected = renk adı, 0-255 arası indeks veya #RRGGBB biçiminde onaltılık metin

color-index = renk indeksi 0 ile 255 arasında olmalıdır

color-unknown = bilinmeyen renk '{ $color }'

color-rgb-components = Rgb üç bileşen gerektirir

color-rgb-expected = Rgb veya Indexed renk bekleniyor

color-modifier-expected = "BOLD | ITALIC" gibi bir değiştirici metni veya bir dizi

color-modifier-unknown = bilinmeyen değiştirici '{ $modifier }'

archive-unsupported = Arşiv türü desteklenmiyor

deb-control-missing = { $path } içinde control dosyası bulunamadı

hash-unsupported = Özet türü { $type } desteklenmiyor

hash-unavailable =
    { $package } { $version } bütünlük açısından denetlenemiyor.
    Bu paket için kullanılabilir özet yok.

dpkg-child-failed = Dpkg alt süreci başarısız oldu: { $error }

dpkg-status-utf8 = Dpkg durum dosya tanımlayıcısı geçersiz UTF-8 döndürdü

dpkg-exit = Dpkg şu kodla çıktı: '{ $code }'

dpkg-poll = Alt süreç yoklanamadı

dpkg-read-status = Durum dosya tanımlayıcısı okunamadı

dpkg-read-pty = Sözde terminalden okunamadı

dpkg-write-pty = Standart girdi sözde terminale gönderilemedi

show-no-summary = Özet Yok

show-size = Boyut

show-sha256 = SHA256

show-archive = Arşiv

show-codename = Kod adı

show-component = Bileşen

show-attributes = Öznitelikler

show-obsoletes = Eski sürümleri geçersiz kılar

show-attr-local = Yerel

show-attr-auto-removable = Otomatik kaldırılabilir

show-attr-automatic = Otomatik

policy-none = yok

policy-candidate = Aday:

policy-version-table = Sürüm tablosu:

policy-no-versions = Sürüm yok.

policy-origin = köken

policy-package-files = Paket dosyaları:

policy-pinned = Sabitlenmiş paketler:

policy-release = sürüm

history-cleared =
    { $count ->
        [one] { $count } geçmiş kaydı temizlendi.
       *[other] { $count } geçmiş kaydı temizlendi.
    }

history-not-replayable = '{ $id }' geçmiş kaydı uygulanmış olarak kaydedilmediğinden yinelenemez

history-no-changes = '{ $id }' geçmiş kaydında yinelenecek paket değişikliği yok

history-cleared-entry = { $id } geçmiş kaydı temizlendi.

history-id = Kimlik

history-command = Komut

history-date-time = Tarih ve Saat

history-requested-by = İsteyen

history-altered = Değiştirilen

history-status = Durum

history-status-applied = Uygulandı

history-started = Başladı

history-finished = Bitti

history-targets = İstenen Hedefler

history-undo-version-missing = Önceki sürüm kaydedilmediğinden '{ $package }' için geri alma desteklenmiyor

history-undo-config-only = Yalnızca yapılandırma dosyalarının kaldığı durumun geri yüklenmesi uygulanmadığından '{ $package }' için geri alma desteklenmiyor

history-undo-installed-version-missing = Önceki kurulu sürüm kaydedilmediğinden '{ $package }' için geri alma desteklenmiyor

history-undo-reinstall = Yeniden kurulumun ters işlemi kaydedilmediğinden '{ $package }' için geri alma desteklenmiyor

history-undo-held = Tutulan '{ $package }' paketi geri alınamaz

history-redo-version-missing = Sonuç sürümü kaydedilmediğinden '{ $package }' için yineleme desteklenmiyor

history-redo-reinstall-missing = Yeniden kurulan sürüm kaydedilmediğinden '{ $package }' için yineleme desteklenmiyor

history-redo-held = Tutulan '{ $package }' paketi yinelenemez

history-clear-target = Geçmişi temizlemek için bir kayıt seçicisi veya --all gerekir

history-config-undo = Yapılandırılmış '{ $package }' paketi geri alınamaz

history-config-redo = Yapılandırılmış '{ $package }' paketi yinelenemez

history-serialize = Geçmiş kaydı serileştirilemedi

fetch-none-selected = Hiçbir yansı seçilmedi.

fetch-unsupported = { $distro } desteklenmiyor.

fetch-title = Nala Yansı Seçimi

fetch-score = Puan:

fetch-help = Hareket için ↓↑, seçmek/seçimi kaldırmak için Boşluk, başa/sona gitmek için Home/End, çıkmak için q/Enter kullanın.

update-item = { $state }: { $description }

upgrade-exclude-unsafe =
    Seçilen paketler yükseltmeden güvenli biçimde hariç tutulamaz.
    { $error }

upgrade-protect = { $package }, { $reason } işleminden korunuyor

upgrade-reason-upgrade = yükseltme

upgrade-reason-auto-remove = otomatik kaldırma

upgrade-config-missing = Yapılandırma ağacı yok!
