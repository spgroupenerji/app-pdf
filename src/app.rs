use std::fs;
use std::path::{Path, PathBuf};
use std::sync::Arc;
use std::time::{Duration, Instant};

use crossbeam_channel::{Receiver, Sender};
use gpui_kit::component::input::{InputEvent, InputState};
use gpui_kit::component::TitleBar;
use gpui_kit::*;
use rfd::FileDialog;

use crate::isci::render_isci;
use crate::mesaj::{IslemTalebi, RenderSonuc, RenderTalebi, SayfaGoruntusu};
use crate::sabitler::{
    PENCERE_GENISLIK, PENCERE_MIN_GENISLIK, PENCERE_MIN_YUKSEKLIK, PENCERE_YUKSEKLIK,
};
use crate::sekme::{sigdirma_hesapla, zum_sinirla, KenarModu, PdfSekme, SigdirmaModu};
use crate::tema;

const SON_DOSYALAR_AZAMI: usize = 10;
const DPI_CARPANI: f32 = 2.5;
const ZUM_BEKLEME: Duration = Duration::from_millis(140);
const BILDIRIM_SURESI: Duration = Duration::from_secs(3);

gpui_kit::actions!(
    pdf_goruntuleyici,
    [
        DosyaAc,
        SonDosyalar,
        SekmeKapat,
        Cikis,
        KenarPaneli,
        Icindekiler,
        Sayfalar,
        AramaPaneli,
        MetinPaneli,
        TemaDegistir,
        Yakinlastir,
        Uzaklastir,
        ZumSifirla,
        GenisligeSigdir,
        SayfayaSigdir,
        OncekiSayfa,
        SonrakiSayfa,
        IlkSayfa,
        SonSayfa,
        Hakkinda,
        YenidenDene,
        KaplamaKapat,
    ]
);

#[derive(Debug, Clone, Default)]
pub struct UygulamaAyarlari {
    pub son_dosyalar: Vec<String>,
}

impl UygulamaAyarlari {
    fn yapilandirma_yolu() -> Option<PathBuf> {
        let taban = std::env::var("APPDATA")
            .ok()
            .map(PathBuf::from)
            .or_else(|| std::env::var("USERPROFILE").ok().map(PathBuf::from));
        taban.map(|mut p| {
            p.push("pdf_goruntuleyici");
            p.push("son_dosyalar.txt");
            p
        })
    }

    pub fn yukle() -> Self {
        let mut ayarlar = Self::default();
        if let Some(yol) = Self::yapilandirma_yolu() {
            if yol.exists() {
                if let Ok(veri) = fs::read_to_string(&yol) {
                    for satir in veri.lines() {
                        let temiz = satir.trim();
                        if !temiz.is_empty() && Path::new(temiz).exists() {
                            ayarlar.son_dosyalar.push(temiz.to_string());
                        }
                    }
                }
            }
        }
        ayarlar
    }

    pub fn kaydet(&self) {
        if let Some(yol) = Self::yapilandirma_yolu() {
            if let Some(parent) = yol.parent() {
                let _ = fs::create_dir_all(parent);
            }
            let _ = fs::write(yol, self.son_dosyalar.join("\n"));
        }
    }

    pub fn dosya_ekle(&mut self, dosya_yolu: &str) {
        let yol_str = dosya_yolu.to_string();
        self.son_dosyalar.retain(|y| y != &yol_str);
        self.son_dosyalar.insert(0, yol_str);
        if self.son_dosyalar.len() > SON_DOSYALAR_AZAMI {
            self.son_dosyalar.truncate(SON_DOSYALAR_AZAMI);
        }
        self.kaydet();
    }

    fn ayarlar_yolu() -> Option<PathBuf> {
        let taban = std::env::var("APPDATA")
            .ok()
            .map(PathBuf::from)
            .or_else(|| std::env::var("USERPROFILE").ok().map(PathBuf::from));
        taban.map(|mut p| {
            p.push("pdf_goruntuleyici");
            p.push("ayarlar.txt");
            p
        })
    }

    pub fn tema_kaydet(koyu: bool) {
        if let Some(yol) = Self::ayarlar_yolu() {
            if let Some(parent) = yol.parent() {
                let _ = fs::create_dir_all(parent);
            }
            let _ = fs::write(
                yol,
                format!("koyu_tema={}\n", if koyu { "true" } else { "false" }),
            );
        }
    }

    pub fn tema_yukle() -> bool {
        if let Some(yol) = Self::ayarlar_yolu() {
            if let Ok(veri) = fs::read_to_string(&yol) {
                for satir in veri.lines() {
                    if let Some(deger) = satir.trim().strip_prefix("koyu_tema=") {
                        return deger == "true";
                    }
                }
            }
        }
        false
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BildirimTuru {
    Hata,
    Basari,
}

#[derive(Debug, Clone)]
pub struct Bildirim {
    pub mesaj: String,
    pub tur: BildirimTuru,
    pub kimlik: u64,
}

/// Uygulamanın merkezi durumu ve GPUI görünümü.
pub struct GoruntuleyiciUygulama {
    pub belgeler: Vec<PdfSekme>,
    pub aktif_id: Option<usize>,
    pub sonraki_id: usize,
    pub ayarlar: UygulamaAyarlari,
    pub koyu_tema: bool,
    pub kenar_acik: bool,
    pub kenar_modu: KenarModu,
    pub hakkinda_acik: bool,
    pub son_dosyalar_acik: bool,
    pub talep_gonderici: Sender<IslemTalebi>,
    pub sonuc_alici: Receiver<RenderSonuc>,
    pub sayfa_girdisi: Entity<InputState>,
    pub arama_girdisi: Entity<InputState>,
    _sayfa_abone: Subscription,
    _arama_abone: Subscription,
    pub ekran_olcegi: f32,
    pub logo: Option<Arc<RenderImage>>,
    pub bildirim: Option<Bildirim>,
    pub surukleme_uzerinde: bool,
    dongu_basladi: bool,
    pub sayfa_girdisi_guncelle: bool,
    pub arama_girdisi_guncelle: bool,
    pub odak: FocusHandle,
    bildirim_sayaci: u64,
}

impl GoruntuleyiciUygulama {
    pub fn yeni(window: &mut Window, cx: &mut Context<Self>) -> Self {
        let (talep_tx, talep_rx) = crossbeam_channel::unbounded();
        let (sonuc_tx, sonuc_rx) = crossbeam_channel::unbounded();

        std::thread::Builder::new()
            .name("mupdf-render-isci".to_string())
            .spawn(move || render_isci(talep_rx, sonuc_tx))
            .expect("Render işçi iş parçacığı başlatılamadı");

        let ayarlar = UygulamaAyarlari::yukle();
        let koyu_tema = UygulamaAyarlari::tema_yukle();

        let sayfa_girdisi = cx.new(|cx| {
            InputState::new(window, cx)
                .default_value("1")
                .placeholder("1")
        });
        let arama_girdisi = cx.new(|cx| InputState::new(window, cx).placeholder("Belgede ara…"));

        let sayfa_abone = cx.subscribe(&sayfa_girdisi, |this, state, olay, cx| {
            if matches!(olay, InputEvent::PressEnter { .. }) {
                let metin = state.read(cx).value().to_string();
                this.sayfa_girisi_uygula(&metin, cx);
            }
        });
        let arama_abone = cx.subscribe(&arama_girdisi, |this, state, olay, cx| {
            if matches!(olay, InputEvent::PressEnter { .. }) {
                let metin = state.read(cx).value().to_string();
                this.arama_yap_metin(metin, cx);
            }
        });

        let odak = cx.focus_handle();

        Self {
            belgeler: Vec::new(),
            aktif_id: None,
            sonraki_id: 0,
            ayarlar,
            koyu_tema,
            kenar_acik: false,
            kenar_modu: KenarModu::Icindekiler,
            hakkinda_acik: false,
            son_dosyalar_acik: false,
            talep_gonderici: talep_tx,
            sonuc_alici: sonuc_rx,
            sayfa_girdisi,
            arama_girdisi,
            _sayfa_abone: sayfa_abone,
            _arama_abone: arama_abone,
            ekran_olcegi: window.scale_factor(),
            logo: logo_yukle(),
            bildirim: None,
            surukleme_uzerinde: false,
            dongu_basladi: false,
            sayfa_girdisi_guncelle: false,
            arama_girdisi_guncelle: false,
            odak,
            bildirim_sayaci: 0,
        }
    }

    pub fn dongu_baslat(&mut self, cx: &mut Context<Self>) {
        if self.dongu_basladi {
            return;
        }
        self.dongu_basladi = true;
        cx.spawn(async move |this, cx| loop {
            cx.background_executor()
                .timer(Duration::from_millis(40))
                .await;
            let devam = this.update(cx, |this, cx| {
                let degisti_1 = this.sonuclari_isle(cx);
                let degisti_2 = this.bekleyenleri_isle(cx);
                if degisti_1 || degisti_2 {
                    cx.notify();
                }
            });
            if devam.is_err() {
                break;
            }
        })
        .detach();
    }

    pub fn aktif_belge(&self) -> Option<&PdfSekme> {
        let id = self.aktif_id?;
        self.belgeler.iter().find(|b| b.id == id)
    }

    pub fn belge(&self, id: usize) -> Option<&PdfSekme> {
        self.belgeler.iter().find(|b| b.id == id)
    }

    pub fn belge_mut(&mut self, id: usize) -> Option<&mut PdfSekme> {
        self.belgeler.iter_mut().find(|b| b.id == id)
    }

    pub fn aktif_belge_mut(&mut self) -> Option<&mut PdfSekme> {
        let id = self.aktif_id?;
        self.belgeler.iter_mut().find(|b| b.id == id)
    }

    pub fn sekme_basligi(&self) -> String {
        match self.aktif_belge() {
            Some(belge) => format!("{} — SP GROUP PDF", belge.baslik),
            None => "SP GROUP PDF Görüntüleyici".to_string(),
        }
    }

    pub fn dosya_sec(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        if let Some(dosyalar) = FileDialog::new()
            .add_filter("PDF Belgesi", &["pdf"])
            .set_title("PDF Belgesi veya Belgeleri Seç")
            .pick_files()
        {
            for yol in dosyalar {
                if let Some(yol) = yol.to_str() {
                    self.dosya_ac(yol, window, cx);
                }
            }
        }
    }

    pub fn dosya_ac(&mut self, yol: &str, window: &mut Window, cx: &mut Context<Self>) {
        if !pdf_mi(yol) {
            self.bildir("Yalnızca PDF belgeleri açılabilir.", BildirimTuru::Hata, cx);
            return;
        }
        self.ekran_olcegi = window.scale_factor();
        self.sonraki_id += 1;
        let baslik = Path::new(yol)
            .file_name()
            .and_then(|s| s.to_str())
            .unwrap_or("Belge")
            .to_string();
        let sekme = PdfSekme::yeni(self.sonraki_id, baslik, yol.to_string(), self.ekran_olcegi);
        let id = sekme.id;
        self.belgeler.push(sekme);
        self.aktif_id = Some(id);
        self.ayarlar.dosya_ekle(yol);
        self.sayfa_girdisi_guncelle = true;
        self.arama_girdisi_guncelle = true;
        self.kenar_modu = KenarModu::Icindekiler;
        self.kenar_acik = true;
        self.icindekiler_talebi_gonder(id);
        self.render_talebi_gonder_id(id, cx);
        window.set_window_title(&self.sekme_basligi());
        window.focus(&self.odak, cx);
        cx.notify();
    }

    pub fn sekmeye_gec(&mut self, id: usize, window: &mut Window, cx: &mut Context<Self>) {
        if self.belgeler.iter().any(|b| b.id == id) {
            self.aktif_id = Some(id);
            self.sayfa_girdisi_guncelle = true;
            self.arama_girdisi_guncelle = true;
            window.set_window_title(&self.sekme_basligi());
            window.focus(&self.odak, cx);
            cx.notify();
        }
    }

    pub fn sekme_kapat(&mut self, id: usize, window: &mut Window, cx: &mut Context<Self>) {
        let Some(konum) = self.belgeler.iter().position(|b| b.id == id) else {
            return;
        };
        self.belgeler.remove(konum);
        if self.aktif_id == Some(id) {
            self.aktif_id = self
                .belgeler
                .get(konum)
                .or_else(|| self.belgeler.last())
                .map(|b| b.id);
            self.sayfa_girdisi_guncelle = true;
            self.arama_girdisi_guncelle = true;
            window.set_window_title(&self.sekme_basligi());
        }
        if self.belgeler.is_empty() {
            self.kenar_acik = false;
            self.hakkinda_acik = false;
            self.son_dosyalar_acik = false;
        }
        cx.notify();
    }

    pub fn aktif_sekmeyi_kapat(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        if let Some(id) = self.aktif_id {
            self.sekme_kapat(id, window, cx);
        }
    }

    pub fn sayfa_girisi_uygula(&mut self, metin: &str, cx: &mut Context<Self>) {
        let Some(id) = self.aktif_id else {
            return;
        };
        let toplam = self.belge(id).map(|b| b.toplam_sayfa).unwrap_or(0);
        if let Ok(sayfa) = metin.trim().parse::<u32>() {
            if sayfa >= 1 && sayfa <= toplam {
                self.sayfaya_git(id, sayfa - 1, cx);
                return;
            }
        }
        self.sayfa_girdisi_guncelle = true;
        cx.notify();
    }

    pub fn sayfaya_git(&mut self, id: usize, yeni_sayfa: u32, cx: &mut Context<Self>) {
        let gecersiz = {
            let Some(sekme) = self.belge_mut(id) else {
                return;
            };
            if sekme.toplam_sayfa == 0
                || yeni_sayfa >= sekme.toplam_sayfa
                || yeni_sayfa == sekme.simdiki_sayfa
            {
                true
            } else {
                sekme.simdiki_sayfa = yeni_sayfa;
                sekme.goruntu = None;
                sekme.yukleniyor = true;
                sekme.hata_mesaji = None;
                sekme.kaydirma.set_offset(point(px(0.), px(0.)));
                false
            }
        };
        if gecersiz {
            return;
        }
        self.sayfa_girdisi_guncelle = true;
        self.render_talebi_gonder_id(id, cx);
        if self.kenar_modu == KenarModu::Metin {
            self.metin_talebi_gonder(id, cx);
        }
        cx.notify();
    }

    pub fn sayfa_ileri(&mut self, ileri: bool, cx: &mut Context<Self>) {
        let Some(sekme) = self.aktif_belge() else {
            return;
        };
        let id = sekme.id;
        let mevcut = sekme.simdiki_sayfa;
        let toplam = sekme.toplam_sayfa;
        if toplam == 0 {
            return;
        }
        let hedef = if ileri {
            (mevcut + 1).min(toplam - 1)
        } else {
            mevcut.saturating_sub(1)
        };
        self.sayfaya_git(id, hedef, cx);
    }

    pub fn ilk_veya_son_sayfa(&mut self, son: bool, cx: &mut Context<Self>) {
        let Some(sekme) = self.aktif_belge() else {
            return;
        };
        let id = sekme.id;
        let toplam = sekme.toplam_sayfa;
        if toplam == 0 {
            return;
        }
        let hedef = if son { toplam - 1 } else { 0 };
        self.sayfaya_git(id, hedef, cx);
    }

    pub fn zum_ayarla(&mut self, yeni_zum: f32, cx: &mut Context<Self>) {
        let Some(sekme) = self.aktif_belge_mut() else {
            return;
        };
        let sinirli = zum_sinirla(yeni_zum);
        if (sekme.zum - sinirli).abs() < 0.001 {
            return;
        }
        sekme.zum = sinirli;
        sekme.sigdirma_modu = SigdirmaModu::Ozel;
        sekme.otomatik_sigdir = false;
        sekme.bekleyen_render = true;
        sekme.son_zum_degisimi = Some(Instant::now());
        sekme.render_nesli = sekme.render_nesli.wrapping_add(1);
        sekme.yukleniyor = true;
        cx.notify();
    }

    pub fn zum_carki(&mut self, delta_y: f32, cx: &mut Context<Self>) {
        let Some(sekme) = self.aktif_belge() else {
            return;
        };
        let carpan = if delta_y > 0.0 { 1.12 } else { 1.0 / 1.12 };
        let yeni = sekme.zum * carpan;
        self.zum_ayarla(yeni, cx);
    }

    pub fn zum_pinch(&mut self, delta: f32, cx: &mut Context<Self>) {
        let Some(sekme) = self.aktif_belge() else {
            return;
        };
        let yeni = sekme.zum * (1.0 + delta);
        self.zum_ayarla(yeni, cx);
    }

    pub fn zum_sifirla(&mut self, cx: &mut Context<Self>) {
        if let Some(sekme) = self.aktif_belge_mut() {
            sekme.sigdirma_modu = SigdirmaModu::Ozel;
        }
        self.zum_ayarla(1.0, cx);
    }

    pub fn sigdir(&mut self, mod_: SigdirmaModu, cx: &mut Context<Self>) {
        let Some(sekme) = self.aktif_belge() else {
            return;
        };
        let id = sekme.id;
        let alan = sekme.gorunum_alan.get();
        let hedef = sigdirma_hesapla(alan, sekme.orjinal_genislik, sekme.orjinal_yukseklik, mod_);
        if let Some(zum) = hedef {
            if let Some(sekme) = self.belge_mut(id) {
                sekme.zum = zum;
                sekme.sigdirma_modu = mod_;
                sekme.otomatik_sigdir = false;
                sekme.bekleyen_render = true;
                sekme.son_zum_degisimi = Some(Instant::now());
                sekme.render_nesli = sekme.render_nesli.wrapping_add(1);
                sekme.yukleniyor = true;
            }
            cx.notify();
        }
    }

    pub fn arama_yap_metin(&mut self, metin: String, cx: &mut Context<Self>) {
        let Some(id) = self.aktif_id else {
            return;
        };
        {
            let Some(sekme) = self.belge_mut(id) else {
                return;
            };
            sekme.arama_metni = metin.clone();
            sekme.arama_sonuclari.clear();
            sekme.arama_yapiliyor = !metin.trim().is_empty();
        }
        self.kenar_acik = true;
        self.kenar_modu = KenarModu::Arama;
        if !metin.trim().is_empty() {
            if let Some(sekme) = self.belge(id) {
                let _ = self.talep_gonderici.send(IslemTalebi::MetinArama {
                    sekme_id: id,
                    dosya_yolu: sekme.dosya_yolu.clone(),
                    aranan: metin,
                });
            }
        }
        cx.notify();
    }

    pub fn metin_yukle(&mut self, id: usize, cx: &mut Context<Self>) {
        let gonder = {
            let Some(sekme) = self.belge_mut(id) else {
                return;
            };
            if sekme.metin_yukleniyor {
                false
            } else {
                sekme.metin_yukleniyor = true;
                true
            }
        };
        if gonder {
            if let Some(sekme) = self.belge(id) {
                let _ = self.talep_gonderici.send(IslemTalebi::SayfaMetniAl {
                    sekme_id: id,
                    dosya_yolu: sekme.dosya_yolu.clone(),
                    sayfa_no: sekme.simdiki_sayfa,
                });
            }
            cx.notify();
        }
    }

    pub fn kenar_modu_ayarla(
        &mut self,
        mod_: KenarModu,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        self.kenar_acik = true;
        self.kenar_modu = mod_;
        if mod_ == KenarModu::Arama {
            let odak = self.arama_girdisi.read(cx).focus_handle(cx);
            window.focus(&odak, cx);
        }
        if mod_ == KenarModu::Metin {
            if let Some(id) = self.aktif_id {
                self.metin_yukle(id, cx);
            }
        }
        cx.notify();
    }

    pub fn tema_degistir(&mut self, cx: &mut Context<Self>) {
        self.koyu_tema = !self.koyu_tema;
        UygulamaAyarlari::tema_kaydet(self.koyu_tema);
        tema::tema_uygula(cx, self.koyu_tema);
        cx.notify();
    }

    pub fn metni_kopyala(&mut self, cx: &mut Context<Self>) {
        let metin = self
            .aktif_belge()
            .map(|b| b.sayfa_metni.clone())
            .unwrap_or_default();
        if metin.is_empty() {
            self.bildir("Kopyalanacak metin yok.", BildirimTuru::Hata, cx);
            return;
        }
        cx.write_to_clipboard(ClipboardItem::new_string(metin));
        self.bildir("Sayfa metni panoya kopyalandı.", BildirimTuru::Basari, cx);
    }

    pub fn bildir(&mut self, mesaj: impl Into<String>, tur: BildirimTuru, cx: &mut Context<Self>) {
        self.bildirim_sayaci += 1;
        let kimlik = self.bildirim_sayaci;
        self.bildirim = Some(Bildirim {
            mesaj: mesaj.into(),
            tur,
            kimlik,
        });
        cx.spawn(async move |this, cx| {
            cx.background_executor().timer(BILDIRIM_SURESI).await;
            let _ = this.update(cx, |this, cx| {
                if this
                    .bildirim
                    .as_ref()
                    .is_some_and(|bildirim| bildirim.kimlik == kimlik)
                {
                    this.bildirim = None;
                    cx.notify();
                }
            });
        })
        .detach();
        cx.notify();
    }

    fn icindekiler_talebi_gonder(&mut self, id: usize) {
        let dosya_yolu = {
            let Some(sekme) = self.belge_mut(id) else {
                return;
            };
            sekme.icindekiler_yukleniyor = true;
            sekme.dosya_yolu.clone()
        };
        let _ = self.talep_gonderici.send(IslemTalebi::IcindekilerAl {
            sekme_id: id,
            dosya_yolu,
        });
    }

    fn metin_talebi_gonder(&mut self, id: usize, cx: &mut Context<Self>) {
        let gonder = {
            let Some(sekme) = self.belge_mut(id) else {
                return;
            };
            if sekme.metin_yukleniyor {
                false
            } else {
                sekme.metin_yukleniyor = true;
                true
            }
        };
        if gonder {
            if let Some(sekme) = self.belge(id) {
                let _ = self.talep_gonderici.send(IslemTalebi::SayfaMetniAl {
                    sekme_id: id,
                    dosya_yolu: sekme.dosya_yolu.clone(),
                    sayfa_no: sekme.simdiki_sayfa,
                });
            }
            cx.notify();
        }
    }

    pub(crate) fn render_talebi_gonder_id(&mut self, id: usize, cx: &mut Context<Self>) {
        let ekran_olcegi = self.ekran_olcegi;
        let talep = {
            let Some(sekme) = self.belge_mut(id) else {
                return;
            };
            sekme.render_nesli = sekme.render_nesli.wrapping_add(1);
            sekme.bekleyen_render = false;
            sekme.son_zum_degisimi = None;
            sekme.yukleniyor = true;
            sekme.render_ekran_olcegi = ekran_olcegi;
            RenderTalebi {
                sekme_id: id,
                nesil: sekme.render_nesli,
                dosya_yolu: sekme.dosya_yolu.clone(),
                sayfa_no: sekme.simdiki_sayfa,
                zum: sekme.zum,
                dpi_carpan: DPI_CARPANI,
                ekran_olcegi,
            }
        };
        let _ = self.talep_gonderici.send(IslemTalebi::Render(talep));
        cx.notify();
    }

    fn bekleyenleri_isle(&mut self, cx: &mut Context<Self>) -> bool {
        let hazir: Vec<usize> = self
            .belgeler
            .iter()
            .filter(|sekme| {
                sekme.bekleyen_render
                    && sekme
                        .son_zum_degisimi
                        .is_some_and(|zaman| zaman.elapsed() >= ZUM_BEKLEME)
            })
            .map(|sekme| sekme.id)
            .collect();
        for id in &hazir {
            self.render_talebi_gonder_id(*id, cx);
        }
        !hazir.is_empty()
    }

    fn sonuclari_isle(&mut self, cx: &mut Context<Self>) -> bool {
        let mut degisti = false;
        while let Ok(sonuc) = self.sonuc_alici.try_recv() {
            if self.tek_sonuc_uygula(sonuc, cx) {
                degisti = true;
            }
        }
        degisti
    }

    fn tek_sonuc_uygula(&mut self, sonuc: RenderSonuc, cx: &mut Context<Self>) -> bool {
        match sonuc {
            RenderSonuc::Basarili {
                sekme_id,
                nesil,
                sayfa_no,
                toplam_sayfa,
                orjinal_genislik,
                orjinal_yukseklik,
                goruntu,
            } => {
                let Some(sekme) = self.belge_mut(sekme_id) else {
                    return false;
                };
                if nesil != sekme.render_nesli {
                    return false;
                }
                let Some(render_goruntusu) = render_goruntusu(goruntu) else {
                    sekme.yukleniyor = false;
                    sekme.hata_mesaji = Some("Sayfa görüntüsü oluşturulamadı.".to_string());
                    return true;
                };
                sekme.toplam_sayfa = toplam_sayfa;
                sekme.simdiki_sayfa = sayfa_no;
                sekme.orjinal_genislik = orjinal_genislik;
                sekme.orjinal_yukseklik = orjinal_yukseklik;
                sekme.goruntu = Some(render_goruntusu);
                sekme.goruntu_sayfa = sayfa_no;
                sekme.goruntu_zum = sekme.zum;
                sekme.yukleniyor = false;
                sekme.hata_mesaji = None;
                self.sayfa_girdisi_guncelle = true;
                true
            }
            RenderSonuc::KucukResimHazir {
                sekme_id,
                sayfa_no,
                goruntu,
            } => {
                let Some(sekme) = self.belge_mut(sekme_id) else {
                    return false;
                };
                sekme.kucuk_resim_bekleyen.remove(&sayfa_no);
                if let Some(render_goruntusu) = render_goruntusu(goruntu) {
                    sekme.kucuk_resimler.insert(sayfa_no, render_goruntusu);
                    return true;
                }
                false
            }
            RenderSonuc::SayfaMetniHazir { sekme_id, metin } => {
                let Some(sekme) = self.belge_mut(sekme_id) else {
                    return false;
                };
                sekme.sayfa_metni = metin;
                sekme.metin_yukleniyor = false;
                true
            }
            RenderSonuc::AramaTamamlandi { sekme_id, sonuclar } => {
                let Some(sekme) = self.belge_mut(sekme_id) else {
                    return false;
                };
                sekme.arama_sonuclari = sonuclar;
                sekme.arama_yapiliyor = false;
                true
            }
            RenderSonuc::IcindekilerHazir {
                sekme_id,
                icindekiler,
            } => {
                let Some(sekme) = self.belge_mut(sekme_id) else {
                    return false;
                };
                sekme.icindekiler = icindekiler;
                sekme.icindekiler_yukleniyor = false;
                true
            }
            RenderSonuc::Hatali {
                sekme_id,
                nesil,
                mesaj,
            } => {
                if let Some(nesil) = nesil {
                    let Some(sekme) = self.belge_mut(sekme_id) else {
                        return false;
                    };
                    if nesil != sekme.render_nesli {
                        return false;
                    }
                    sekme.yukleniyor = false;
                    sekme.hata_mesaji = Some(mesaj);
                    return true;
                }
                self.bildir(mesaj, BildirimTuru::Hata, cx);
                true
            }
        }
    }

    pub fn kucuk_resim_iste(&mut self, sekme_id: usize, sayfa_no: u32) -> bool {
        let gonder = {
            let Some(sekme) = self.belge_mut(sekme_id) else {
                return false;
            };
            !sekme.kucuk_resimler.contains_key(&sayfa_no)
                && sekme.kucuk_resim_bekleyen.insert(sayfa_no)
        };
        if gonder {
            if let Some(sekme) = self.belge(sekme_id) {
                let _ = self.talep_gonderici.send(IslemTalebi::KucukResimRender {
                    sekme_id,
                    dosya_yolu: sekme.dosya_yolu.clone(),
                    sayfa_no,
                });
            }
        }
        gonder
    }

    pub fn girdi_odakta(&self, window: &Window, cx: &App) -> bool {
        self.sayfa_girdisi
            .read(cx)
            .focus_handle(cx)
            .is_focused(window)
            || self
                .arama_girdisi
                .read(cx)
                .focus_handle(cx)
                .is_focused(window)
    }

    pub fn eylem_dosya_ac(&mut self, _: &DosyaAc, window: &mut Window, cx: &mut Context<Self>) {
        self.dosya_sec(window, cx);
    }

    pub fn eylem_son_dosyalar(&mut self, _: &SonDosyalar, _: &mut Window, cx: &mut Context<Self>) {
        self.son_dosyalar_acik = !self.son_dosyalar_acik;
        self.hakkinda_acik = false;
        cx.notify();
    }

    pub fn eylem_sekme_kapat(
        &mut self,
        _: &SekmeKapat,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        self.aktif_sekmeyi_kapat(window, cx);
    }

    pub fn eylem_cikis(&mut self, _: &Cikis, _: &mut Window, cx: &mut Context<Self>) {
        cx.quit();
    }

    pub fn eylem_kenar_paneli(
        &mut self,
        _: &KenarPaneli,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        self.kenar_acik = !self.kenar_acik;
        if self.kenar_acik && self.kenar_modu == KenarModu::Arama {
            let odak = self.arama_girdisi.read(cx).focus_handle(cx);
            window.focus(&odak, cx);
        }
        cx.notify();
    }

    pub fn eylem_icindekiler(
        &mut self,
        _: &Icindekiler,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        self.kenar_modu_ayarla(KenarModu::Icindekiler, window, cx);
    }

    pub fn eylem_sayfalar(&mut self, _: &Sayfalar, window: &mut Window, cx: &mut Context<Self>) {
        self.kenar_modu_ayarla(KenarModu::KucukResimler, window, cx);
    }

    pub fn eylem_arama_paneli(
        &mut self,
        _: &AramaPaneli,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        if self.kenar_acik && self.kenar_modu == KenarModu::Arama {
            self.kenar_acik = false;
            window.focus(&self.odak, cx);
            cx.notify();
        } else {
            self.kenar_modu_ayarla(KenarModu::Arama, window, cx);
        }
    }

    pub fn eylem_metin_paneli(
        &mut self,
        _: &MetinPaneli,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        self.kenar_modu_ayarla(KenarModu::Metin, window, cx);
    }

    pub fn eylem_tema(&mut self, _: &TemaDegistir, _: &mut Window, cx: &mut Context<Self>) {
        self.tema_degistir(cx);
    }

    pub fn eylem_yakinlastir(&mut self, _: &Yakinlastir, _: &mut Window, cx: &mut Context<Self>) {
        if let Some(sekme) = self.aktif_belge() {
            let yeni = sekme.zum + 0.25;
            self.zum_ayarla(yeni, cx);
        }
    }

    pub fn eylem_uzaklastir(&mut self, _: &Uzaklastir, _: &mut Window, cx: &mut Context<Self>) {
        if let Some(sekme) = self.aktif_belge() {
            let yeni = sekme.zum - 0.25;
            self.zum_ayarla(yeni, cx);
        }
    }

    pub fn eylem_zum_sifirla(&mut self, _: &ZumSifirla, _: &mut Window, cx: &mut Context<Self>) {
        self.zum_sifirla(cx);
    }

    pub fn eylem_genislige_sigdir(
        &mut self,
        _: &GenisligeSigdir,
        _: &mut Window,
        cx: &mut Context<Self>,
    ) {
        self.sigdir(SigdirmaModu::Genislik, cx);
    }

    pub fn eylem_sayfaya_sigdir(
        &mut self,
        _: &SayfayaSigdir,
        _: &mut Window,
        cx: &mut Context<Self>,
    ) {
        self.sigdir(SigdirmaModu::Sayfa, cx);
    }

    pub fn eylem_onceki_sayfa(
        &mut self,
        _: &OncekiSayfa,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        if !self.girdi_odakta(window, cx) {
            self.sayfa_ileri(false, cx);
        }
    }

    pub fn eylem_sonraki_sayfa(
        &mut self,
        _: &SonrakiSayfa,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        if !self.girdi_odakta(window, cx) {
            self.sayfa_ileri(true, cx);
        }
    }

    pub fn eylem_ilk_sayfa(&mut self, _: &IlkSayfa, window: &mut Window, cx: &mut Context<Self>) {
        if !self.girdi_odakta(window, cx) {
            self.ilk_veya_son_sayfa(false, cx);
        }
    }

    pub fn eylem_son_sayfa(&mut self, _: &SonSayfa, window: &mut Window, cx: &mut Context<Self>) {
        if !self.girdi_odakta(window, cx) {
            self.ilk_veya_son_sayfa(true, cx);
        }
    }

    pub fn eylem_hakkinda(&mut self, _: &Hakkinda, _: &mut Window, cx: &mut Context<Self>) {
        self.hakkinda_acik = !self.hakkinda_acik;
        self.son_dosyalar_acik = false;
        cx.notify();
    }

    pub fn eylem_yeniden_dene(&mut self, _: &YenidenDene, _: &mut Window, cx: &mut Context<Self>) {
        if let Some(id) = self.aktif_id {
            if let Some(sekme) = self.belge_mut(id) {
                sekme.hata_mesaji = None;
                sekme.goruntu = None;
            }
            self.render_talebi_gonder_id(id, cx);
        }
    }

    pub fn eylem_kaplama_kapat(
        &mut self,
        _: &KaplamaKapat,
        _: &mut Window,
        cx: &mut Context<Self>,
    ) {
        if self.hakkinda_acik || self.son_dosyalar_acik {
            self.hakkinda_acik = false;
            self.son_dosyalar_acik = false;
            cx.notify();
        }
    }
}

pub fn pdf_mi(yol: &str) -> bool {
    Path::new(yol)
        .extension()
        .and_then(|uzanti| uzanti.to_str())
        .is_some_and(|uzanti| uzanti.eq_ignore_ascii_case("pdf"))
}

fn render_goruntusu(goruntu: SayfaGoruntusu) -> Option<Arc<RenderImage>> {
    let tampon = image::RgbaImage::from_raw(goruntu.genislik, goruntu.yukseklik, goruntu.rgba)?;
    Some(Arc::new(RenderImage::new(vec![image::Frame::new(tampon)])))
}

fn logo_yukle() -> Option<Arc<RenderImage>> {
    let resim = image::load_from_memory(include_bytes!("../assets/logo.png"))
        .ok()?
        .to_rgba8();
    let (genislik, yukseklik) = resim.dimensions();
    render_goruntusu(SayfaGoruntusu {
        genislik,
        yukseklik,
        rgba: resim.into_raw(),
    })
}

pub fn baslik_cubugu_secenekleri() -> WindowOptions {
    WindowOptions {
        window_min_size: Some(size(px(PENCERE_MIN_GENISLIK), px(PENCERE_MIN_YUKSEKLIK))),
        window_bounds: Some(WindowBounds::Maximized(Bounds::new(
            point(px(80.), px(80.)),
            size(px(PENCERE_GENISLIK), px(PENCERE_YUKSEKLIK)),
        ))),
        app_id: Some("spgroup_pdf_goruntuleyici".to_string()),
        ..TitleBar::window_options()
    }
}
