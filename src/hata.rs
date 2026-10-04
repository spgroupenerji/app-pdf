// Uygulama genelinde kullanilan hata tipleri (kullaniciya gosterilecek Turkce mesajlar uretir)

#[derive(Debug)]
pub enum GoruntuleyiciHata {
    BelgeAcilamadi { yol: String, neden: String },
    SayfaYuklenemedi { sayfa: u32, neden: String },
    RasterizasyonHatasi { neden: String },
    GecersizSayfaNo { istenen: u32, toplam: u32 },
    BosBelge { yol: String },
}

impl GoruntuleyiciHata {
    // Hatayi kullaniciya gosterilecek, anlasilir Turkce metne cevirir
    pub fn turkce_mesaj(&self) -> String {
        match self {
            GoruntuleyiciHata::BelgeAcilamadi { yol, neden } => {
                format!("Belge açılamadı ({}): {}", yol, neden)
            }
            GoruntuleyiciHata::SayfaYuklenemedi { sayfa, neden } => {
                format!("Sayfa {} yüklenemedi: {}", sayfa + 1, neden)
            }
            GoruntuleyiciHata::RasterizasyonHatasi { neden } => {
                format!("Sayfa görüntülenemedi: {}", neden)
            }
            GoruntuleyiciHata::GecersizSayfaNo { istenen, toplam } => {
                format!("Geçersiz sayfa numarası: {} / {}", istenen + 1, toplam)
            }
            GoruntuleyiciHata::BosBelge { yol } => {
                format!("Belgede hiç sayfa yok: {}", yol)
            }
        }
    }
}
