# SP GROUP PDF Görüntüleyici

MuPDF render motoru ve GPU hızlandırmalı GPUI Kit arayüzüyle geliştirilen, çok iş parçacıklı Windows PDF görüntüleyicisi.

## Özellikler

- GPU üzerinde çizilen, sekmeli belge çalışma alanı
- Gerçek PDF içindekiler ağacı; sayfa önizlemeleri, belge içi arama ve sayfa metni panelleri
- Genişliğe/sayfaya sığdırma, ilk açılışta otomatik genişlik sığdırma, Ctrl+tekerlek ve pinch ile yakınlaştırma
- Zoom sırasında önbellek görüntüsünü anında ölçekleme ve 140 ms sonra keskin render; eski render sonuçları nesil denetimiyle yok sayılır
- Sanal listeyle yalnızca görünen sayfaların küçük resimlerini üretme
- Sürükle-bırak ile belge açma, son açılan dosyalar ve başlangıç argümanıyla PDF açma
- Kurumsal “SP Grafit” koyu/açık teması; tema tercihi otomatik hatırlanır
- Lucide ikonları, özel başlık çubuğu, odak halkaları ve erişilebilir düğme etiketleri
- 44 px birincil eylem alanları; klavye ile tam panel ve sayfa denetimi

## Klavye Kısayolları

| Kısayol | Eylem |
| --- | --- |
| Ctrl + O | Belge aç |
| Ctrl + W | Aktif sekmeyi kapat |
| Ctrl + F | Belgede ara |
| Ctrl + B | Kenar panelini aç/kapat |
| Alt + T / S / A / M | İçindekiler / Sayfalar / Ara / Metin paneli |
| Ctrl + Shift + D | Tema değiştir |
| F1 | Hakkında |
| ← / →, PgUp / PgDn | Önceki / sonraki sayfa |
| Home / End | İlk / son sayfa |
| Ctrl + + / Ctrl + - | Yakınlaştır / uzaklaştır |
| Ctrl + 0 | %100 |
| Ctrl + Tekerlek | Akıcı yakınlaştırma |
| Esc | Açık kaplamayı kapat |

## Hazır Uygulama

`release` klasöründeki `app-pdf_beta-vYYYYMMDDHHMM.exe` dosyası kurulum gerektirmez ve doğrudan çalıştırılır. Klasörde yalnızca en güncel sürüm bulundurulur; her derlemede önceki sürümler otomatik olarak kaldırılır. Dosya adındaki `beta-vYYYYMMDDHHMM` damgası derleme anını gösterir ve uygulama içindeki sürüm bilgisiyle aynıdır.

## Kaynaktan Derleme

Gereksinimler: Rust (MSVC hedefi), Visual Studio 2022 C++ Build Tools, CMake ve LLVM (`C:\Program Files\LLVM`).

```powershell
.\vendor\derle.ps1
```

Betik sürüm damgasını oluşturur, GPUI ve MuPDF bağımlılıklarını derler ve çıktıyı `release\app-pdf_beta-vYYYYMMDDHHMM.exe` adıyla kopyalar. İlk derleme GPUI ve MuPDF C çekirdeğini sıfırdan derlediğinden uzun sürer; sonraki derlemeler artımlıdır.

## Geliştirici Kurulumu (Yeni Bilgisayar)

```powershell
git clone https://github.com/spgroupenerji/app-pdf.git
cd app-pdf
powershell -ExecutionPolicy Bypass -File vendor\hazirla.ps1
```

Betik ön koşulları denetler (Rust, LLVM, MSVC, CMake), bağımlılıkları indirir ve derler. Cargo derleme önbelleği (`target/`) makineye ve yola özgüdür; depoya dahil edilmez ve kopyalanamaz.

## Mimari

- Arayüz GPUI Kit 0.8.1 üzerinde çalışır (`gpui-pre 0.3.10`, `gpui-component 0.8.1`, Apache-2.0).
- MuPDF yalnızca tek bir arka plan iş parçacığında kullanılır; `Document` `!Send` olduğu için bu kural zorunludur.
- İşçi ile arayüz arasında yalnızca `IslemTalebi` / `RenderSonuc` mesajları ve ham RGBA `SayfaGoruntusu` taşınır; MuPDF tipleri UI katmanına sızmaz.
- Sayfa görüntüleri GPU dokusu olarak `RenderImage` ile tutulur; zoom sırasında önbellek görüntüsü anında ölçeklenir.
- Her render talebi bir nesil numarası taşır; gecikmiş sonuçlar yeni sayfayı veya zoom seviyesini ezemez.
- İçindekiler paneli PDF outline ağacını gerçek sayfa hedefleriyle okur; küçük resimler sanal listede yalnız görünen aralık için istenir.
- Tercihler ve son dosyalar `%APPDATA%\pdf_goruntuleyici` altında saklanır.

## Güvenlik

- Uygulama yalnızca yerel PDF dosyalarını işler; arka planda ağ isteği yapmaz ve kullanıcı verisi toplamaz.
- Tercihler ve son açılan dosyalar listesi yalnızca yerel kullanıcı dizininde saklanır.
- Belge ayrıştırma ve render işlemleri arayüz iş parçacığından bağımsız yürütülür; sorunlu belgeler arayüzü kilitlemez.
- Dosya yolları yalnızca yerel dosya seçici, sürükle-bırak veya başlangıç argümanıyla alınır; uzantı `.pdf` olarak doğrulanır.

## Doğrulama

```powershell
cargo fmt --check
cargo clippy --all-targets -- -D warnings
cargo test
cargo build --release
```

Birim testleri zoom kırpma ve sığdırma hesaplarını kapsar. Arayüz doğrulaması boş ekran, belge görünümü, arama, içindekiler, küçük resimler, metin, tema, Hakkında ve hata durumu ekran görüntüleriyle yapılır.

## Depo Yapısı

| Dizin | İçerik |
| --- | --- |
| `src/` | GPUI arayüzü, durum yönetimi, MuPDF işçisi ve tema |
| `assets/` | Uygulama logosu ve ikon kaynağı |
| `vendor/mupdf/` | Windows MSVC uyumu için yamalanmış MuPDF Rust sarmalayıcısı |
| `vendor/*.ps1` | Derleme ve geliştirme ortamı kurulum betikleri |
| `docs/` | Belgeler |
| `release/` | Güncel sürümlü uygulama çalıştırıcısı |

## Lisans

- GPUI Kit paketleri (`gpui-kit`, `gpui-component`, `gpui-pre`, `gpui-kit-assets`) Apache-2.0 lisansıyla dağıtılır.
- MuPDF AGPL lisansıyla dağıtılmaktadır; yeniden dağıtım sırasında ilgili lisans yükümlülükleri dikkate alınmalıdır.
- Lucide ikon seti GPUI Kit varlık paketi üzerinden gelir; ayrıntılar paket lisans dosyalarındadır.

---

© 2026 SP GROUP ENERJİ SAN. VE TİC. LTD. ŞTİ. — info@spgroupenerji.com
