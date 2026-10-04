use egui::ColorImage;
use crate::sekme::{AramaSonucu, IcindekilerOgesi};

// Arayuz (UI) ile MuPDF isci is paracigi arasindaki asenkron mesajlasma protokolleri

/// Arayuzden MuPDF iscisine giden talep türleri
#[derive(Debug, Clone)]
pub enum IslemTalebi {
    Render(RenderTalebi),
    KucukResimRender {
        sekme_id: usize,
        dosya_yolu: String,
        sayfa_no: u32,
    },
    MetinArama {
        sekme_id: usize,
        dosya_yolu: String,
        aranan: String,
    },
    SayfaMetniAl {
        sekme_id: usize,
        dosya_yolu: String,
        sayfa_no: u32,
    },
    IcindekilerAl {
        sekme_id: usize,
        dosya_yolu: String,
    },
}

/// Arayuzden MuPDF iscisine giden piksellestirme (render) talebi
#[derive(Debug, Clone)]
pub struct RenderTalebi {
    pub sekme_id: usize,
    pub dosya_yolu: String,
    pub sayfa_no: u32,
    pub zum: f32,
    pub dpi_carpan: f32,
    pub ekran_olcegi: f32,
}

/// MuPDF iscisinden arayuze donen render/işlem sonucu
pub enum RenderSonuc {
    Basarili {
        sekme_id: usize,
        sayfa_no: u32,
        toplam_sayfa: u32,
        orjinal_genislik: f32,
        orjinal_yukseklik: f32,
        goruntu: ColorImage,
    },
    KucukResimHazir {
        sekme_id: usize,
        sayfa_no: u32,
        goruntu: ColorImage,
    },
    SayfaMetniHazir {
        sekme_id: usize,
        metin: String,
    },
    AramaTamamlandi {
        sekme_id: usize,
        sonuclar: Vec<AramaSonucu>,
    },
    IcindekilerHazir {
        sekme_id: usize,
        icindekiler: Vec<IcindekilerOgesi>,
    },
    Hatali {
        sekme_id: usize,
        mesaj: String,
    },
}
