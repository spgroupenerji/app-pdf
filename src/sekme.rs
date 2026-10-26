use std::cell::Cell;
use std::collections::{HashMap, HashSet};
use std::rc::Rc;
use std::sync::Arc;
use std::time::Instant;

use gpui_kit::{px, Pixels, RenderImage, ScrollHandle, Size};

/// Sayfa sığdırma modları.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SigdirmaModu {
    Ozel,
    Genislik,
    Sayfa,
}

/// Denetçi panelindeki içerik modları.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum KenarModu {
    Icindekiler,
    KucukResimler,
    Arama,
    Metin,
}

impl KenarModu {
    pub fn indeks(self) -> usize {
        match self {
            Self::Icindekiler => 0,
            Self::KucukResimler => 1,
            Self::Arama => 2,
            Self::Metin => 3,
        }
    }
}

/// Metin arama sonucu.
#[derive(Debug, Clone)]
pub struct AramaSonucu {
    pub sayfa_no: u32,
    pub baglam: String,
}

/// İçindekiler / yer imi öğesi; seviye, gerçek PDF outline girintisini taşır.
#[derive(Debug, Clone)]
pub struct IcindekilerOgesi {
    pub baslik: String,
    pub sayfa_no: u32,
    pub seviye: u8,
}

/// Bir açık PDF belgesinin tüm UI durumu.
pub struct PdfSekme {
    pub id: usize,
    pub baslik: String,
    pub dosya_yolu: String,
    pub simdiki_sayfa: u32,
    pub toplam_sayfa: u32,
    pub zum: f32,
    pub sigdirma_modu: SigdirmaModu,
    pub arama_metni: String,
    pub arama_sonuclari: Vec<AramaSonucu>,
    pub arama_yapiliyor: bool,
    pub icindekiler: Vec<IcindekilerOgesi>,
    pub icindekiler_yukleniyor: bool,
    pub sayfa_metni: String,
    pub metin_yukleniyor: bool,
    pub orjinal_genislik: f32,
    pub orjinal_yukseklik: f32,
    pub goruntu: Option<Arc<RenderImage>>,
    pub goruntu_sayfa: u32,
    pub goruntu_zum: f32,
    pub kucuk_resimler: HashMap<u32, Arc<RenderImage>>,
    pub kucuk_resim_bekleyen: HashSet<u32>,
    pub yukleniyor: bool,
    pub hata_mesaji: Option<String>,
    pub render_nesli: u64,
    pub bekleyen_render: bool,
    pub son_zum_degisimi: Option<Instant>,
    pub otomatik_sigdir: bool,
    pub kaydirma: ScrollHandle,
    pub gorunum_alan: Rc<Cell<Size<Pixels>>>,
    pub render_ekran_olcegi: f32,
}

impl PdfSekme {
    pub fn yeni(id: usize, baslik: String, dosya_yolu: String, ekran_olcegi: f32) -> Self {
        Self {
            id,
            baslik,
            dosya_yolu,
            simdiki_sayfa: 0,
            toplam_sayfa: 0,
            zum: 1.0,
            sigdirma_modu: SigdirmaModu::Ozel,
            arama_metni: String::new(),
            arama_sonuclari: Vec::new(),
            arama_yapiliyor: false,
            icindekiler: Vec::new(),
            icindekiler_yukleniyor: true,
            sayfa_metni: String::new(),
            metin_yukleniyor: false,
            orjinal_genislik: 595.0,
            orjinal_yukseklik: 842.0,
            goruntu: None,
            goruntu_sayfa: 0,
            goruntu_zum: 1.0,
            kucuk_resimler: HashMap::new(),
            kucuk_resim_bekleyen: HashSet::new(),
            yukleniyor: true,
            hata_mesaji: None,
            render_nesli: 0,
            bekleyen_render: false,
            son_zum_degisimi: None,
            otomatik_sigdir: true,
            kaydirma: ScrollHandle::new(),
            gorunum_alan: Rc::new(Cell::new(Size::new(px(0.), px(0.)))),
            render_ekran_olcegi: ekran_olcegi,
        }
    }

    /// Görüntünün mantıksal (nokta) görüntüleme boyutu.
    pub fn gorunum_boyutu(&self) -> (f32, f32) {
        (
            self.orjinal_genislik * self.zum,
            self.orjinal_yukseklik * self.zum,
        )
    }
}

/// GPUI RenderImage BGRA beklediği için RGBA piksel verisini dönüştürür.
pub fn rgba_to_bgra(mut veri: Vec<u8>) -> Vec<u8> {
    for piksel in veri.as_chunks_mut::<4>().0 {
        piksel.swap(0, 2);
    }
    veri
}

/// Zoom değerini güvenli aralığa kırpar.
pub fn zum_sinirla(zum: f32) -> f32 {
    if !zum.is_finite() {
        return 1.0;
    }
    zum.clamp(0.2, 20.0)
}

/// Sığdırma moduna göre hedef zoom değerini hesaplar.
pub fn sigdirma_hesapla(
    alan: Size<Pixels>,
    orjinal_genislik: f32,
    orjinal_yukseklik: f32,
    mod_: SigdirmaModu,
) -> Option<f32> {
    if orjinal_genislik <= 0.0 || orjinal_yukseklik <= 0.0 {
        return None;
    }
    let genislik = (alan.width.as_f32() - 56.0).max(80.0);
    let yukseklik = (alan.height.as_f32() - 56.0).max(80.0);
    if mod_ == SigdirmaModu::Ozel {
        return None;
    }
    let zum = match mod_ {
        SigdirmaModu::Genislik => genislik / orjinal_genislik,
        SigdirmaModu::Sayfa => (genislik / orjinal_genislik).min(yukseklik / orjinal_yukseklik),
        SigdirmaModu::Ozel => return None,
    };
    Some(zum_sinirla(zum))
}

#[cfg(test)]
mod testler {
    use super::*;

    #[test]
    fn zum_sinirlari_uygulanir() {
        assert_eq!(zum_sinirla(0.01), 0.2);
        assert_eq!(zum_sinirla(50.0), 20.0);
        assert_eq!(zum_sinirla(f32::NAN), 1.0);
    }

    #[test]
    fn genislige_sigdirma_orani_hesaplanir() {
        let alan = Size::new(px(656.0), px(800.0));
        let zum = sigdirma_hesapla(alan, 600.0, 800.0, SigdirmaModu::Genislik).unwrap();
        assert!((zum - 1.0).abs() < 0.0001);
    }

    #[test]
    fn sayfaya_sigdirma_kucuk_ekseni_secer() {
        let alan = Size::new(px(656.0), px(456.0));
        let zum = sigdirma_hesapla(alan, 600.0, 800.0, SigdirmaModu::Sayfa).unwrap();
        assert!((zum - 0.5).abs() < 0.0001);
    }

    #[test]
    fn rgba_bgra_donusumu_kanallari_degistirir() {
        assert_eq!(rgba_to_bgra(vec![10, 20, 30, 255]), vec![30, 20, 10, 255]);
    }
}
