# PDF Görüntüleyici

MuPDF ve egui tabanlı, çok iş parçacıklı Windows PDF görüntüleyici.

## Hazır uygulama

`release/app-pdf_vYYYYMMDDHHMM.exe` — kurulum gerektirmez, doğrudan çalıştırılır.

## Kaynaktan derleme

Gereksinimler: Rust (MSVC hedefi), Visual Studio C++ Build Tools ve LLVM (`C:\Program Files\LLVM`).

```powershell
.\vendor\derle.ps1
```

Derleme sonunda çıktı `release\app-pdf_vYYYYMMDDHHMM.exe` adıyla kopyalanır; dosya adındaki damga, uygulama içindeki sürümle aynıdır.

## Geliştirici kurulumu (taze bilgisayar)

```powershell
git clone https://github.com/spgroupenerji/app-pdf.git
cd app-pdf
powershell -ExecutionPolicy Bypass -File vendor\hazirla.ps1
```

Betik ön koşulları denetler (Rust, LLVM, MSVC), bağımlılıkları indirir ve derler.

Not: Cargo derleme önbelleği (`target/`) makineye ve yola özgüdür; depoya dahil edilmez ve kopyalanamaz (Cargo, mtime + mutlak yol tabanlı fingerprint kullanır). Taze bilgisayarda ilk derleme MuPDF C çekirdeğini sıfırdan derlediğinden uzun sürer; sonraki derlemeler artımlıdır. Yalnızca uygulamayı kullanmak isteyenler geliştirme ortamı kurmak zorunda değildir: `release/` klasöründeki en güncel `app-pdf_v...exe` doğrudan indirilebilir.

## Sürümlandırma

Her derleme, derleme anını gösteren `vYYYYMMDDHHMM` (yıl-ay-gün-saat-dakika) etiketini otomatik alır; sürüm uygulamanın Hakkında penceresinde görünür.

## Depo yapısı

- `src/` — uygulama kaynak kodu
- `vendor/mupdf/` — Windows MSVC uyumu için yamalanmış MuPDF Rust sarmalayıcısı (`Cargo.toml` içinde `[patch.crates-io]` ile kullanılır)
- `docs/` — belgeler

## Lisans

Uygulama bağımlılıklarından MuPDF AGPL lisanslıdır; dağıtım sırasında lisans yükümlülüklerini dikkate alın.
