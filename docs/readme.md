# SP GROUP PDF Görüntüleyici

MuPDF render motoru ve egui arayüz çerçevesiyle geliştirilen, çok iş parçacıklı mimariye sahip Windows PDF görüntüleyicisi.

## Özellikler

- Sekmeli arayüzle birden fazla belgenin eş zamanlı görüntülenmesi
- İçindekiler, sayfa önizlemeleri, tam metin arama ve panoya kopyalama panelleri
- Genişliğe/sayfaya sığdırma, Ctrl+tekerlek ile akıcı yakınlaştırma ve yüzde yüz sıfırlama
- Sürükle-bırak ile toplu belge açma ve son açılan dosyalar listesi
- Klavye kısayollarıyla hızlı sayfa gezinme (ok tuşları, PgUp/PgDn, Home/End, Ctrl+F)
- Kurumsal koyu/açık tema; tema tercihi otomatik olarak hatırlanır
- WCAG 2.5.5 ve Apple HIG ile uyumlu 44 px dokunma hedefleri
- 2,5x supersampling ile yüksek çözünürlüklü ekranlarda keskin sayfa render'ı

## Hazır Uygulama

`release` klasöründeki `app-pdf_vYYYYMMDDHHMM.exe` dosyası kurulum gerektirmez ve doğrudan çalıştırılır. Klasörde yalnızca en güncel sürüm bulundurulur; her derlemede önceki sürümler otomatik olarak kaldırılır. Dosya adındaki `vYYYYMMDDHHMM` damgası derleme anını gösterir ve uygulama içindeki sürüm bilgisiyle aynıdır.

## Kaynaktan Derleme

Gereksinimler: Rust (MSVC hedefi), Visual Studio C++ Build Tools ve LLVM (`C:\Program Files\LLVM`).

```powershell
.\vendor\derle.ps1
```

Betik sürüm damgasını oluşturur, uygulamayı derler ve çıktıyı `release\app-pdf_vYYYYMMDDHHMM.exe` adıyla kopyalar. İlk derleme MuPDF C çekirdeğini sıfırdan derlediğinden uzun sürer; sonraki derlemeler artımlıdır.

## Geliştirici Kurulumu (Yeni Bilgisayar)

```powershell
git clone https://github.com/spgroupenerji/app-pdf.git
cd app-pdf
powershell -ExecutionPolicy Bypass -File vendor\hazirla.ps1
```

Betik ön koşulları denetler (Rust, LLVM, MSVC), bağımlılıkları indirir ve derler. Cargo derleme önbelleği (`target/`) makineye ve yola özgüdür; depoya dahil edilmez ve kopyalanamaz.

## Güvenlik

- Uygulama yalnızca yerel PDF dosyalarını işler; arka planda ağ isteği yapmaz ve kullanıcı verisi toplamaz.
- Tercihler ve son açılan dosyalar listesi yalnızca yerel kullanıcı dizininde (`%APPDATA%\pdf_goruntuleyici`) saklanır.
- Belge ayrıştırma ve render işlemleri, arayüz iş parçacığından bağımsız bir iş parçacığında yürütülür; sorunlu belgeler arayüzü kilitlemez.

## Sürümlandırma

Her derleme, derleme anını gösteren `vYYYYMMDDHHMM` (yıl-ay-gün-saat-dakika) damgasını otomatik alır; sürüm bilgisi uygulamanın Hakkında penceresinde görüntülenir.

## Depo Yapısı

| Dizin | İçerik |
| --- | --- |
| `src/` | Uygulama kaynak kodu |
| `assets/` | Uygulama logosu ve ikon kaynağı |
| `vendor/mupdf/` | Windows MSVC uyumu için yamalanmış MuPDF Rust sarmalayıcısı |
| `vendor/*.ps1` | Derleme ve geliştirme ortamı kurulum betikleri |
| `docs/` | Belgeler |
| `release/` | Güncel sürümlü uygulama çalıştırıcısı |

## Lisans

Uygulama bağımlılıklarından MuPDF AGPL lisansıyla dağıtılmaktadır; yeniden dağıtım sırasında ilgili lisans yükümlülükleri dikkate alınmalıdır.

---

© 2026 SP GROUP ENERJİ SAN. VE TİC. LTD. ŞTİ. — info@spgroupenerji.com
