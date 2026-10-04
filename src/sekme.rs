use std::collections::HashMap;
use egui::TextureHandle;

// DockState icinde tutulan her bir acik PDF belgesinin veri yapisidir

/// Sayfa sığdırma modları
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum SigdirmaModu {
    Ozel,
    Genislik,
    Sayfa,
}

/// Sol yan panel sekme modları
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum SolPanelModu {
    Kapali,
    Icindekiler,
    KucukResimler,
    Arama,
    MetinKopyala,
}

/// Metin arama sonucu
#[derive(Debug, Clone)]
pub struct AramaSonucu {
    pub sayfa_no: u32,
    pub baglam: String,
}

/// İçindekiler / Bookmark öğesi
#[derive(Debug, Clone)]
pub struct IcindekilerOgesi {
    pub baslik: String,
    pub sayfa_no: u32,
}

/// Bir acik PDF belgesini temsil eden sekme durumu.
pub struct PdfSekme {
    pub id: usize,
    pub baslik: String,
    pub dosya_yolu: String,
    pub simdiki_sayfa: u32,
    pub toplam_sayfa: u32,
    pub sayfa_girdi_metni: String,
    pub zum: f32,
    pub sigdirma_modu: SigdirmaModu,
    pub sol_panel_modu: SolPanelModu,
    pub arama_metni: String,
    pub arama_sonuclari: Vec<AramaSonucu>,
    pub arama_yapiliyor: bool,
    pub icindekiler: Vec<IcindekilerOgesi>,
    pub sayfa_metni: String,
    pub metin_yukleniyor: bool,
    pub orjinal_genislik: f32,
    pub orjinal_yukseklik: f32,
    pub doku: Option<TextureHandle>,
    pub kucuk_resimler: HashMap<u32, TextureHandle>,
    pub yukleniyor: bool,
    pub hata_mesaji: Option<String>,
    /// Bu sekmenin render edildigi ekran olcegi; HiDPI'de net supersampling saglar.
    pub render_ekran_olcegi: f32,
}

impl PdfSekme {
    /// Yeni bir sekme olusturur; ilk sayfanin render edilmesi beklenir.
    pub fn yeni(id: usize, baslik: String, dosya_yolu: String, ekran_olcegi: f32) -> Self {
        Self {
            id,
            baslik,
            dosya_yolu,
            simdiki_sayfa: 0,
            toplam_sayfa: 0,
            sayfa_girdi_metni: "1".to_string(),
            zum: 1.0,
            sigdirma_modu: SigdirmaModu::Ozel,
            sol_panel_modu: SolPanelModu::Kapali,
            arama_metni: String::new(),
            arama_sonuclari: Vec::new(),
            arama_yapiliyor: false,
            icindekiler: Vec::new(),
            sayfa_metni: String::new(),
            metin_yukleniyor: false,
            orjinal_genislik: 595.0, // Varsayilan A4 genislik (pt)
            orjinal_yukseklik: 842.0, // Varsayilan A4 yukseklik (pt)
            doku: None,
            kucuk_resimler: HashMap::new(),
            yukleniyor: true,
            hata_mesaji: None,
            render_ekran_olcegi: ekran_olcegi,
        }
    }
}
