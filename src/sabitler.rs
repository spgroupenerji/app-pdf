// Uygulama sabitleri: sürüm, firma bilgileri ve dokunmatik tasarım değerleri.

/// Arayüzde gösterilen sürüm etiketi. derle.ps1 her derlemede
/// UYGULAMA_SURUMU=beta-vYYYYMMDDHHMM ortam değişkenini basar.
pub const SURUM: &str = match option_env!("UYGULAMA_SURUMU") {
    Some(v) => v,
    None => "beta-v2026.07.24",
};

/// Geliştirici / firma bilgileri (Hakkında penceresinde gösterilir).
pub const GELISTIRICI: &str = "Serkan POTUK";
pub const FIRMA: &str = "SP GROUP ENERJİ SAN. VE TİC. LTD. ŞTİ.";
pub const VERGI_DAIRESI: &str = "Sarayönü Mal Müdürlüğü";
pub const VERGI_NUMARASI: &str = "7811025029";
pub const POSTA_KODU: &str = "42430";
pub const ADRES: &str = "Doğu İstasyon Mah. 148207 Sok. No:12/B, (PK:42430) Sarayönü/KONYA";
pub const E_POSTA: &str = "info@spgroupenerji.com";
pub const WEB: &str = "www.spgroupenerji.com";

/// Dokunmatik arayüz için minimum dokunma hedefi (px). WCAG 2.5.5 (AAA) ve Apple HIG: 44px.
pub const DOKUNMA_HEDEFI: f32 = 44.0;

/// Varsayılan pencere boyutu (mantıksal puan).
pub const PENCERE_GENISLIK: f32 = 1280.0;
pub const PENCERE_YUKSEKLIK: f32 = 860.0;
pub const PENCERE_MIN_GENISLIK: f32 = 960.0;
pub const PENCERE_MIN_YUKSEKLIK: f32 = 640.0;
