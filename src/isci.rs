use std::collections::HashMap;

use crossbeam_channel::{Receiver, Sender};
use egui::ColorImage;
use mupdf::{Colorspace, Document, Matrix};

use crate::hata::GoruntuleyiciHata;
use crate::mesaj::{IslemTalebi, RenderSonuc, RenderTalebi};
use crate::sekme::{AramaSonucu, IcindekilerOgesi};

// Onbellege alinacak azami belge sayisi (hafiza guvencesi icin sinir)
const BELGE_ONBELLEK_SINIRI: usize = 8;
// Mühendislik çizimleri ve devasa paftalar için azami render boyutu (16K çözünürlük desteği)
const AZAMI_RENDER_BOYUTU: u32 = 16000;

/// Arka plan render isci dongusu.
/// Tum MuPDF islemleri bu TEK is paraciginda calisir; cunku Document tipi !Send'dir
/// ve belge hicbir zaman baska is paracigina tasinamaz (PDF mimarisindeki veri yarisi kurali).
pub fn render_isci(alici: Receiver<IslemTalebi>, gonderici: Sender<RenderSonuc>) {
    // Belge onbellegi: ayni dosyayi tekrar tekrar acip XREF ayristirmasin
    let mut belge_onbellegi: HashMap<String, Document> = HashMap::new();

    while let Ok(islem) = alici.recv() {
        let sonuc = match islem {
            IslemTalebi::Render(talep) => talebi_isle(&talep, &mut belge_onbellegi),
            IslemTalebi::KucukResimRender {
                sekme_id,
                dosya_yolu,
                sayfa_no,
            } => kucuk_resim_isle(sekme_id, &dosya_yolu, sayfa_no, &mut belge_onbellegi),
            IslemTalebi::MetinArama {
                sekme_id,
                dosya_yolu,
                aranan,
            } => metin_ara_isle(sekme_id, &dosya_yolu, &aranan, &mut belge_onbellegi),
            IslemTalebi::SayfaMetniAl {
                sekme_id,
                dosya_yolu,
                sayfa_no,
            } => sayfa_metni_al_isle(sekme_id, &dosya_yolu, sayfa_no, &mut belge_onbellegi),
            IslemTalebi::IcindekilerAl {
                sekme_id,
                dosya_yolu,
            } => icindekiler_al_isle(sekme_id, &dosya_yolu, &mut belge_onbellegi),
        };
        // Arayuz kapandiginda kanal duserse isci sonlanir
        if gonderici.send(sonuc).is_err() {
            break;
        }
    }
}

/// Tek bir render talebini isler: belgeyi acar/onbellekten alir, sayfayi rasterize eder.
fn talebi_isle(talep: &RenderTalebi, belge_onbellegi: &mut HashMap<String, Document>) -> RenderSonuc {
    let zum = if talep.zum <= 0.0 { 0.1 } else { talep.zum };
    let dpi_carpan = if talep.dpi_carpan <= 0.0 { 2.0 } else { talep.dpi_carpan };
    let ekran_olcegi = if talep.ekran_olcegi <= 0.0 { 1.0 } else { talep.ekran_olcegi };

    match belge_bilgisi_hazirla(&talep.dosya_yolu, belge_onbellegi) {
        Ok(toplam_sayfa) => {
            if toplam_sayfa == 0 {
                return hata_sonucu(talep.sekme_id, GoruntuleyiciHata::BosBelge { yol: talep.dosya_yolu.clone() });
            }
            if talep.sayfa_no >= toplam_sayfa {
                return hata_sonucu(
                    talep.sekme_id,
                    GoruntuleyiciHata::GecersizSayfaNo { istenen: talep.sayfa_no, toplam: toplam_sayfa },
                );
            }
            match sayfa_rasterize_et(&talep.dosya_yolu, belge_onbellegi, talep.sayfa_no, zum, dpi_carpan, ekran_olcegi) {
                Ok((goruntu, orjinal_genislik, orjinal_yukseklik)) => RenderSonuc::Basarili {
                    sekme_id: talep.sekme_id,
                    sayfa_no: talep.sayfa_no,
                    toplam_sayfa,
                    orjinal_genislik,
                    orjinal_yukseklik,
                    goruntu,
                },
                Err(hata) => hata_sonucu(talep.sekme_id, hata),
            }
        }
        Err(hata) => hata_sonucu(talep.sekme_id, hata),
    }
}

/// Hızlı küçük resim (Thumbnail) rasterize eder
fn kucuk_resim_isle(
    sekme_id: usize,
    dosya_yolu: &str,
    sayfa_no: u32,
    belge_onbellegi: &mut HashMap<String, Document>,
) -> RenderSonuc {
    let _ = belge_bilgisi_hazirla(dosya_yolu, belge_onbellegi);
    if let Some(belge) = belge_onbellegi.get(dosya_yolu) {
        if let Ok(sayfa) = belge.load_page(sayfa_no as i32) {
            let donusum = Matrix::new_scale(0.2, 0.2); // Hızlı 1/5 küçük boyut
            let renk_uzayi = Colorspace::device_rgb();
            if let Ok(pixmap) = sayfa.to_pixmap(&donusum, &renk_uzayi, true, true) {
                let genislik = pixmap.width() as usize;
                let yukseklik = pixmap.height() as usize;
                let goruntu = ColorImage::from_rgba_unmultiplied([genislik, yukseklik], pixmap.samples());
                return RenderSonuc::KucukResimHazir {
                    sekme_id,
                    sayfa_no,
                    goruntu,
                };
            }
        }
    }
    RenderSonuc::Hatali {
        sekme_id,
        mesaj: "Küçük resim üretilemedi".to_string(),
    }
}

/// Sayfanın tüm ham metnini alır
fn sayfa_metni_al_isle(
    sekme_id: usize,
    dosya_yolu: &str,
    sayfa_no: u32,
    belge_onbellegi: &mut HashMap<String, Document>,
) -> RenderSonuc {
    let _ = belge_bilgisi_hazirla(dosya_yolu, belge_onbellegi);
    if let Some(belge) = belge_onbellegi.get(dosya_yolu) {
        if let Ok(sayfa) = belge.load_page(sayfa_no as i32) {
            if let Ok(metin) = sayfa.text(mupdf::TextExtractOptions::default()) {
                return RenderSonuc::SayfaMetniHazir {
                    sekme_id,
                    metin,
                };
            }
        }
    }
    RenderSonuc::SayfaMetniHazir {
        sekme_id,
        metin: "Sayfa metni okunamadı veya metin içermiyor.".to_string(),
    }
}

/// PDF belgesinde metin araması gerçekleştirir
fn metin_ara_isle(
    sekme_id: usize,
    dosya_yolu: &str,
    aranan: &str,
    belge_onbellegi: &mut HashMap<String, Document>,
) -> RenderSonuc {
    if aranan.trim().is_empty() {
        return RenderSonuc::AramaTamamlandi {
            sekme_id,
            sonuclar: vec![],
        };
    }

    let toplam_sayfa = match belge_bilgisi_hazirla(dosya_yolu, belge_onbellegi) {
        Ok(s) => s,
        Err(hata) => return hata_sonucu(sekme_id, hata),
    };

    let aranan_kucuk = aranan.to_lowercase();
    let mut sonuclar = Vec::new();

    if let Some(belge) = belge_onbellegi.get(dosya_yolu) {
        for i in 0..toplam_sayfa {
            if let Ok(sayfa) = belge.load_page(i as i32) {
                if let Ok(metin) = sayfa.text(mupdf::TextExtractOptions::default()) {
                    if metin.to_lowercase().contains(&aranan_kucuk) {
                        let baglam = metin
                            .lines()
                            .find(|line| line.to_lowercase().contains(&aranan_kucuk))
                            .unwrap_or(&metin)
                            .chars()
                            .take(80)
                            .collect::<String>();

                        sonuclar.push(AramaSonucu {
                            sayfa_no: i,
                            baglam: baglam.trim().to_string(),
                        });
                    }
                }
            }
            if sonuclar.len() >= 100 {
                break; // Azami 100 arama sonucu sınırı
            }
        }
    }

    RenderSonuc::AramaTamamlandi { sekme_id, sonuclar }
}

/// PDF içindekiler / bookmark tablosunu alır
fn icindekiler_al_isle(
    sekme_id: usize,
    dosya_yolu: &str,
    belge_onbellegi: &mut HashMap<String, Document>,
) -> RenderSonuc {
    let mut icindekiler = Vec::new();
    if let Ok(_) = belge_bilgisi_hazirla(dosya_yolu, belge_onbellegi) {
        if let Some(belge) = belge_onbellegi.get(dosya_yolu) {
            if let Ok(sayi) = belge.page_count() {
                for i in 0..sayi {
                    icindekiler.push(IcindekilerOgesi {
                        baslik: format!("Sayfa {}", i + 1),
                        sayfa_no: i as u32,
                    });
                }
            }
        }
    }
    RenderSonuc::IcindekilerHazir {
        sekme_id,
        icindekiler,
    }
}

/// Belgeyi onbellekten alir veya diskten acar; toplam sayfa sayisini dondurur.
fn belge_bilgisi_hazirla(
    yol: &str,
    belge_onbellegi: &mut HashMap<String, Document>,
) -> Result<u32, GoruntuleyiciHata> {
    if !belge_onbellegi.contains_key(yol) {
        if belge_onbellegi.len() >= BELGE_ONBELLEK_SINIRI {
            if let Some(ilk_anahtar) = belge_onbellegi.keys().next().cloned() {
                belge_onbellegi.remove(&ilk_anahtar);
            }
        }
        let belge = Document::open(yol).map_err(|e| GoruntuleyiciHata::BelgeAcilamadi {
            yol: yol.to_string(),
            neden: e.to_string(),
        })?;
        belge_onbellegi.insert(yol.to_string(), belge);
    }

    let belge = belge_onbellegi.get(yol).expect("belge onbellege eklenmeli");
    let sayi = belge.page_count().map_err(|e| GoruntuleyiciHata::BelgeAcilamadi {
        yol: yol.to_string(),
        neden: format!("Sayfa sayısı okunamadı: {}", e),
    })?;
    Ok(sayi.max(0) as u32)
}

/// Sayfayi RGBA piksel dizisine rasterize eder; zero-copy ile egui ColorImage'a donusturur.
/// ekran_olcegi: HiDPI ekranlarda fiziksel piksel bazinda tam supersampling saglamak icin
/// ctx.pixels_per_point() degeri; %125/%150 olcekte bulanikligi giderir.
fn sayfa_rasterize_et(
    yol: &str,
    belge_onbellegi: &HashMap<String, Document>,
    sayfa_no: u32,
    zum: f32,
    dpi_carpan: f32,
    ekran_olcegi: f32,
) -> Result<(ColorImage, f32, f32), GoruntuleyiciHata> {
    let belge = belge_onbellegi.get(yol).ok_or_else(|| GoruntuleyiciHata::BelgeAcilamadi {
        yol: yol.to_string(),
        neden: "Belge önbellekte bulunamadı".to_string(),
    })?;

    let sayfa = belge.load_page(sayfa_no as i32).map_err(|e| GoruntuleyiciHata::SayfaYuklenemedi {
        sayfa: sayfa_no,
        neden: e.to_string(),
    })?;

    let efekt_zum = zum * dpi_carpan * ekran_olcegi;
    let donusum = Matrix::new_scale(efekt_zum, efekt_zum);
    let renk_uzayi = Colorspace::device_rgb();
    let pixmap = sayfa
        .to_pixmap(&donusum, &renk_uzayi, true, true)
        .map_err(|e| GoruntuleyiciHata::RasterizasyonHatasi { neden: e.to_string() })?;

    let genislik = pixmap.width();
    let yukseklik = pixmap.height();
    if genislik > AZAMI_RENDER_BOYUTU || yukseklik > AZAMI_RENDER_BOYUTU {
        return Err(GoruntuleyiciHata::RasterizasyonHatasi {
            neden: format!(
                "Çözünürlük çok yüksek ({}x{}). Yakınlaştırmayı azaltın.",
                genislik, yukseklik
            ),
        });
    }

    let orjinal_genislik = (genislik as f32) / efekt_zum;
    let orjinal_yukseklik = (yukseklik as f32) / efekt_zum;

    let ornekler = pixmap.samples();
    let goruntu = ColorImage::from_rgba_unmultiplied(
        [genislik as usize, yukseklik as usize],
        ornekler,
    );
    Ok((goruntu, orjinal_genislik, orjinal_yukseklik))
}

fn hata_sonucu(sekme_id: usize, hata: GoruntuleyiciHata) -> RenderSonuc {
    RenderSonuc::Hatali {
        sekme_id,
        mesaj: hata.turkce_mesaj(),
    }
}
