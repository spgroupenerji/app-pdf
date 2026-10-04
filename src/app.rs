use std::fs;
use std::path::{Path, PathBuf};

use crossbeam_channel::{Receiver, Sender};
use eframe::egui;
use egui_dock::{DockArea, DockState, Node, Style as DockStili, TabViewer};
use rfd::FileDialog;

use crate::isci::render_isci;
use crate::hakkinda::hakkinda_penceresi;
use crate::mesaj::{IslemTalebi, RenderSonuc, RenderTalebi};
use crate::sabitler::{DOKUNMA_HEDEFI, SURUM, FIRMA};
use crate::sekme::{PdfSekme, SigdirmaModu, SolPanelModu};
use crate::tema::tema_uygula;

const SON_DOSYALAR_AZAMI: usize = 10;
const DPI_CARPANI: f32 = 2.5; // 2.5x Ultra HD Supersampling Netliği

/// Yapılandırma ve son açılan dosyalar verisi
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
            let icerik = self.son_dosyalar.join("\n");
            let _ = fs::write(yol, icerik);
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

    // Tema tercihini ayarlar.txt dosyasina kaydeder.
    pub fn tema_kaydet(koyu: bool) {
        if let Some(yol) = Self::ayarlar_yolu() {
            if let Some(parent) = yol.parent() {
                let _ = fs::create_dir_all(parent);
            }
            let _ = fs::write(yol, format!("koyu_tema={}\n", if koyu { "true" } else { "false" }));
        }
    }

    // Kaydedilmis tema tercihini okur; yoksa varsayilan (aydinlik) doneer.
    pub fn tema_yukle() -> bool {
        if let Some(yol) = Self::ayarlar_yolu() {
            if let Ok(veri) = fs::read_to_string(&yol) {
                for satir in veri.lines() {
                    let temiz = satir.trim();
                    if let Some(deger) = temiz.strip_prefix("koyu_tema=") {
                        return deger == "true";
                    }
                }
            }
        }
        false
    }
}

/// Uygulamanin merkezi durumu; eframe::App trait'ini uygular.
pub struct GoruntuleyiciUygulama {
    pub dock_state: DockState<PdfSekme>,
    pub talep_gonderici: Sender<IslemTalebi>,
    pub sonuc_alici: Receiver<RenderSonuc>,
    pub sekme_sayaci: usize,
    pub ayarlar: UygulamaAyarlari,
    pub koyu_tema: bool,
    pub hakkinda_acok: bool,
    pub ekran_olcegi: f32,
    pub ilk_cerceve: bool,
}

impl GoruntuleyiciUygulama {
    /// Uygulamayi olusturur ve arka plan render iscisini baslatir.
    pub fn yeni() -> Self {
        let (talep_tx, talep_rx) = crossbeam_channel::unbounded();
        let (sonuc_tx, sonuc_rx) = crossbeam_channel::unbounded();

        // Arka plan render iscisini baslat
        std::thread::Builder::new()
            .name("mupdf-render-isci".to_string())
            .spawn(move || {
                render_isci(talep_rx, sonuc_tx);
            })
            .expect("Render isci is paracigi baslatilamadi");

        let ayarlar = UygulamaAyarlari::yukle();
        let koyu_tema = UygulamaAyarlari::tema_yukle();

        Self {
            dock_state: DockState::new(vec![]),
            talep_gonderici: talep_tx,
            sonuc_alici: sonuc_rx,
            sekme_sayaci: 0,
            ayarlar,
            koyu_tema,
            hakkinda_acok: false,
            ekran_olcegi: 1.0,
            ilk_cerceve: true,
        }
    }

    /// Yeni bir PDF belgesini yeni sekmede acar ve ilk sayfanin renderini talep eder.
    pub fn yeni_sekme_ac(&mut self, yol: &str) {
        self.ayarlar.dosya_ekle(yol);

        self.sekme_sayaci += 1;
        let baslik = Path::new(yol)
            .file_name()
            .and_then(|s| s.to_str())
            .unwrap_or("Belge")
            .to_string();
        let ekran_olcegi = self.ekran_olcegi;
        let sekme = PdfSekme::yeni(self.sekme_sayaci, baslik, yol.to_string(), ekran_olcegi);
        self.render_talebi_gonder(&sekme);
        self.icindekiler_talebi_gonder(&sekme);
        self.dock_state.main_surface_mut().push_to_focused_leaf(sekme);
    }

    /// Verilen sekme icin arka plan render iscisine render talebi gonderir.
    fn render_talebi_gonder(&self, sekme: &PdfSekme) {
        let talep = RenderTalebi {
            sekme_id: sekme.id,
            dosya_yolu: sekme.dosya_yolu.clone(),
            sayfa_no: sekme.simdiki_sayfa,
            zum: sekme.zum,
            dpi_carpan: DPI_CARPANI,
            ekran_olcegi: sekme.render_ekran_olcegi,
        };
        let _ = self.talep_gonderici.send(IslemTalebi::Render(talep));
    }

    /// İçindekiler listesini almak için talep gönderir
    fn icindekiler_talebi_gonder(&self, sekme: &PdfSekme) {
        let _ = self.talep_gonderici.send(IslemTalebi::IcindekilerAl {
            sekme_id: sekme.id,
            dosya_yolu: sekme.dosya_yolu.clone(),
        });
    }

    /// Arka plandan gelen render sonuclarini toplar ve ilgili sekmelere uygular.
    fn sonuclari_uygula(&mut self, ctx: &egui::Context) -> bool {
        let mut repaint_gerekli = false;
        while let Ok(sonuc) = self.sonuc_alici.try_recv() {
            self.tek_sonuc_uygula(sonuc, ctx);
            repaint_gerekli = true;
        }
        repaint_gerekli
    }

    /// Tek bir render sonucunu, id'sine gore ilgili sekmeye uygular.
    fn tek_sonuc_uygula(&mut self, sonuc: RenderSonuc, ctx: &egui::Context) {
        let hedef_id = match &sonuc {
            RenderSonuc::Basarili { sekme_id, .. }
            | RenderSonuc::KucukResimHazir { sekme_id, .. }
            | RenderSonuc::SayfaMetniHazir { sekme_id, .. }
            | RenderSonuc::AramaTamamlandi { sekme_id, .. }
            | RenderSonuc::IcindekilerHazir { sekme_id, .. }
            | RenderSonuc::Hatali { sekme_id, .. } => *sekme_id,
        };

        let agac = self.dock_state.main_surface_mut();
        'dis: for dugum in agac.iter_mut() {
            if let Node::Leaf(yaprak) = dugum {
                for sekme in yaprak.tabs_mut() {
                    if sekme.id != hedef_id {
                        continue;
                    }
                    match sonuc {
                        RenderSonuc::Basarili {
                            sayfa_no,
                            toplam_sayfa,
                            orjinal_genislik,
                            orjinal_yukseklik,
                            goruntu,
                            ..
                        } => {
                            sekme.toplam_sayfa = toplam_sayfa;
                            sekme.simdiki_sayfa = sayfa_no;
                            sekme.sayfa_girdi_metni = (sayfa_no + 1).to_string();
                            sekme.orjinal_genislik = orjinal_genislik;
                            sekme.orjinal_yukseklik = orjinal_yukseklik;
                            sekme.yukleniyor = false;
                            sekme.hata_mesaji = None;

                            if let Some(doku) = sekme.doku.as_mut() {
                                doku.set(goruntu, egui::TextureOptions {
                                    magnification: egui::TextureFilter::Linear,
                                    minification: egui::TextureFilter::Linear,
                                    wrap_mode: egui::TextureWrapMode::ClampToEdge,
                                    mipmap_mode: Some(egui::TextureFilter::Linear),
                                });
                            } else {
                                let yeni_doku = ctx.load_texture(
                                    format!("sekme_{}", hedef_id),
                                    goruntu,
                                    egui::TextureOptions {
                                        magnification: egui::TextureFilter::Linear,
                                        minification: egui::TextureFilter::Linear,
                                        wrap_mode: egui::TextureWrapMode::ClampToEdge,
                                        mipmap_mode: Some(egui::TextureFilter::Linear),
                                    },
                                );
                                sekme.doku = Some(yeni_doku);
                            }
                        }
                        RenderSonuc::KucukResimHazir {
                            sayfa_no, goruntu, ..
                        } => {
                            let kucuk_doku = ctx.load_texture(
                                format!("kr_{}_{}", hedef_id, sayfa_no),
                                goruntu,
                                egui::TextureOptions {
                                    magnification: egui::TextureFilter::Linear,
                                    minification: egui::TextureFilter::Linear,
                                    wrap_mode: egui::TextureWrapMode::ClampToEdge,
                                    mipmap_mode: Some(egui::TextureFilter::Linear),
                                },
                            );
                            sekme.kucuk_resimler.insert(sayfa_no, kucuk_doku);
                        }
                        RenderSonuc::SayfaMetniHazir { metin, .. } => {
                            sekme.sayfa_metni = metin;
                            sekme.metin_yukleniyor = false;
                        }
                        RenderSonuc::AramaTamamlandi { sonuclar, .. } => {
                            sekme.arama_sonuclari = sonuclar;
                            sekme.arama_yapiliyor = false;
                        }
                        RenderSonuc::IcindekilerHazir { icindekiler, .. } => {
                            sekme.icindekiler = icindekiler;
                        }
                        RenderSonuc::Hatali { mesaj, .. } => {
                            sekme.yukleniyor = false;
                            sekme.hata_mesaji = Some(mesaj);
                        }
                    }
                    break 'dis;
                }
            }
        }
    }

    /// Dosya acma diyalogunu gosterir ve secilen dosyayi/dosyalari yeni sekmede acar.
    fn dosya_ac_diyalogu(&mut self) {
        if let Some(dosyalar) = FileDialog::new()
            .add_filter("PDF Belgesi", &["pdf"])
            .set_title("PDF Belgesi veya Belgeleri Seç")
            .pick_files()
        {
            for yol in dosyalar {
                if let Some(yol_str) = yol.to_str() {
                    self.yeni_sekme_ac(yol_str);
                }
            }
        }
    }

    /// Aktif/Odaklanmış sekmeyi kapatır.
    fn aktif_sekmeyi_kapat(&mut self) {
        let main_surface = self.dock_state.main_surface_mut();
        if let Some(leaf_idx) = main_surface.focused_leaf() {
            if let Node::Leaf(leaf) = &mut main_surface[leaf_idx] {
                if !leaf.tabs.is_empty() {
                    let tab_idx = leaf.active;
                    let path = egui_dock::TabPath::new(
                        egui_dock::SurfaceIndex::main(),
                        leaf_idx,
                        tab_idx,
                    );
                    self.dock_state.remove_tab(path);
                }
            }
        }
    }
}

impl eframe::App for GoruntuleyiciUygulama {
    #[allow(deprecated)]
    fn ui(&mut self, ui: &mut egui::Ui, _cerceve: &mut eframe::Frame) {
        let ctx = ui.ctx().clone();

        // Ilk frame'de, pencere gorunur olduktan sonra maximize uygula.
        // (Gizli pencerede maximize Windows'ta sag kenarda bosluk birakiyordu.)
        if self.ilk_cerceve {
            self.ilk_cerceve = false;
            ctx.send_viewport_cmd(egui::ViewportCommand::Maximized(true));
        }

        self.ekran_olcegi = ctx.pixels_per_point();

        tema_uygula(&ctx, self.koyu_tema);
        hakkinda_penceresi(&ctx, &mut self.hakkinda_acok);

        // 1) Arka plandan gelen render sonuclarini topla
        if self.sonuclari_uygula(&ctx) {
            ctx.request_repaint();
        }

        // 2) Sürükle - Bırak (Drag & Drop) dosya desteği
        let birakilan_dosyalar = ctx.input(|i| i.raw.dropped_files.clone());
        if !birakilan_dosyalar.is_empty() {
            for dosya in birakilan_dosyalar {
                if let Some(yol) = dosya.path {
                    if yol.extension().and_then(|e| e.to_str()).map(|e| e.to_lowercase()) == Some("pdf".to_string()) {
                        if let Some(yol_str) = yol.to_str() {
                            self.yeni_sekme_ac(yol_str);
                        }
                    }
                }
            }
        }

        // 3) Global Klavye Kısayolları
        ctx.input(|giris| {
            if giris.modifiers.ctrl && giris.key_pressed(egui::Key::O) {
                self.dosya_ac_diyalogu();
            }
            if giris.modifiers.ctrl && giris.key_pressed(egui::Key::W) {
                self.aktif_sekmeyi_kapat();
            }
        });

        // 4) Ust menu cubugu
        egui::menu::bar(ui, |ui| {
            ui.menu_button("Dosya", |ui| {
                if ui.button("📂 Belge Aç... (Ctrl+O)").clicked() {
                    ui.close();
                    self.dosya_ac_diyalogu();
                }

                ui.menu_button("📄 Son Açılan Dosyalar", |ui| {
                    if self.ayarlar.son_dosyalar.is_empty() {
                        ui.label("Son açılan dosya yok");
                    } else {
                        let mut acilacak_yol = None;
                        for yol in &self.ayarlar.son_dosyalar {
                            let dosya_adi = Path::new(yol)
                                .file_name()
                                .and_then(|s| s.to_str())
                                .unwrap_or(yol);
                            if ui.button(dosya_adi).on_hover_text(yol).clicked() {
                                acilacak_yol = Some(yol.clone());
                                ui.close();
                            }
                        }
                        if let Some(yol) = acilacak_yol {
                            self.yeni_sekme_ac(&yol);
                        }
                    }
                });

                ui.separator();
                if ui.button("❌ Sekmeyi Kapat (Ctrl+W)").clicked() {
                    ui.close();
                    self.aktif_sekmeyi_kapat();
                }
                if ui.button("🚪 Çıkış").clicked() {
                    ctx.send_viewport_cmd(egui::ViewportCommand::Close);
                }
            });

            ui.menu_button("Görünüm", |ui| {
                let tema_etiket = if self.koyu_tema { "Koyu Tema" } else { "Açık Tema" };
                if ui.button(tema_etiket).clicked() {
                    self.koyu_tema = !self.koyu_tema;
                    UygulamaAyarlari::tema_kaydet(self.koyu_tema);
                    ui.close();
                }
            });

            ui.menu_button("Yardım", |ui| {
                ui.label(egui::RichText::new("Klavye Kısayolları:").strong());
                ui.label("• Ctrl + O : PDF Belgesi Aç");
                ui.label("• Ctrl + W : Aktif Sekmeyi Kapat");
                ui.label("• Ctrl + F : Metin Arama Paneli");
                ui.label("• Sol / Sağ Ok, PgUp / PgDn : Sayfa Değiştir");
                ui.label("• Home / End : İlk / Son Sayfaya Git");
                ui.label("• Ctrl + Tekerlek : Akıcı Yakınlaştır / Uzaklaştır");
                ui.label("• Ctrl + 0 : Zum %100 Sıfırla");
                ui.separator();
                ui.label("Rust + MuPDF + egui Güçlü PDF Görüntüleyici");
                ui.separator();
                if ui.button("ℹ Hakkında").clicked() {
                    self.hakkinda_acok = true;
                    ui.close();
                }
            });

            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                // Emoji yerine mevcut tema metni; dokunmatik uyumlu büyük buton.
                let tema_metin = if self.koyu_tema { "Koyu Tema" } else { "Açık Tema" };
                let tema_buton = egui::Button::new(tema_metin).min_size(egui::vec2(110.0, DOKUNMA_HEDEFI));
                if ui.add(tema_buton).on_hover_text("Tema Değiştir").clicked() {
                    self.koyu_tema = !self.koyu_tema;
                    UygulamaAyarlari::tema_kaydet(self.koyu_tema);
                }
            });
        });

        // 5) Merkez: Hosgeldin ekrani veya Sekmeli Dock Alani
        if self.dock_state.main_surface().num_tabs() == 0 {
            // Dikey/yatay responsive: içerik küçük ekranda kaydırılabilir, üst boşluk yüksekliğe orantılı.
            egui::ScrollArea::both()
                .auto_shrink([false, false])
                .show(ui, |ui| {
                    ui.vertical_centered(|ui| {
                        ui.add_space((ui.available_height() * 0.30).clamp(24.0, 96.0));
                        ui.heading(egui::RichText::new("Rust PDF Görüntüleyici").size(28.0).strong());
                        ui.add_space(12.0);
                        ui.label(egui::RichText::new("Hızlı, güvenli ve kararlı PDF okuma deneyimi").weak());
                        ui.add_space(24.0);

                        if ui.button(egui::RichText::new("📂  Belge Aç...").size(16.0)).clicked() {
                            self.dosya_ac_diyalogu();
                        }
                        ui.add_space(8.0);
                        ui.label(egui::RichText::new("veya PDF dosyalarını pencereye sürükleyip bırakın").weak());
                        ui.add_space(24.0);
                        ui.label(egui::RichText::new(format!("Sürüm {}", SURUM))
                            .size(11.0)
                            .color(ui.visuals().weak_text_color()));
                        ui.label(egui::RichText::new(FIRMA).size(11.0).color(ui.visuals().weak_text_color()));

                        if !self.ayarlar.son_dosyalar.is_empty() {
                            ui.add_space(36.0);
                            ui.label(egui::RichText::new("Son Açılan Belgeler").strong());
                            ui.add_space(10.0);

                            let mut acilacak_yol = None;
                            for yol in &self.ayarlar.son_dosyalar {
                                let dosya_adi = Path::new(yol)
                                    .file_name()
                                    .and_then(|s| s.to_str())
                                    .unwrap_or(yol);

                                if ui.button(format!("📄 {}", dosya_adi)).on_hover_text(yol).clicked() {
                                    acilacak_yol = Some(yol.clone());
                                }
                            }
                            if let Some(yol) = acilacak_yol {
                                self.yeni_sekme_ac(&yol);
                            }
                        }
                    });
                });
        } else {
            let mut tab_goruntuleyici = SekmeGoruntuleyici {
                talep_gonderici: &self.talep_gonderici,
                ekran_olcegi: self.ekran_olcegi,
            };
            // Sekme çubuğu da dokunmatik hedefle aynı yükseklikte olur.
            let mut dock_stili = DockStili::from_egui(ui.style());
            dock_stili.tab_bar.height = DOKUNMA_HEDEFI;
            DockArea::new(&mut self.dock_state)
                .style(dock_stili)
                .show_inside(ui, &mut tab_goruntuleyici);
        }
    }
}

/// Sekmelerin ekrana cizimini ve etkilesimlerini yonetir (egui_dock TabViewer).
struct SekmeGoruntuleyici<'a> {
    talep_gonderici: &'a Sender<IslemTalebi>,
    ekran_olcegi: f32,
}

impl<'a> SekmeGoruntuleyici<'a> {
    /// Sekme icin render talebini isci kanalina gonderir.
    fn render_talebi_gonder(&self, sekme: &PdfSekme) {
        let _ = self.talep_gonderici.send(IslemTalebi::Render(RenderTalebi {
            sekme_id: sekme.id,
            dosya_yolu: sekme.dosya_yolu.clone(),
            sayfa_no: sekme.simdiki_sayfa,
            zum: sekme.zum,
            dpi_carpan: DPI_CARPANI,
            ekran_olcegi: self.ekran_olcegi,
        }));
    }

    /// Küçük resim talebi gönderir
    fn kucuk_resim_talebi_gonder(&self, sekme: &PdfSekme, sayfa_no: u32) {
        let _ = self.talep_gonderici.send(IslemTalebi::KucukResimRender {
            sekme_id: sekme.id,
            dosya_yolu: sekme.dosya_yolu.clone(),
            sayfa_no,
        });
    }

    /// Sayfa metni talebi gönderir
    fn sayfa_metni_talebi_gonder(&self, sekme: &mut PdfSekme) {
        sekme.metin_yukleniyor = true;
        let _ = self.talep_gonderici.send(IslemTalebi::SayfaMetniAl {
            sekme_id: sekme.id,
            dosya_yolu: sekme.dosya_yolu.clone(),
            sayfa_no: sekme.simdiki_sayfa,
        });
    }

    /// Arama talebini gönderir
    fn arama_talebi_gonder(&self, sekme: &mut PdfSekme) {
        if !sekme.arama_metni.trim().is_empty() {
            sekme.arama_yapiliyor = true;
            let _ = self.talep_gonderici.send(IslemTalebi::MetinArama {
                sekme_id: sekme.id,
                dosya_yolu: sekme.dosya_yolu.clone(),
                aranan: sekme.arama_metni.clone(),
            });
        }
    }

    /// Zum seviyesini günceller.
    fn zum_ayarla(&self, sekme: &mut PdfSekme, yeni_zum: f32) {
        let sinirli_zum = yeni_zum.clamp(0.2, 20.0);
        if (sekme.zum - sinirli_zum).abs() > 0.001 {
            sekme.zum = sinirli_zum;
            sekme.sigdirma_modu = SigdirmaModu::Ozel;
            sekme.yukleniyor = true;
            self.render_talebi_gonder(sekme);
        }
    }

    /// Sayfa numarasını değiştirir.
    fn sayfa_degistir(&self, sekme: &mut PdfSekme, yeni_sayfa: u32) {
        if sekme.toplam_sayfa > 0 && yeni_sayfa < sekme.toplam_sayfa && yeni_sayfa != sekme.simdiki_sayfa {
            sekme.simdiki_sayfa = yeni_sayfa;
            sekme.sayfa_girdi_metni = (yeni_sayfa + 1).to_string();
            sekme.yukleniyor = true;
            self.render_talebi_gonder(sekme);
            if sekme.sol_panel_modu == SolPanelModu::MetinKopyala {
                self.sayfa_metni_talebi_gonder(sekme);
            }
        }
    }
}

impl<'a> TabViewer for SekmeGoruntuleyici<'a> {
    type Tab = PdfSekme;

    fn title(&mut self, sekme: &mut Self::Tab) -> egui::WidgetText {
        egui::WidgetText::from(sekme.baslik.clone())
    }

    fn ui(&mut self, ui: &mut egui::Ui, sekme: &mut Self::Tab) {
        egui::CentralPanel::default().show_inside(ui, |ui| {
            let mut render_gerekli = false;

        // Sekmeye özel Klavye Kısayolları
        ui.input(|i| {
            if i.key_pressed(egui::Key::ArrowLeft) || i.key_pressed(egui::Key::PageUp) {
                if sekme.simdiki_sayfa > 0 {
                    sekme.simdiki_sayfa -= 1;
                    sekme.sayfa_girdi_metni = (sekme.simdiki_sayfa + 1).to_string();
                    render_gerekli = true;
                }
            }
            if i.key_pressed(egui::Key::ArrowRight) || i.key_pressed(egui::Key::PageDown) {
                if sekme.toplam_sayfa > 0 && sekme.simdiki_sayfa + 1 < sekme.toplam_sayfa {
                    sekme.simdiki_sayfa += 1;
                    sekme.sayfa_girdi_metni = (sekme.simdiki_sayfa + 1).to_string();
                    render_gerekli = true;
                }
            }
            if i.key_pressed(egui::Key::Home) {
                if sekme.simdiki_sayfa != 0 {
                    sekme.simdiki_sayfa = 0;
                    sekme.sayfa_girdi_metni = "1".to_string();
                    render_gerekli = true;
                }
            }
            if i.key_pressed(egui::Key::End) {
                if sekme.toplam_sayfa > 0 && sekme.simdiki_sayfa + 1 != sekme.toplam_sayfa {
                    sekme.simdiki_sayfa = sekme.toplam_sayfa - 1;
                    sekme.sayfa_girdi_metni = sekme.toplam_sayfa.to_string();
                    render_gerekli = true;
                }
            }
            if i.modifiers.ctrl && i.key_pressed(egui::Key::F) {
                sekme.sol_panel_modu = if sekme.sol_panel_modu == SolPanelModu::Arama {
                    SolPanelModu::Kapali
                } else {
                    SolPanelModu::Arama
                };
            }
            if i.modifiers.ctrl && (i.key_pressed(egui::Key::Equals) || i.key_pressed(egui::Key::Plus)) {
                sekme.zum = (sekme.zum + 0.25).min(20.0);
                sekme.sigdirma_modu = SigdirmaModu::Ozel;
                render_gerekli = true;
            }
            if i.modifiers.ctrl && i.key_pressed(egui::Key::Minus) {
                sekme.zum = (sekme.zum - 0.25).max(0.2);
                sekme.sigdirma_modu = SigdirmaModu::Ozel;
                render_gerekli = true;
            }
            if i.modifiers.ctrl && i.key_pressed(egui::Key::Num0) {
                sekme.zum = 1.0;
                sekme.sigdirma_modu = SigdirmaModu::Ozel;
                render_gerekli = true;
            }
        });

        if render_gerekli {
            sekme.yukleniyor = true;
            self.render_talebi_gonder(sekme);
        }

        // --- UST KONTROL CUBUĞU ---
        // Sığdırma isteği burada toplanır; zum, araç çubuğu çizildikten sonra
        // kalan gerçek görünüm alanına göre uygulanır (dikey/yatay responsive).
        let mut sigdirma_istegi: Option<SigdirmaModu> = None;

        ui.horizontal_wrapped(|ui| {
            // Sol Yan Panel Butonları
            if ui
                .selectable_label(sekme.sol_panel_modu == SolPanelModu::Icindekiler, "📑 İçindekiler")
                .clicked()
            {
                sekme.sol_panel_modu = if sekme.sol_panel_modu == SolPanelModu::Icindekiler {
                    SolPanelModu::Kapali
                } else {
                    SolPanelModu::Icindekiler
                };
            }

            if ui
                .selectable_label(sekme.sol_panel_modu == SolPanelModu::KucukResimler, "🖼 Sayfalar")
                .clicked()
            {
                sekme.sol_panel_modu = if sekme.sol_panel_modu == SolPanelModu::KucukResimler {
                    SolPanelModu::Kapali
                } else {
                    SolPanelModu::KucukResimler
                };
            }

            if ui
                .selectable_label(sekme.sol_panel_modu == SolPanelModu::Arama, "🔍 Ara (Ctrl+F)")
                .clicked()
            {
                sekme.sol_panel_modu = if sekme.sol_panel_modu == SolPanelModu::Arama {
                    SolPanelModu::Kapali
                } else {
                    SolPanelModu::Arama
                };
            }

            if ui
                .selectable_label(sekme.sol_panel_modu == SolPanelModu::MetinKopyala, "📋 Metin Al")
                .clicked()
            {
                if sekme.sol_panel_modu == SolPanelModu::MetinKopyala {
                    sekme.sol_panel_modu = SolPanelModu::Kapali;
                } else {
                    sekme.sol_panel_modu = SolPanelModu::MetinKopyala;
                    self.sayfa_metni_talebi_gonder(sekme);
                }
            }

            ui.separator();

            // Sayfa Navigasyonu
            ui.add_enabled_ui(sekme.simdiki_sayfa > 0, |ui| {
                if ui.button("◀").on_hover_text("Önceki Sayfa").clicked() {
                    self.sayfa_degistir(sekme, sekme.simdiki_sayfa - 1);
                }
            });

            ui.label("Sayfa");
            let sayfa_girdi = ui.add_sized(
                egui::vec2(48.0, DOKUNMA_HEDEFI),
                egui::TextEdit::singleline(&mut sekme.sayfa_girdi_metni)
                    .horizontal_align(egui::Align::Center),
            );

            if sayfa_girdi.lost_focus() && ui.input(|i| i.key_pressed(egui::Key::Enter)) {
                if let Ok(girilen_sayfa) = sekme.sayfa_girdi_metni.trim().parse::<u32>() {
                    if girilen_sayfa >= 1 && girilen_sayfa <= sekme.toplam_sayfa {
                        self.sayfa_degistir(sekme, girilen_sayfa - 1);
                    } else {
                        sekme.sayfa_girdi_metni = (sekme.simdiki_sayfa + 1).to_string();
                    }
                } else {
                    sekme.sayfa_girdi_metni = (sekme.simdiki_sayfa + 1).to_string();
                }
            }

            ui.label(format!("/ {}", sekme.toplam_sayfa.max(1)));

            let son_sayfada = sekme.toplam_sayfa > 0 && sekme.simdiki_sayfa + 1 >= sekme.toplam_sayfa;
            ui.add_enabled_ui(!son_sayfada, |ui| {
                if ui.button("▶").on_hover_text("Sonraki Sayfa").clicked() {
                    self.sayfa_degistir(sekme, sekme.simdiki_sayfa + 1);
                }
            });

            ui.separator();

            // Zum Kontrolleri
            ui.add_enabled_ui(sekme.zum > 0.2, |ui| {
                if ui.button("➖").on_hover_text("Uzaklaştır").clicked() {
                    self.zum_ayarla(sekme, sekme.zum - 0.25);
                }
            });

            ui.label(format!("%{:.0}", sekme.zum * 100.0));

            ui.add_enabled_ui(sekme.zum < 20.0, |ui| {
                if ui.button("➕").on_hover_text("Yakınlaştır").clicked() {
                    self.zum_ayarla(sekme, sekme.zum + 0.25);
                }
            });

            ui.separator();

            // Sığdırma Butonları
            if ui
                .selectable_label(sekme.sigdirma_modu == SigdirmaModu::Genislik, "Genişliğe Sığdır")
                .clicked()
            {
                sigdirma_istegi = Some(SigdirmaModu::Genislik);
            }

            if ui
                .selectable_label(sekme.sigdirma_modu == SigdirmaModu::Sayfa, "Sayfaya Sığdır")
                .clicked()
            {
                sigdirma_istegi = Some(SigdirmaModu::Sayfa);
            }

            if sekme.zum != 1.0 && ui.button("100%").on_hover_text("Sıfırla").clicked() {
                self.zum_ayarla(sekme, 1.0);
            }
        });

        ui.separator();

        // Sığdırma zumunu, araç çubuğu sonrası kalan görünüm alanına göre hesapla.
        if let Some(mod_) = sigdirma_istegi {
            if sekme.orjinal_genislik > 0.0 {
                let alan = ui.available_size();
                let zum_genislik = (alan.x - 32.0) / sekme.orjinal_genislik;
                let zum = match mod_ {
                    SigdirmaModu::Sayfa if sekme.orjinal_yukseklik > 0.0 => {
                        zum_genislik.min((alan.y - 32.0) / sekme.orjinal_yukseklik)
                    }
                    _ => zum_genislik,
                };
                sekme.sigdirma_modu = mod_;
                self.zum_ayarla(sekme, zum);
            }
        }

        // --- SOL YAN PANEL & ANA GÖRÜNTÜ ALANI ---
        // Sol Yan Panel
        if sekme.sol_panel_modu != SolPanelModu::Kapali {
            egui::Panel::left(format!("sol_panel_{}", sekme.id))
                .resizable(true)
                .default_size(240.0)
                .show_inside(ui, |ui| {
                    match sekme.sol_panel_modu {
                        SolPanelModu::Icindekiler => {
                            ui.heading("📑 İçindekiler");
                            ui.separator();
                            egui::ScrollArea::vertical().show(ui, |ui| {
                                if sekme.icindekiler.is_empty() {
                                    ui.label("İçindekiler yükleniyor veya bulunamadı.");
                                } else {
                                    let mut git_sayfa = None;
                                    for oge in &sekme.icindekiler {
                                        if ui
                                            .button(format!("• {}", oge.baslik))
                                            .on_hover_text(format!("Sayfa {}", oge.sayfa_no + 1))
                                            .clicked()
                                        {
                                            git_sayfa = Some(oge.sayfa_no);
                                        }
                                    }
                                    if let Some(s_no) = git_sayfa {
                                        self.sayfa_degistir(sekme, s_no);
                                    }
                                }
                            });
                        }
                        SolPanelModu::KucukResimler => {
                            ui.heading("🖼 Sayfa Önizlemeleri");
                            ui.separator();
                            egui::ScrollArea::vertical().show(ui, |ui| {
                                let mut git_sayfa = None;
                                for s in 0..sekme.toplam_sayfa {
                                    ui.vertical_centered(|ui| {
                                        if let Some(kr_doku) = sekme.kucuk_resimler.get(&s) {
                                            let btn = ui.add(
                                                egui::Button::image((kr_doku.id(), egui::vec2(120.0, 160.0)))
                                                    .selected(s == sekme.simdiki_sayfa),
                                            );
                                            if btn.clicked() {
                                                git_sayfa = Some(s);
                                            }
                                        } else {
                                            if ui
                                                .button(format!("🖼 Sayfa {}", s + 1))
                                                .clicked()
                                            {
                                                git_sayfa = Some(s);
                                            }
                                            self.kucuk_resim_talebi_gonder(sekme, s);
                                        }
                                        ui.label(format!("{}", s + 1));
                                        ui.add_space(8.0);
                                    });
                                }
                                if let Some(s_no) = git_sayfa {
                                    self.sayfa_degistir(sekme, s_no);
                                }
                            });
                        }
                        SolPanelModu::MetinKopyala => {
                            ui.heading("📋 Sayfa Metni");
                            ui.separator();
                            if sekme.metin_yukleniyor {
                                ui.spinner();
                                ui.label("Metin çıkarılıyor...");
                            } else {
                                if ui.button("📋 Panoya Kopyala").clicked() {
                                    ui.ctx().copy_text(sekme.sayfa_metni.clone());
                                }
                                ui.separator();
                                egui::ScrollArea::vertical().show(ui, |ui| {
                                    ui.label(egui::RichText::new(&sekme.sayfa_metni).small());
                                });
                            }
                        }
                        SolPanelModu::Arama => {
                            ui.heading("🔍 Belgede Ara");
                            ui.separator();
                            // Dokunmatik uyumlu: tam genişlik, 44px yükseklik; düğme altında ayrı satırda.
                            let arama_edit = ui.add_sized(
                                egui::vec2(ui.available_width(), DOKUNMA_HEDEFI),
                                egui::TextEdit::singleline(&mut sekme.arama_metni)
                                    .hint_text("Metin girin..."),
                            );
                            if ui.button("Ara").clicked()
                                || (arama_edit.lost_focus()
                                    && ui.input(|i| i.key_pressed(egui::Key::Enter)))
                            {
                                self.arama_talebi_gonder(sekme);
                            }

                            ui.add_space(8.0);
                            egui::ScrollArea::vertical().show(ui, |ui| {
                                if sekme.arama_yapiliyor {
                                    ui.spinner();
                                    ui.label("Aranıyor...");
                                } else if sekme.arama_sonuclari.is_empty() {
                                    if !sekme.arama_metni.is_empty() {
                                        ui.label("Sonuç bulunamadı.");
                                    }
                                } else {
                                    ui.label(format!("{} sonuç bulundu:", sekme.arama_sonuclari.len()));
                                    ui.separator();
                                    let mut secilen_sayfa = None;
                                    for sonuc in &sekme.arama_sonuclari {
                                        ui.vertical(|ui| {
                                            if ui
                                                .button(egui::RichText::new(format!(
                                                    "Sayfa {}: {}",
                                                    sonuc.sayfa_no + 1,
                                                    sonuc.baglam
                                                )).small())
                                                .clicked()
                                            {
                                                secilen_sayfa = Some(sonuc.sayfa_no);
                                            }
                                        });
                                        ui.separator();
                                    }
                                    if let Some(sayfa_no) = secilen_sayfa {
                                        self.sayfa_degistir(sekme, sayfa_no);
                                    }
                                }
                            });
                        }
                        _ => {}
                    }
                });
        }

        // PDF Görünüm Alanı (Kalan Tüm Ekranı Kaplayan Ortalanmış ScrollArea)
        let _scroll_response = egui::ScrollArea::both()
            .auto_shrink([false, false])
            .show(ui, |ui| {
                if let Some(hata) = &sekme.hata_mesaji {
                    ui.vertical_centered(|ui| {
                        ui.add_space(60.0);
                        ui.colored_label(ui.visuals().error_fg_color, "⚠ Belge açılamadı");
                        ui.add_space(8.0);
                        ui.label(egui::RichText::new(hata).weak());
                    });
                } else if sekme.yukleniyor {
                    ui.vertical_centered(|ui| {
                        ui.add_space(60.0);
                        ui.spinner();
                        ui.label("Sayfa hazırlanıyor...");
                    });
                } else if let Some(doku) = &sekme.doku {
                    let mevcud_genislik = ui.available_width();
                    let gorunum_boyutu = egui::vec2(
                        sekme.orjinal_genislik * sekme.zum,
                        sekme.orjinal_yukseklik * sekme.zum,
                    );
                    let doku_genisligi = gorunum_boyutu.x;
                    if mevcud_genislik > doku_genisligi {
                        let sol_bosluk = (mevcud_genislik - doku_genisligi) / 2.0;
                        ui.horizontal(|ui| {
                            ui.add_space(sol_bosluk);
                            ui.image((doku.id(), gorunum_boyutu));
                        });
                    } else {
                        ui.image((doku.id(), gorunum_boyutu));
                    }
                } else {
                    ui.label("Görüntü bulunamadı.");
                }
            });

        // Fare Tekerleği ile Akıcı Zum (Ctrl + Scroll)
        let ctrl_basili = ui.input(|i| i.modifiers.ctrl);
        let scroll_delta = ui.input(|i| i.smooth_scroll_delta.y);
        if scroll_delta.abs() > 1.0 && ctrl_basili {
            let degisim = if scroll_delta > 0.0 { 0.1 } else { -0.1 };
            self.zum_ayarla(sekme, sekme.zum + degisim);
        }

        // Parmakla sıkıştır/aç (pinch-to-zoom) — dokunmatik cihazlar için.
        let zoom_delta = ui.input(|i| i.zoom_delta());
        if (zoom_delta - 1.0).abs() > 0.001 {
            self.zum_ayarla(sekme, sekme.zum * zoom_delta);
        }
        });
    }
}
