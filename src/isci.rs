use std::collections::HashMap;

use crossbeam_channel::{Receiver, Sender};
use mupdf::{Colorspace, Document, Matrix, Outline};

use crate::hata::GoruntuleyiciHata;
use crate::mesaj::{IslemTalebi, RenderSonuc, RenderTalebi, SayfaGoruntusu};
use crate::sekme::{AramaSonucu, IcindekilerOgesi};

const BELGE_ONBELLEK_SINIRI: usize = 8;
const AZAMI_RENDER_BOYUTU: u32 = 16000;
const AZAMI_ARAMA_SONUCU: usize = 100;

/// Arka plan render işçi döngüsü.
/// Tüm MuPDF işlemleri tek iş parçacığında çalışır; `Document` `!Send`'dir.
pub fn render_isci(alici: Receiver<IslemTalebi>, gonderici: Sender<RenderSonuc>) {
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
        if gonderici.send(sonuc).is_err() {
            break;
        }
    }
}

fn talebi_isle(
    talep: &RenderTalebi,
    belge_onbellegi: &mut HashMap<String, Document>,
) -> RenderSonuc {
    let zum = if talep.zum <= 0.0 { 0.1 } else { talep.zum };
    let dpi_carpan = if talep.dpi_carpan <= 0.0 {
        2.0
    } else {
        talep.dpi_carpan
    };
    let ekran_olcegi = if talep.ekran_olcegi <= 0.0 {
        1.0
    } else {
        talep.ekran_olcegi
    };

    match belge_bilgisi_hazirla(&talep.dosya_yolu, belge_onbellegi) {
        Ok(toplam_sayfa) => {
            if toplam_sayfa == 0 {
                return hata_sonucu(
                    talep.sekme_id,
                    Some(talep.nesil),
                    GoruntuleyiciHata::BosBelge {
                        yol: talep.dosya_yolu.clone(),
                    },
                );
            }
            if talep.sayfa_no >= toplam_sayfa {
                return hata_sonucu(
                    talep.sekme_id,
                    Some(talep.nesil),
                    GoruntuleyiciHata::GecersizSayfaNo {
                        istenen: talep.sayfa_no,
                        toplam: toplam_sayfa,
                    },
                );
            }
            match sayfa_rasterize_et(
                &talep.dosya_yolu,
                belge_onbellegi,
                talep.sayfa_no,
                zum,
                dpi_carpan,
                ekran_olcegi,
            ) {
                Ok((goruntu, orjinal_genislik, orjinal_yukseklik)) => RenderSonuc::Basarili {
                    sekme_id: talep.sekme_id,
                    nesil: talep.nesil,
                    sayfa_no: talep.sayfa_no,
                    toplam_sayfa,
                    orjinal_genislik,
                    orjinal_yukseklik,
                    goruntu,
                },
                Err(hata) => hata_sonucu(talep.sekme_id, Some(talep.nesil), hata),
            }
        }
        Err(hata) => hata_sonucu(talep.sekme_id, Some(talep.nesil), hata),
    }
}

fn kucuk_resim_isle(
    sekme_id: usize,
    dosya_yolu: &str,
    sayfa_no: u32,
    belge_onbellegi: &mut HashMap<String, Document>,
) -> RenderSonuc {
    let _ = belge_bilgisi_hazirla(dosya_yolu, belge_onbellegi);
    if let Some(belge) = belge_onbellegi.get(dosya_yolu) {
        if let Ok(sayfa) = belge.load_page(sayfa_no as i32) {
            let donusum = Matrix::new_scale(0.2, 0.2);
            let renk_uzayi = Colorspace::device_rgb();
            if let Ok(pixmap) = sayfa.to_pixmap(&donusum, &renk_uzayi, true, true) {
                return RenderSonuc::KucukResimHazir {
                    sekme_id,
                    sayfa_no,
                    goruntu: pixmap_goruntusu(&pixmap),
                };
            }
        }
    }
    RenderSonuc::Hatali {
        sekme_id,
        nesil: None,
        mesaj: "Küçük resim üretilemedi".to_string(),
    }
}

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
                return RenderSonuc::SayfaMetniHazir { sekme_id, metin };
            }
        }
    }
    RenderSonuc::SayfaMetniHazir {
        sekme_id,
        metin: "Sayfa metni okunamadı veya metin içermiyor.".to_string(),
    }
}

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
        Err(hata) => return hata_sonucu(sekme_id, None, hata),
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
            if sonuclar.len() >= AZAMI_ARAMA_SONUCU {
                break;
            }
        }
    }

    RenderSonuc::AramaTamamlandi { sekme_id, sonuclar }
}

fn icindekiler_al_isle(
    sekme_id: usize,
    dosya_yolu: &str,
    belge_onbellegi: &mut HashMap<String, Document>,
) -> RenderSonuc {
    let mut icindekiler = Vec::new();
    if belge_bilgisi_hazirla(dosya_yolu, belge_onbellegi).is_ok() {
        if let Some(belge) = belge_onbellegi.get(dosya_yolu) {
            if let Ok(outline) = belge.outlines() {
                outline_topla(&outline, 0, &mut icindekiler);
            }
        }
    }
    RenderSonuc::IcindekilerHazir {
        sekme_id,
        icindekiler,
    }
}

fn outline_topla(liste: &[Outline], seviye: u8, hedef: &mut Vec<IcindekilerOgesi>) {
    for oge in liste {
        let sayfa_no = oge
            .dest
            .as_ref()
            .map(|hedef| hedef.loc.page_number)
            .unwrap_or(0);
        hedef.push(IcindekilerOgesi {
            baslik: oge.title.clone(),
            sayfa_no,
            seviye,
        });
        if !oge.down.is_empty() && seviye < 7 {
            outline_topla(&oge.down, seviye + 1, hedef);
        }
    }
}

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
    let sayi = belge
        .page_count()
        .map_err(|e| GoruntuleyiciHata::BelgeAcilamadi {
            yol: yol.to_string(),
            neden: format!("Sayfa sayısı okunamadı: {}", e),
        })?;
    Ok(sayi.max(0) as u32)
}

fn pixmap_goruntusu(pixmap: &mupdf::Pixmap) -> SayfaGoruntusu {
    SayfaGoruntusu {
        genislik: pixmap.width(),
        yukseklik: pixmap.height(),
        rgba: pixmap.samples().to_vec(),
    }
}

fn sayfa_rasterize_et(
    yol: &str,
    belge_onbellegi: &HashMap<String, Document>,
    sayfa_no: u32,
    zum: f32,
    dpi_carpan: f32,
    ekran_olcegi: f32,
) -> Result<(SayfaGoruntusu, f32, f32), GoruntuleyiciHata> {
    let belge = belge_onbellegi
        .get(yol)
        .ok_or_else(|| GoruntuleyiciHata::BelgeAcilamadi {
            yol: yol.to_string(),
            neden: "Belge önbellekte bulunamadı".to_string(),
        })?;

    let sayfa =
        belge
            .load_page(sayfa_no as i32)
            .map_err(|e| GoruntuleyiciHata::SayfaYuklenemedi {
                sayfa: sayfa_no,
                neden: e.to_string(),
            })?;

    let efekt_zum = zum * dpi_carpan * ekran_olcegi;
    let donusum = Matrix::new_scale(efekt_zum, efekt_zum);
    let renk_uzayi = Colorspace::device_rgb();
    let pixmap = sayfa
        .to_pixmap(&donusum, &renk_uzayi, true, true)
        .map_err(|e| GoruntuleyiciHata::RasterizasyonHatasi {
            neden: e.to_string(),
        })?;

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
    Ok((
        pixmap_goruntusu(&pixmap),
        orjinal_genislik,
        orjinal_yukseklik,
    ))
}

fn hata_sonucu(sekme_id: usize, nesil: Option<u64>, hata: GoruntuleyiciHata) -> RenderSonuc {
    RenderSonuc::Hatali {
        sekme_id,
        nesil,
        mesaj: hata.turkce_mesaj(),
    }
}
