use crate::sekme::{AramaSonucu, IcindekilerOgesi};

// Arayüz ile MuPDF işçi iş parçacığı arasındaki asenkron mesajlaşma protokolleri.

/// Arayüzden MuPDF işçisine giden talep türleri.
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

/// Arayüzden MuPDF işçisine giden pikselliştirme (render) talebi.
#[derive(Debug, Clone)]
pub struct RenderTalebi {
    pub sekme_id: usize,
    pub nesil: u64,
    pub dosya_yolu: String,
    pub sayfa_no: u32,
    pub zum: f32,
    pub dpi_carpan: f32,
    pub ekran_olcegi: f32,
}

/// UI katmanının taşıdığı ham RGBA sayfa görüntüsü; MuPDF tipleri sızmaz.
pub struct SayfaGoruntusu {
    pub genislik: u32,
    pub yukseklik: u32,
    pub rgba: Vec<u8>,
}

/// MuPDF işçisinden arayüze dönen işlem sonucu.
pub enum RenderSonuc {
    Basarili {
        sekme_id: usize,
        nesil: u64,
        sayfa_no: u32,
        toplam_sayfa: u32,
        orjinal_genislik: f32,
        orjinal_yukseklik: f32,
        goruntu: SayfaGoruntusu,
    },
    KucukResimHazir {
        sekme_id: usize,
        sayfa_no: u32,
        goruntu: SayfaGoruntusu,
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
        nesil: Option<u64>,
        mesaj: String,
    },
}
