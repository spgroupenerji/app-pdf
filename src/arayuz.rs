use gpui_kit::assets::IconName as Ikon;
use gpui_kit::component::button::{Button, ButtonVariants};
use gpui_kit::component::input::Input;
use gpui_kit::component::menu::DropdownMenu;
use gpui_kit::component::scroll::ScrollableElement;
use gpui_kit::component::spinner::Spinner;
use gpui_kit::component::tab::{Tab, TabBar};
use gpui_kit::component::{
    h_resizable, resizable_panel, ActiveTheme, Disableable, Icon, Sizable, TitleBar,
};
use gpui_kit::prelude::*;
use gpui_kit::*;
use std::path::Path;

use crate::app::{
    AramaPaneli, Bildirim, BildirimTuru, Cikis, DosyaAc, GenisligeSigdir, GoruntuleyiciUygulama,
    Hakkinda, Icindekiler, KenarPaneli, MetinPaneli, Sayfalar, SayfayaSigdir, SekmeKapat,
    SonDosyalar, TemaDegistir, Uzaklastir, Yakinlastir, ZumSifirla,
};
use crate::sabitler::{DOKUNMA_HEDEFI, FIRMA, SURUM};
use crate::sekme::{sigdirma_hesapla, KenarModu, SigdirmaModu};
use crate::tema;

impl Render for GoruntuleyiciUygulama {
    fn render(&mut self, window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        self.ekran_olcegi = window.scale_factor();
        self.girdileri_esitle(window, cx);

        let mut kok = div()
            .id("uygulama_koku")
            .track_focus(&self.odak)
            .relative()
            .size_full()
            .flex()
            .flex_col()
            .bg(cx.theme().background)
            .text_color(cx.theme().foreground)
            .font_family(cx.theme().font_family.clone())
            .on_action(cx.listener(Self::eylem_dosya_ac))
            .on_action(cx.listener(Self::eylem_son_dosyalar))
            .on_action(cx.listener(Self::eylem_sekme_kapat))
            .on_action(cx.listener(Self::eylem_cikis))
            .on_action(cx.listener(Self::eylem_kenar_paneli))
            .on_action(cx.listener(Self::eylem_icindekiler))
            .on_action(cx.listener(Self::eylem_sayfalar))
            .on_action(cx.listener(Self::eylem_arama_paneli))
            .on_action(cx.listener(Self::eylem_metin_paneli))
            .on_action(cx.listener(Self::eylem_tema))
            .on_action(cx.listener(Self::eylem_yakinlastir))
            .on_action(cx.listener(Self::eylem_uzaklastir))
            .on_action(cx.listener(Self::eylem_zum_sifirla))
            .on_action(cx.listener(Self::eylem_genislige_sigdir))
            .on_action(cx.listener(Self::eylem_sayfaya_sigdir))
            .on_action(cx.listener(Self::eylem_onceki_sayfa))
            .on_action(cx.listener(Self::eylem_sonraki_sayfa))
            .on_action(cx.listener(Self::eylem_ilk_sayfa))
            .on_action(cx.listener(Self::eylem_son_sayfa))
            .on_action(cx.listener(Self::eylem_hakkinda))
            .on_action(cx.listener(Self::eylem_yeniden_dene))
            .on_action(cx.listener(Self::eylem_kaplama_kapat))
            .on_drop::<ExternalPaths>(cx.listener(Self::dosya_birakildi))
            .on_drag_move::<ExternalPaths>(cx.listener(Self::surukleme_hareketi))
            .on_file_drop_exit(cx.listener(Self::surukleme_bitti))
            .child(self.baslik_cubugu(cx))
            .child(self.arac_cubugu(cx));

        if !self.belgeler.is_empty() {
            kok = kok.child(self.sekme_cubugu(cx));
        }

        kok = kok
            .child(self.ana_icerik(window, cx))
            .child(self.durum_cubugu(cx));

        if self.surukleme_uzerinde {
            kok = kok.child(self.surukleme_kaplamasi(cx));
        }
        if let Some(bildirim) = &self.bildirim {
            kok = kok.child(bildirim_kaplamasi(bildirim));
        }
        if self.son_dosyalar_acik {
            kok = kok.child(self.son_dosyalar_penceresi(cx));
        }
        if self.hakkinda_acik {
            kok = kok.child(self.hakkinda_penceresi(cx));
        }

        kok
    }
}

impl GoruntuleyiciUygulama {
    fn girdileri_esitle(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        if self.sayfa_girdisi_guncelle {
            let metin = self
                .aktif_belge()
                .map(|belge| (belge.simdiki_sayfa + 1).to_string())
                .unwrap_or_else(|| "1".to_string());
            let odakta = self
                .sayfa_girdisi
                .read(cx)
                .focus_handle(cx)
                .is_focused(window);
            if !odakta {
                self.sayfa_girdisi_guncelle = false;
                self.sayfa_girdisi.update(cx, |state, cx| {
                    state.set_value(metin, window, cx);
                });
            }
        }
        if self.arama_girdisi_guncelle {
            let metin = self
                .aktif_belge()
                .map(|belge| belge.arama_metni.clone())
                .unwrap_or_default();
            let odakta = self
                .arama_girdisi
                .read(cx)
                .focus_handle(cx)
                .is_focused(window);
            if !odakta {
                self.arama_girdisi_guncelle = false;
                self.arama_girdisi.update(cx, |state, cx| {
                    state.set_value(metin, window, cx);
                });
            }
        }
    }

    fn baslik_cubugu(&self, cx: &mut Context<Self>) -> impl IntoElement {
        let koyu = self.koyu_tema;
        let tema_ikonu = if koyu { Ikon::Sun } else { Ikon::Moon };
        let tema_ipucu = if koyu {
            "Açık temaya geç"
        } else {
            "Koyu temaya geç"
        };

        let marka = div()
            .flex()
            .items_center()
            .gap_2()
            .pl_1()
            .child(
                div()
                    .size(px(24.))
                    .rounded(px(6.))
                    .overflow_hidden()
                    .bg(rgb(0xFFFFFF))
                    .border_1()
                    .border_color(cx.theme().border)
                    .when_some(self.logo.clone(), |el, logo| {
                        el.child(img(logo).size(px(24.)))
                    }),
            )
            .child(
                div()
                    .flex()
                    .flex_col()
                    .child(
                        div()
                            .flex()
                            .items_center()
                            .gap_1()
                            .child(
                                div()
                                    .child("SP GROUP")
                                    .text_size(px(11.))
                                    .font_weight(FontWeight::SEMIBOLD)
                                    .text_color(cx.theme().foreground),
                            )
                            .child(
                                div()
                                    .px(px(5.))
                                    .py(px(1.))
                                    .rounded(px(3.))
                                    .bg(cx.theme().primary.opacity(0.14))
                                    .text_size(px(8.))
                                    .font_weight(FontWeight::SEMIBOLD)
                                    .text_color(cx.theme().primary)
                                    .child("PDF"),
                            ),
                    )
                    .child(
                        div()
                            .child("Görüntüleyici")
                            .text_size(px(9.))
                            .text_color(cx.theme().muted_foreground),
                    ),
            );

        let menuler = div()
            .flex()
            .items_center()
            .gap_1()
            .child(
                Button::new("menu_dosya")
                    .label("Dosya")
                    .ghost()
                    .compact()
                    .dropdown_menu(|menu, _, _| {
                        menu.menu("Belge Aç…", Box::new(DosyaAc))
                            .menu("Son Açılan Belgeler", Box::new(SonDosyalar))
                            .separator()
                            .menu("Sekmeyi Kapat", Box::new(SekmeKapat))
                            .separator()
                            .menu("Çıkış", Box::new(Cikis))
                    }),
            )
            .child(
                Button::new("menu_gorunum")
                    .label("Görünüm")
                    .ghost()
                    .compact()
                    .dropdown_menu(|menu, _, _| {
                        menu.menu("Kenar Paneli", Box::new(KenarPaneli))
                            .menu("İçindekiler", Box::new(Icindekiler))
                            .menu("Sayfalar", Box::new(Sayfalar))
                            .menu("Arama", Box::new(AramaPaneli))
                            .menu("Metin", Box::new(MetinPaneli))
                            .separator()
                            .menu("Yakınlaştır", Box::new(Yakinlastir))
                            .menu("Uzaklaştır", Box::new(Uzaklastir))
                            .menu("Gerçek Boyut", Box::new(ZumSifirla))
                            .menu("Genişliğe Sığdır", Box::new(GenisligeSigdir))
                            .menu("Sayfaya Sığdır", Box::new(SayfayaSigdir))
                            .separator()
                            .menu("Temayı Değiştir", Box::new(TemaDegistir))
                    }),
            )
            .child(
                Button::new("menu_yardim")
                    .label("Yardım")
                    .ghost()
                    .compact()
                    .dropdown_menu(|menu, _, _| menu.menu("Hakkında", Box::new(Hakkinda))),
            );

        let sag = div()
            .flex()
            .items_center()
            .gap_1()
            .pr_1()
            .child(
                Button::new("tema_dugmesi")
                    .icon(Icon::new(tema_ikonu))
                    .ghost()
                    .compact()
                    .tooltip(tema_ipucu)
                    .accessibility_label("Temayı değiştir")
                    .on_click(cx.listener(|this, _, _, cx| this.tema_degistir(cx))),
            )
            .child(
                Button::new("hakkinda_dugmesi")
                    .icon(Icon::new(Ikon::Info))
                    .ghost()
                    .compact()
                    .tooltip("Hakkında")
                    .accessibility_label("Hakkında")
                    .on_click(cx.listener(|this, _, _, cx| {
                        this.hakkinda_acik = !this.hakkinda_acik;
                        this.son_dosyalar_acik = false;
                        cx.notify();
                    })),
            );

        TitleBar::new().child(marka).child(menuler).child(sag)
    }

    fn arac_cubugu(&self, cx: &mut Context<Self>) -> impl IntoElement {
        let belge_var = self.aktif_belge().is_some();
        let (simdiki, toplam, zum) = self
            .aktif_belge()
            .map(|belge| (belge.simdiki_sayfa, belge.toplam_sayfa, belge.zum))
            .unwrap_or((0, 0, 1.0));
        let sayfa_gecerli = belge_var && toplam > 0;

        let sol = div()
            .flex()
            .items_center()
            .gap_2()
            .child(
                Button::new("belge_ac")
                    .primary()
                    .icon(Icon::new(Ikon::FolderOpen))
                    .label("Belge Aç")
                    .tooltip("PDF belgesi aç (Ctrl+O)")
                    .on_click(cx.listener(|this, _, window, cx| this.dosya_sec(window, cx))),
            )
            .child(ayirici(cx))
            .child(
                Button::new("onceki_sayfa")
                    .icon(Icon::new(Ikon::ChevronLeft))
                    .ghost()
                    .compact()
                    .disabled(!sayfa_gecerli || simdiki == 0)
                    .tooltip("Önceki sayfa")
                    .accessibility_label("Önceki sayfa")
                    .on_click(cx.listener(|this, _, _, cx| this.sayfa_ileri(false, cx))),
            )
            .child(
                Input::new(&self.sayfa_girdisi)
                    .appearance(false)
                    .h(px(30.))
                    .w(px(48.)),
            )
            .child(
                div()
                    .text_size(px(11.))
                    .text_color(cx.theme().muted_foreground)
                    .child(format!("/ {}", toplam.max(1))),
            )
            .child(
                Button::new("sonraki_sayfa")
                    .icon(Icon::new(Ikon::ChevronRight))
                    .ghost()
                    .compact()
                    .disabled(!sayfa_gecerli || simdiki + 1 >= toplam)
                    .tooltip("Sonraki sayfa")
                    .accessibility_label("Sonraki sayfa")
                    .on_click(cx.listener(|this, _, _, cx| this.sayfa_ileri(true, cx))),
            )
            .child(ayirici(cx))
            .child(
                Button::new("uzaklastir")
                    .icon(Icon::new(Ikon::Minus))
                    .ghost()
                    .compact()
                    .disabled(!belge_var)
                    .tooltip("Uzaklaştır (Ctrl+-)")
                    .accessibility_label("Uzaklaştır")
                    .on_click(cx.listener(|this, _, _, cx| {
                        if let Some(sekme) = this.aktif_belge() {
                            let yeni = sekme.zum - 0.25;
                            this.zum_ayarla(yeni, cx);
                        }
                    })),
            )
            .child(
                Button::new("zum")
                    .label(format!("%{:.0}", zum * 100.0))
                    .ghost()
                    .compact()
                    .disabled(!belge_var)
                    .tooltip("Gerçek boyuta dön (Ctrl+0)")
                    .on_click(cx.listener(|this, _, _, cx| this.zum_sifirla(cx))),
            )
            .child(
                Button::new("yakinlastir")
                    .icon(Icon::new(Ikon::Plus))
                    .ghost()
                    .compact()
                    .disabled(!belge_var)
                    .tooltip("Yakınlaştır (Ctrl+=)")
                    .accessibility_label("Yakınlaştır")
                    .on_click(cx.listener(|this, _, _, cx| {
                        if let Some(sekme) = this.aktif_belge() {
                            let yeni = sekme.zum + 0.25;
                            this.zum_ayarla(yeni, cx);
                        }
                    })),
            )
            .child(
                Button::new("sigdir")
                    .icon(Icon::new(Ikon::Maximize))
                    .label("Sığdır")
                    .ghost()
                    .compact()
                    .disabled(!belge_var)
                    .dropdown_menu(|menu, _, _| {
                        menu.menu("Genişliğe Sığdır", Box::new(GenisligeSigdir))
                            .menu("Sayfaya Sığdır", Box::new(SayfayaSigdir))
                            .separator()
                            .menu("Gerçek Boyut", Box::new(ZumSifirla))
                    }),
            );

        let sag = div()
            .ml_auto()
            .flex()
            .items_center()
            .gap_1()
            .child(
                Button::new("arama_dugmesi")
                    .icon(Icon::new(Ikon::Search))
                    .ghost()
                    .compact()
                    .toggled(self.kenar_acik && self.kenar_modu == KenarModu::Arama)
                    .tooltip("Belgede ara (Ctrl+F)")
                    .accessibility_label("Belgede ara")
                    .on_click(cx.listener(|this, _, window, cx| {
                        if this.kenar_acik && this.kenar_modu == KenarModu::Arama {
                            this.kenar_acik = false;
                            window.focus(&this.odak, cx);
                            cx.notify();
                        } else {
                            this.kenar_modu_ayarla(KenarModu::Arama, window, cx);
                        }
                    })),
            )
            .child(
                Button::new("kenar_dugmesi")
                    .icon(Icon::new(if self.kenar_acik {
                        Ikon::PanelLeftClose
                    } else {
                        Ikon::PanelLeftOpen
                    }))
                    .ghost()
                    .compact()
                    .tooltip("Kenar panelini aç/kapat (Ctrl+B)")
                    .accessibility_label("Kenar panelini aç veya kapat")
                    .on_click(cx.listener(|this, _, window, cx| {
                        this.kenar_acik = !this.kenar_acik;
                        if !this.kenar_acik {
                            window.focus(&this.odak, cx);
                        }
                        cx.notify();
                    })),
            );

        div()
            .h(px(52.))
            .flex_shrink_0()
            .flex()
            .items_center()
            .gap_2()
            .px_3()
            .border_b_1()
            .border_color(cx.theme().border)
            .bg(cx.theme().background)
            .child(sol)
            .child(sag)
    }

    fn sekme_cubugu(&self, cx: &mut Context<Self>) -> impl IntoElement {
        let aktif_indeks = self
            .aktif_id
            .and_then(|id| self.belgeler.iter().position(|belge| belge.id == id))
            .unwrap_or(0);
        let mut sekmeler = TabBar::new("belge_sekmeleri")
            .folder()
            .selected_index(aktif_indeks)
            .max_width(px(230.));

        for belge in &self.belgeler {
            let id = belge.id;
            let baslik = belge.baslik.clone();
            let kapat_renk = cx.theme().muted_foreground;
            let kapat_hover_bg = cx.theme().secondary;
            let kapat_hover_fg = cx.theme().foreground;
            let kapat = div()
                .id(format!("sekme_kapat_{}", id))
                .flex()
                .items_center()
                .justify_center()
                .size(px(20.))
                .rounded(px(4.))
                .text_color(kapat_renk)
                .hover(move |stil| stil.bg(kapat_hover_bg).text_color(kapat_hover_fg))
                .cursor_pointer()
                .aria_label("Sekmeyi kapat")
                .on_click(cx.listener(move |this, _, window, cx| {
                    cx.stop_propagation();
                    this.sekme_kapat(id, window, cx);
                }))
                .child(
                    div()
                        .text_size(px(14.))
                        .font_weight(FontWeight::SEMIBOLD)
                        .child("×"),
                );
            let sekme = Tab::new()
                .label(baslik)
                .prefix(Icon::new(Ikon::FileText).with_size(px(14.)))
                .suffix(kapat)
                .on_click(cx.listener(move |this, _, window, cx| {
                    this.sekmeye_gec(id, window, cx);
                }));
            sekmeler = sekmeler.child(sekme);
        }

        let yeni = Button::new("yeni_sekme")
            .icon(Icon::new(Ikon::Plus))
            .ghost()
            .compact()
            .tooltip("Yeni belge aç")
            .accessibility_label("Yeni belge aç")
            .on_click(cx.listener(|this, _, window, cx| this.dosya_sec(window, cx)));

        div()
            .h(px(40.))
            .flex_shrink_0()
            .bg(cx.theme().tab_bar)
            .border_b_1()
            .border_color(cx.theme().border)
            .child(sekmeler.suffix(yeni))
    }

    fn ana_icerik(&mut self, _window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let Some(id) = self.aktif_id else {
            return self.bos_ekran(cx).into_any_element();
        };
        if self.belgeler.is_empty() {
            return self.bos_ekran(cx).into_any_element();
        }

        if self.kenar_acik {
            div()
                .flex_1()
                .min_h(px(0.))
                .child(
                    h_resizable("ana_govde")
                        .child(
                            resizable_panel()
                                .size(px(320.))
                                .size_range(px(260.)..px(460.))
                                .fixed()
                                .child(self.kenar_paneli(id, cx)),
                        )
                        .child(resizable_panel().child(self.belge_alani(id, cx))),
                )
                .into_any_element()
        } else {
            self.belge_alani(id, cx).into_any_element()
        }
    }

    fn belge_alani(&mut self, id: usize, cx: &mut Context<Self>) -> AnyElement {
        let otomatik_sigdir = self.belge(id).is_some_and(|belge| {
            belge.otomatik_sigdir
                && belge.goruntu.is_some()
                && belge.gorunum_alan.get().width.as_f32() > 100.0
        });
        if otomatik_sigdir {
            if let Some(belge) = self.belge_mut(id) {
                let alan = belge.gorunum_alan.get();
                if let Some(zum) = sigdirma_hesapla(
                    alan,
                    belge.orjinal_genislik,
                    belge.orjinal_yukseklik,
                    SigdirmaModu::Genislik,
                ) {
                    belge.zum = zum;
                    belge.sigdirma_modu = SigdirmaModu::Genislik;
                    belge.otomatik_sigdir = false;
                    belge.bekleyen_render = true;
                    belge.son_zum_degisimi = Some(std::time::Instant::now());
                    belge.render_nesli = belge.render_nesli.wrapping_add(1);
                    belge.yukleniyor = true;
                }
            }
            cx.notify();
        }

        let Some(belge) = self.belge(id) else {
            return div().into_any_element();
        };
        let goruntu = belge.goruntu.clone();
        let (genislik, yukseklik) = belge.gorunum_boyutu();
        let yukleniyor = belge.yukleniyor;
        let hata = belge.hata_mesaji.clone();
        let alan = belge.gorunum_alan.clone();
        let kaydirma = belge.kaydirma.clone();
        let koyu = self.koyu_tema;
        let tuval = tema::tuval_rengi(koyu);

        let icerik = if let Some(hata) = hata {
            div()
                .min_w(relative(1.0))
                .min_h(relative(1.0))
                .flex()
                .justify_center()
                .items_center()
                .p(px(28.))
                .child(hata_karti(hata, id, cx))
                .into_any_element()
        } else if let Some(goruntu) = goruntu {
            let sayfa = div()
                .bg(rgb(0xFFFFFF))
                .border_1()
                .border_color(cx.theme().border)
                .shadow(golge())
                .child(
                    img(goruntu)
                        .w(px(genislik.max(1.0)))
                        .h(px(yukseklik.max(1.0))),
                );
            div()
                .min_w(relative(1.0))
                .min_h(relative(1.0))
                .flex()
                .justify_center()
                .items_start()
                .p(px(28.))
                .child(sayfa)
                .into_any_element()
        } else {
            div()
                .min_w(relative(1.0))
                .min_h(relative(1.0))
                .flex()
                .justify_center()
                .items_start()
                .p(px(28.))
                .child(
                    div()
                        .w(px(genislik.max(320.0)))
                        .h(px(yukseklik.max(420.0)))
                        .rounded(px(4.))
                        .bg(cx.theme().skeleton)
                        .flex()
                        .items_center()
                        .justify_center()
                        .child(Spinner::new().color(cx.theme().primary)),
                )
                .into_any_element()
        };

        div()
            .flex_1()
            .size_full()
            .min_h(px(0.))
            .min_w(px(0.))
            .relative()
            .bg(tuval)
            .on_children_prepainted(move |sinirlar, _, _| {
                if let Some(sinir) = sinirlar.first() {
                    alan.set(sinir.size);
                }
            })
            .child(
                div()
                    .id("sayfa_kaydirma")
                    .size_full()
                    .overflow_scroll()
                    .track_scroll(&kaydirma)
                    .on_scroll(cx.listener(|this, olay: &ScrollEvent, _, cx| {
                        if olay.modifiers.control {
                            cx.stop_propagation();
                            let delta = olay.delta.pixel_delta(px(20.)).y.as_f32();
                            this.zum_carki(delta, cx);
                        }
                    }))
                    .on_pinch(cx.listener(|this, olay: &PinchEvent, _, cx| {
                        this.zum_pinch(olay.delta, cx);
                    }))
                    .on_mouse_down(
                        MouseButton::Left,
                        cx.listener(|this, _, window, cx| {
                            window.focus(&this.odak, cx);
                        }),
                    )
                    .child(icerik),
            )
            .when(
                yukleniyor && self.aktif_belge().is_some_and(|b| b.goruntu.is_some()),
                |el| {
                    el.child(
                        div()
                            .absolute()
                            .top_0()
                            .left_0()
                            .right_0()
                            .h(px(2.))
                            .bg(cx.theme().primary),
                    )
                },
            )
            .into_any_element()
    }

    fn bos_ekran(&self, cx: &mut Context<Self>) -> impl IntoElement {
        let mut icerik = div()
            .w_full()
            .max_w(px(540.))
            .flex()
            .flex_col()
            .items_center();

        icerik = icerik
            .child(
                div()
                    .size(px(88.))
                    .rounded_full()
                    .bg(cx.theme().primary.opacity(0.10))
                    .flex()
                    .items_center()
                    .justify_center()
                    .when_some(self.logo.clone(), |el, logo| {
                        el.child(img(logo).size(px(64.)).rounded(px(12.)))
                    }),
            )
            .child(div().h(px(24.)))
            .child(
                div()
                    .child("SP GROUP PDF Görüntüleyici")
                    .text_size(px(26.))
                    .font_weight(FontWeight::BOLD)
                    .text_color(cx.theme().foreground),
            )
            .child(
                div().h(px(8.)).child(
                    div()
                        .child("Hızlı, güvenli ve kararlı belge okuma deneyimi")
                        .text_size(px(13.))
                        .text_color(cx.theme().muted_foreground),
                ),
            )
            .child(div().h(px(26.)))
            .child(
                Button::new("bos_belge_ac")
                    .primary()
                    .icon(Icon::new(Ikon::FolderOpen))
                    .label("Belge Aç")
                    .h(px(DOKUNMA_HEDEFI))
                    .on_click(cx.listener(|this, _, window, cx| this.dosya_sec(window, cx))),
            )
            .child(div().h(px(10.)))
            .child(
                div()
                    .child("veya PDF dosyalarını pencereye sürükleyip bırakın")
                    .text_size(px(11.))
                    .text_color(cx.theme().muted_foreground),
            );

        if !self.ayarlar.son_dosyalar.is_empty() {
            icerik = icerik.child(div().h(px(32.))).child(
                div()
                    .w_full()
                    .flex()
                    .flex_col()
                    .child(
                        div()
                            .child("Son Açılan Belgeler")
                            .text_size(px(11.))
                            .font_weight(FontWeight::SEMIBOLD)
                            .text_color(cx.theme().muted_foreground),
                    )
                    .child(div().h(px(8.))),
            );
            let hover = cx.theme().secondary;
            for (konum, yol) in self.ayarlar.son_dosyalar.iter().enumerate() {
                let dosya_adi = Path::new(yol)
                    .file_name()
                    .and_then(|ad| ad.to_str())
                    .unwrap_or(yol)
                    .to_string();
                let klasor = Path::new(yol)
                    .parent()
                    .and_then(|dizin| dizin.to_str())
                    .unwrap_or("")
                    .to_string();
                let yol = yol.clone();
                icerik = icerik.child(
                    div()
                        .id(("son_dosya_karti", konum))
                        .w_full()
                        .flex()
                        .items_center()
                        .gap_2()
                        .px_3()
                        .py_2()
                        .rounded(px(8.))
                        .hover(move |stil| stil.bg(hover))
                        .cursor_pointer()
                        .on_click(cx.listener(move |this, _, window, cx| {
                            this.son_dosyalar_acik = false;
                            this.dosya_ac(&yol, window, cx);
                        }))
                        .child(
                            Icon::new(Ikon::FileText)
                                .with_size(px(18.))
                                .text_color(cx.theme().primary),
                        )
                        .child(
                            div()
                                .flex_1()
                                .min_w(px(0.))
                                .flex()
                                .flex_col()
                                .child(
                                    div()
                                        .truncate()
                                        .text_size(px(12.))
                                        .text_color(cx.theme().foreground)
                                        .child(dosya_adi),
                                )
                                .child(
                                    div()
                                        .truncate()
                                        .text_size(px(10.))
                                        .text_color(cx.theme().muted_foreground)
                                        .child(klasor),
                                ),
                        )
                        .child(
                            div()
                                .text_size(px(9.))
                                .text_color(cx.theme().muted_foreground)
                                .child("PDF"),
                        ),
                );
            }
        }

        icerik = icerik.child(div().h(px(36.))).child(
            div()
                .child(format!("Sürüm {}  •  {}", SURUM, FIRMA))
                .text_size(px(10.))
                .text_color(cx.theme().muted_foreground),
        );

        div().flex_1().min_h(px(0.)).overflow_y_scrollbar().child(
            div()
                .min_h(relative(1.0))
                .flex()
                .items_center()
                .justify_center()
                .p_8()
                .child(icerik),
        )
    }

    fn kenar_paneli(&self, id: usize, cx: &mut Context<Self>) -> impl IntoElement {
        let mod_ = self.kenar_modu;
        let mod_dugmesi =
            |etiket: &'static str, ikon: Ikon, hedef: KenarModu, cx: &mut Context<Self>| {
                let dugme = Button::new(format!("kenar_mod_{}", hedef.indeks()))
                    .label(etiket)
                    .icon(Icon::new(ikon))
                    .compact()
                    .flex_1();
                let dugme = if mod_ == hedef {
                    dugme.primary()
                } else {
                    dugme.ghost()
                };
                dugme.on_click(cx.listener(move |this, _, window, cx| {
                    this.kenar_modu_ayarla(hedef, window, cx);
                }))
            };

        let icerik = match mod_ {
            KenarModu::Icindekiler => self.icindekiler_paneli(id, cx),
            KenarModu::KucukResimler => self.kucuk_resim_paneli(id, cx),
            KenarModu::Arama => self.arama_paneli(cx),
            KenarModu::Metin => self.metin_paneli(id, cx),
        };

        div()
            .size_full()
            .flex()
            .flex_col()
            .bg(cx.theme().sidebar)
            .text_color(cx.theme().sidebar_foreground)
            .border_r_1()
            .border_color(cx.theme().sidebar_border)
            .child(
                div()
                    .flex_shrink_0()
                    .flex()
                    .flex_col()
                    .gap_1()
                    .p_2()
                    .border_b_1()
                    .border_color(cx.theme().sidebar_border)
                    .child(
                        div()
                            .flex()
                            .gap_1()
                            .child(mod_dugmesi(
                                "İçindekiler",
                                Ikon::BookOpen,
                                KenarModu::Icindekiler,
                                cx,
                            ))
                            .child(mod_dugmesi(
                                "Sayfalar",
                                Ikon::Images,
                                KenarModu::KucukResimler,
                                cx,
                            )),
                    )
                    .child(
                        div()
                            .flex()
                            .gap_1()
                            .child(mod_dugmesi("Ara", Ikon::Search, KenarModu::Arama, cx))
                            .child(mod_dugmesi("Metin", Ikon::FileText, KenarModu::Metin, cx)),
                    ),
            )
            .child(div().flex_1().min_h(px(0.)).child(icerik))
    }

    fn icindekiler_paneli(&self, id: usize, cx: &mut Context<Self>) -> AnyElement {
        let Some(belge) = self.belge(id) else {
            return div().into_any_element();
        };
        if belge.icindekiler_yukleniyor {
            return yukleniyor_paneli("İçindekiler yükleniyor…").into_any_element();
        }
        if belge.icindekiler.is_empty() {
            return bos_panel(
                Ikon::BookOpen,
                "İçindekiler yok",
                "Bu belge yer imi veya içindekiler tablosu içermiyor.",
                cx,
            )
            .into_any_element();
        }

        let adet = belge.icindekiler.len();
        let simdiki = belge.simdiki_sayfa;
        let aktif_renk = cx.theme().list_active;
        let hover_renk = cx.theme().list_hover;
        let metin_renk = cx.theme().foreground;
        let zayif_renk = cx.theme().muted_foreground;

        div()
            .size_full()
            .flex()
            .flex_col()
            .child(
                uniform_list(
                    "icindekiler_listesi",
                    adet,
                    cx.processor(move |this, aralik: std::ops::Range<usize>, _, cx| {
                        let Some(belge) = this.belge(id) else {
                            return Vec::new();
                        };
                        aralik
                            .map(|i| {
                                let oge = &belge.icindekiler[i];
                                let sayfa_no = oge.sayfa_no;
                                let secili = sayfa_no == simdiki;
                                let girinti = 10.0 + oge.seviye as f32 * 14.0;
                                div()
                                    .id(("icindekiler_satiri", i))
                                    .h(px(32.))
                                    .w_full()
                                    .flex()
                                    .items_center()
                                    .gap_2()
                                    .pl(px(girinti))
                                    .pr_2()
                                    .when(secili, |el| el.bg(aktif_renk))
                                    .hover(move |stil| stil.bg(hover_renk))
                                    .cursor_pointer()
                                    .aria_label(format!("{}: sayfa {}", oge.baslik, sayfa_no + 1))
                                    .child(
                                        Icon::new(Ikon::ChevronRight)
                                            .with_size(px(12.))
                                            .text_color(zayif_renk),
                                    )
                                    .child(
                                        div()
                                            .flex_1()
                                            .truncate()
                                            .text_size(px(12.))
                                            .text_color(metin_renk)
                                            .child(oge.baslik.clone()),
                                    )
                                    .child(
                                        div()
                                            .text_size(px(10.))
                                            .text_color(zayif_renk)
                                            .child(format!("{}", sayfa_no + 1)),
                                    )
                                    .on_click(cx.listener(move |this, _, _, cx| {
                                        this.sayfaya_git(id, sayfa_no, cx);
                                    }))
                                    .into_any_element()
                            })
                            .collect()
                    }),
                )
                .flex_1()
                .min_h(px(0.)),
            )
            .into_any_element()
    }

    fn kucuk_resim_paneli(&self, id: usize, cx: &mut Context<Self>) -> AnyElement {
        let Some(belge) = self.belge(id) else {
            return div().into_any_element();
        };
        if belge.toplam_sayfa == 0 {
            return yukleniyor_paneli("Sayfalar hazırlanıyor…").into_any_element();
        }
        let adet = belge.toplam_sayfa as usize;
        let simdiki = belge.simdiki_sayfa;
        let kenar = cx.theme().border;
        let secili_kenar = cx.theme().primary;
        let kagit = rgb(0xFFFFFF);
        let zayif_renk = cx.theme().muted_foreground;
        let metin_renk = cx.theme().foreground;

        div()
            .size_full()
            .flex()
            .flex_col()
            .child(
                uniform_list(
                    "kucuk_resim_listesi",
                    adet,
                    cx.processor(move |this, aralik: std::ops::Range<usize>, _, cx| {
                        aralik
                            .map(|i| {
                                let sayfa_no = i as u32;
                                let mevcut = this.belge(id).is_some_and(|belge| {
                                    belge.kucuk_resimler.contains_key(&sayfa_no)
                                });
                                if !mevcut {
                                    this.kucuk_resim_iste(id, sayfa_no);
                                }
                                let goruntu = this
                                    .belge(id)
                                    .and_then(|belge| belge.kucuk_resimler.get(&sayfa_no).cloned());
                                let secili = sayfa_no == simdiki;
                                div()
                                    .id(("kucuk_resim_satiri", i))
                                    .h(px(198.))
                                    .w_full()
                                    .flex()
                                    .flex_col()
                                    .items_center()
                                    .gap_1()
                                    .py_1()
                                    .cursor_pointer()
                                    .aria_label(format!("Sayfa {}", sayfa_no + 1))
                                    .on_click(cx.listener(move |this, _, _, cx| {
                                        this.sayfaya_git(id, sayfa_no, cx);
                                    }))
                                    .child(
                                        div()
                                            .p_1()
                                            .bg(kagit)
                                            .rounded(px(3.))
                                            .border_2()
                                            .border_color(if secili { secili_kenar } else { kenar })
                                            .when(goruntu.is_none(), |el| {
                                                el.child(
                                                    div()
                                                        .w(px(104.))
                                                        .h(px(140.))
                                                        .bg(zayif_renk.opacity(0.12)),
                                                )
                                            })
                                            .when_some(goruntu, |el, goruntu| {
                                                el.child(
                                                    img(goruntu)
                                                        .w(px(104.))
                                                        .h(px(140.))
                                                        .object_fit(ObjectFit::Contain),
                                                )
                                            }),
                                    )
                                    .child(
                                        div()
                                            .text_size(px(10.))
                                            .text_color(if secili {
                                                secili_kenar
                                            } else {
                                                metin_renk
                                            })
                                            .child(format!("{}", sayfa_no + 1)),
                                    )
                                    .into_any_element()
                            })
                            .collect()
                    }),
                )
                .flex_1()
                .min_h(px(0.)),
            )
            .into_any_element()
    }

    fn arama_paneli(&self, cx: &mut Context<Self>) -> AnyElement {
        let belge = self.aktif_belge();
        let arama_yapiliyor = belge.is_some_and(|b| b.arama_yapiliyor);
        let arama_metni = belge.map(|b| b.arama_metni.clone()).unwrap_or_default();
        let sonuclar = belge.map(|b| b.arama_sonuclari.clone()).unwrap_or_default();
        let id = self.aktif_id;

        let mut panel = div().size_full().flex().flex_col().p_2().gap_2().child(
            div()
                .flex()
                .gap_1()
                .child(
                    div()
                        .flex_1()
                        .min_w(px(0.))
                        .child(Input::new(&self.arama_girdisi).cleanable(true).h(px(34.))),
                )
                .child(
                    Button::new("arama_yap")
                        .primary()
                        .label("Ara")
                        .disabled(!self.aktif_belge().is_some())
                        .on_click(cx.listener(|this, _, _, cx| {
                            let metin = this.arama_girdisi.read(cx).value().to_string();
                            this.arama_yap_metin(metin, cx);
                        })),
                ),
        );

        if arama_yapiliyor {
            panel = panel.child(yukleniyor_paneli("Aranıyor…"));
        } else if arama_metni.trim().is_empty() {
            panel = panel.child(bos_panel(
                Ikon::Search,
                "Belgede arayın",
                "Metni yazıp Enter'a basın; sonuçlar sayfa numarasıyla listelenir.",
                cx,
            ));
        } else if sonuclar.is_empty() {
            panel = panel.child(bos_panel(
                Ikon::Search,
                "Sonuç bulunamadı",
                "Farklı bir kelime veya ifade deneyin.",
                cx,
            ));
        } else {
            panel = panel.child(
                div()
                    .text_size(px(10.))
                    .text_color(cx.theme().muted_foreground)
                    .child(format!("{} sonuç", sonuclar.len())),
            );
            let adet = sonuclar.len();
            let metin_renk = cx.theme().foreground;
            let zayif_renk = cx.theme().muted_foreground;
            let vurgu = cx.theme().primary;
            let hover_renk = cx.theme().list_hover;
            panel = panel.child(
                div().flex_1().min_h(px(0.)).flex().flex_col().child(
                    uniform_list(
                        "arama_sonuclari",
                        adet,
                        cx.processor(move |this, aralik: std::ops::Range<usize>, _, cx| {
                            let Some(belge) = this.belge(id.unwrap_or(0)) else {
                                return Vec::new();
                            };
                            aralik
                                .map(|i| {
                                    let sonuc = &belge.arama_sonuclari[i];
                                    let sayfa_no = sonuc.sayfa_no;
                                    div()
                                        .id(("arama_satiri", i))
                                        .h(px(62.))
                                        .w_full()
                                        .flex()
                                        .flex_col()
                                        .justify_center()
                                        .gap_1()
                                        .px_2()
                                        .rounded(px(6.))
                                        .hover(move |stil| stil.bg(hover_renk))
                                        .cursor_pointer()
                                        .aria_label(format!(
                                            "Sayfa {}: {}",
                                            sayfa_no + 1,
                                            sonuc.baglam
                                        ))
                                        .on_click(cx.listener(move |this, _, _, cx| {
                                            this.sayfaya_git(id.unwrap_or(0), sayfa_no, cx);
                                        }))
                                        .child(
                                            div()
                                                .text_size(px(10.))
                                                .font_weight(FontWeight::SEMIBOLD)
                                                .text_color(vurgu)
                                                .child(format!("Sayfa {}", sayfa_no + 1)),
                                        )
                                        .child(
                                            div()
                                                .truncate()
                                                .text_size(px(11.))
                                                .text_color(metin_renk)
                                                .child(sonuc.baglam.clone()),
                                        )
                                        .child(
                                            div()
                                                .text_size(px(9.))
                                                .text_color(zayif_renk)
                                                .child("Sonuca git"),
                                        )
                                        .into_any_element()
                                })
                                .collect()
                        }),
                    )
                    .flex_1()
                    .min_h(px(0.)),
                ),
            );
        }

        panel.into_any_element()
    }

    fn metin_paneli(&self, id: usize, cx: &mut Context<Self>) -> AnyElement {
        let Some(belge) = self.belge(id) else {
            return div().into_any_element();
        };
        if belge.metin_yukleniyor {
            return yukleniyor_paneli("Metin çıkarılıyor…").into_any_element();
        }
        if belge.sayfa_metni.trim().is_empty() {
            return bos_panel(
                Ikon::FileText,
                "Metin bulunamadı",
                "Bu sayfa seçilebilir metin içermiyor olabilir.",
                cx,
            )
            .into_any_element();
        }

        div()
            .size_full()
            .flex()
            .flex_col()
            .child(
                div()
                    .flex_shrink_0()
                    .p_2()
                    .border_b_1()
                    .border_color(cx.theme().sidebar_border)
                    .child(
                        Button::new("metni_kopyala")
                            .label("Panoya Kopyala")
                            .icon(Icon::new(Ikon::Copy))
                            .ghost()
                            .compact()
                            .on_click(cx.listener(|this, _, _, cx| this.metni_kopyala(cx))),
                    ),
            )
            .child(
                div()
                    .flex_1()
                    .min_h(px(0.))
                    .overflow_y_scrollbar()
                    .p_3()
                    .child(
                        div()
                            .whitespace_normal()
                            .text_size(px(12.))
                            .text_color(cx.theme().foreground)
                            .child(belge.sayfa_metni.clone()),
                    ),
            )
            .into_any_element()
    }

    fn durum_cubugu(&self, cx: &mut Context<Self>) -> impl IntoElement {
        let (durum, durum_rengi) = match self.aktif_belge() {
            Some(belge) if belge.hata_mesaji.is_some() => ("Hata", cx.theme().danger),
            Some(belge) if belge.yukleniyor => ("İşleniyor", cx.theme().warning),
            Some(_) => ("Hazır", cx.theme().success),
            None => ("Belge yok", cx.theme().muted_foreground),
        };
        let dosya = self
            .aktif_belge()
            .map(|belge| belge.dosya_yolu.clone())
            .unwrap_or_else(|| "SP GROUP PDF Görüntüleyici".to_string());
        let (simdiki, toplam, zum) = self
            .aktif_belge()
            .map(|belge| (belge.simdiki_sayfa, belge.toplam_sayfa, belge.zum))
            .unwrap_or((0, 0, 1.0));

        div()
            .h(px(28.))
            .flex_shrink_0()
            .flex()
            .items_center()
            .gap_2()
            .px_3()
            .border_t_1()
            .border_color(cx.theme().border)
            .bg(cx.theme().background)
            .text_size(px(10.))
            .text_color(cx.theme().muted_foreground)
            .child(div().flex_1().min_w(px(0.)).truncate().child(dosya))
            .child(
                div()
                    .px(px(6.))
                    .py(px(1.))
                    .rounded(px(3.))
                    .bg(cx.theme().primary.opacity(0.12))
                    .text_color(cx.theme().primary)
                    .child("GPU"),
            )
            .child(
                div()
                    .flex()
                    .items_center()
                    .gap_1()
                    .child(div().size(px(6.)).rounded_full().bg(durum_rengi))
                    .child(durum),
            )
            .child(div().child(format!("Sayfa {}/{}", simdiki + 1, toplam.max(1))))
            .child(div().child(format!("%{:.0}", zum * 100.0)))
    }

    fn surukleme_kaplamasi(&self, cx: &mut Context<Self>) -> impl IntoElement {
        let vurgu = cx.theme().primary;
        div()
            .absolute()
            .top_0()
            .left_0()
            .size_full()
            .bg(vurgu.opacity(0.08))
            .border_2()
            .border_color(vurgu)
            .flex()
            .items_center()
            .justify_center()
            .child(
                div()
                    .flex()
                    .flex_col()
                    .items_center()
                    .gap_3()
                    .p_6()
                    .rounded(px(12.))
                    .bg(cx.theme().popover)
                    .shadow(golge())
                    .child(
                        Icon::new(Ikon::FolderOpen)
                            .with_size(px(36.))
                            .text_color(vurgu),
                    )
                    .child(
                        div()
                            .text_size(px(15.))
                            .font_weight(FontWeight::SEMIBOLD)
                            .child("PDF belgelerini bırakın"),
                    )
                    .child(
                        div()
                            .text_size(px(11.))
                            .text_color(cx.theme().muted_foreground)
                            .child("Belgeler yeni sekmelerde açılır."),
                    ),
            )
    }

    fn son_dosyalar_penceresi(&self, cx: &mut Context<Self>) -> AnyElement {
        let hover = cx.theme().secondary;
        let mut liste = div()
            .flex()
            .flex_col()
            .gap_1()
            .max_h(px(380.))
            .overflow_y_scrollbar();
        if self.ayarlar.son_dosyalar.is_empty() {
            liste = liste.child(
                div()
                    .py_6()
                    .text_size(px(12.))
                    .text_color(cx.theme().muted_foreground)
                    .child("Henüz açılmış bir belge yok."),
            );
        } else {
            for (konum, yol) in self.ayarlar.son_dosyalar.iter().enumerate() {
                let dosya_adi = Path::new(yol)
                    .file_name()
                    .and_then(|ad| ad.to_str())
                    .unwrap_or(yol)
                    .to_string();
                let yol_ekran = yol.clone();
                let yol_eylem = yol.clone();
                liste = liste.child(
                    div()
                        .id(("son_dosya_pencere", konum))
                        .flex()
                        .items_center()
                        .gap_2()
                        .px_3()
                        .py_2()
                        .rounded(px(8.))
                        .hover(move |stil| stil.bg(hover))
                        .cursor_pointer()
                        .on_click(cx.listener(move |this, _, window, cx| {
                            this.son_dosyalar_acik = false;
                            this.dosya_ac(&yol_eylem, window, cx);
                        }))
                        .child(
                            Icon::new(Ikon::FileText)
                                .with_size(px(18.))
                                .text_color(cx.theme().primary),
                        )
                        .child(
                            div()
                                .flex_1()
                                .min_w(px(0.))
                                .flex()
                                .flex_col()
                                .child(div().truncate().text_size(px(12.)).child(dosya_adi))
                                .child(
                                    div()
                                        .truncate()
                                        .text_size(px(10.))
                                        .text_color(cx.theme().muted_foreground)
                                        .child(yol_ekran),
                                ),
                        ),
                );
            }
        }

        div()
            .absolute()
            .top_0()
            .left_0()
            .size_full()
            .bg(hsla(0., 0., 0., 0.45))
            .flex()
            .items_center()
            .justify_center()
            .on_mouse_down(
                MouseButton::Left,
                cx.listener(|this, _, _, cx| {
                    this.son_dosyalar_acik = false;
                    cx.notify();
                }),
            )
            .child(
                div()
                    .w(px(560.))
                    .max_w(relative(0.92))
                    .p_5()
                    .rounded(px(12.))
                    .bg(cx.theme().popover)
                    .border_1()
                    .border_color(cx.theme().border)
                    .shadow(golge())
                    .on_mouse_down(MouseButton::Left, |_, _, cx| cx.stop_propagation())
                    .child(
                        div()
                            .flex()
                            .items_center()
                            .justify_between()
                            .child(
                                div()
                                    .text_size(px(16.))
                                    .font_weight(FontWeight::SEMIBOLD)
                                    .child("Son Açılan Belgeler"),
                            )
                            .child(
                                Button::new("son_dosya_kapat")
                                    .icon(Icon::new(Ikon::X))
                                    .ghost()
                                    .compact()
                                    .on_click(cx.listener(|this, _, _, cx| {
                                        this.son_dosyalar_acik = false;
                                        cx.notify();
                                    })),
                            ),
                    )
                    .child(div().h(px(12.)))
                    .child(liste),
            )
            .into_any_element()
    }

    fn dosya_birakildi(
        &mut self,
        yollar: &ExternalPaths,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        self.surukleme_uzerinde = false;
        for yol in yollar.paths().iter() {
            if let Some(yol) = yol.to_str() {
                self.dosya_ac(yol, window, cx);
            }
        }
        cx.notify();
    }

    fn surukleme_hareketi(
        &mut self,
        _: &DragMoveEvent<ExternalPaths>,
        _: &mut Window,
        cx: &mut Context<Self>,
    ) {
        if !self.surukleme_uzerinde {
            self.surukleme_uzerinde = true;
            cx.notify();
        }
    }

    fn surukleme_bitti(&mut self, _: &FileDropEvent, _: &mut Window, cx: &mut Context<Self>) {
        if self.surukleme_uzerinde {
            self.surukleme_uzerinde = false;
            cx.notify();
        }
    }
}

fn ayirici(cx: &App) -> impl IntoElement {
    div().h(px(24.)).w(px(1.)).bg(cx.theme().border)
}

pub(crate) fn golge() -> Vec<BoxShadow> {
    vec![BoxShadow {
        color: hsla(0., 0., 0., 0.35),
        offset: point(px(0.), px(10.)),
        blur_radius: px(30.),
        spread_radius: px(0.),
        inset: false,
    }]
}

fn yukleniyor_paneli(mesaj: &'static str) -> impl IntoElement {
    div()
        .size_full()
        .flex()
        .flex_col()
        .items_center()
        .justify_center()
        .gap_2()
        .child(Spinner::new())
        .child(div().text_size(px(11.)).child(mesaj))
}

fn bos_panel(
    ikon: Ikon,
    baslik: &'static str,
    aciklama: &'static str,
    cx: &App,
) -> impl IntoElement {
    div()
        .size_full()
        .flex()
        .flex_col()
        .items_center()
        .justify_center()
        .gap_2()
        .p_4()
        .child(
            Icon::new(ikon)
                .with_size(px(28.))
                .text_color(cx.theme().muted_foreground),
        )
        .child(
            div()
                .text_size(px(12.))
                .font_weight(FontWeight::SEMIBOLD)
                .child(baslik),
        )
        .child(
            div()
                .text_size(px(10.))
                .text_color(cx.theme().muted_foreground)
                .text_center()
                .child(aciklama),
        )
}

fn hata_karti(
    hata: String,
    id: usize,
    cx: &mut Context<GoruntuleyiciUygulama>,
) -> impl IntoElement {
    div()
        .flex()
        .flex_col()
        .items_center()
        .gap_3()
        .max_w(px(420.))
        .p_5()
        .rounded(px(12.))
        .bg(cx.theme().popover)
        .border_1()
        .border_color(cx.theme().border)
        .shadow(golge())
        .child(
            Icon::new(Ikon::TriangleAlert)
                .with_size(px(30.))
                .text_color(cx.theme().danger),
        )
        .child(
            div()
                .text_size(px(14.))
                .font_weight(FontWeight::SEMIBOLD)
                .child("Belge açılamadı"),
        )
        .child(
            div()
                .text_size(px(11.))
                .text_color(cx.theme().muted_foreground)
                .text_center()
                .child(hata),
        )
        .child(
            Button::new("yeniden_dene_kart")
                .label("Yeniden Dene")
                .icon(Icon::new(Ikon::RefreshCw))
                .on_click(cx.listener(move |this, _, _, cx| {
                    this.render_talebi_gonder_id(id, cx);
                })),
        )
}

fn bildirim_kaplamasi(bildirim: &Bildirim) -> impl IntoElement {
    let (ikon, renk) = match bildirim.tur {
        BildirimTuru::Hata => (Ikon::TriangleAlert, 0xF08078),
        BildirimTuru::Basari => (Ikon::CircleCheck, 0x5BC49A),
    };
    div()
        .absolute()
        .bottom(px(44.))
        .right(px(16.))
        .w(px(360.))
        .max_w(relative(0.9))
        .flex()
        .items_center()
        .gap_2()
        .p_3()
        .rounded(px(10.))
        .bg(rgb(0x1F242E))
        .border_1()
        .border_color(rgba(0xFFFFFF1A))
        .shadow(golge())
        .child(Icon::new(ikon).with_size(px(18.)).text_color(rgb(renk)))
        .child(
            div()
                .flex_1()
                .text_size(px(11.))
                .text_color(rgb(0xE7EAF0))
                .child(bildirim.mesaj.clone()),
        )
}
