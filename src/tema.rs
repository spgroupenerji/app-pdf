// Koyu/açık tema paletleri ve dokunmatik stil uygulayıcısı.
// Tek Palet yapısı her iki modu tanımlar; tüm widget durumları ve metin
// katmanları tek yerden türetilir, böylece modlar arası uyum garantidir.
use egui::{Color32, CornerRadius, Stroke, Visuals};
use crate::sabitler::DOKUNMA_HEDEFI;

struct Palet {
    /// Ana panel/araç çubuğu zemini.
    zemin: Color32,
    /// Pencere, menü ve açılır kutu zemini.
    pencere: Color32,
    /// En koyu/açık zemin: kaydırma alanları, sekme çubuğu.
    extreme: Color32,
    /// Hafif farklı zemin: çizelge şeritleri, pasif düğmeler.
    faint: Color32,
    /// Birincil metin.
    metin: Color32,
    /// İkincil/yardımcı metin (her iki zeminde de okunur kontrastta).
    metin_zayif: Color32,
    /// Kurumsal vurgu (bağlantı, seçim, etkin kenarlık).
    vurgu: Color32,
    /// Üzerine gelinen düğme zemini.
    hover: Color32,
    /// Basılı/etkin düğme zemini.
    aktif: Color32,
    /// Ayraç ve pencere kenarlığı.
    ayirici: Color32,
    /// Metin seçimi zemini.
    secim: Color32,
    /// Hata metni (koyu zeminde açık, açık zeminde koyu ton).
    hata: Color32,
    /// Uyarı metni.
    uyari: Color32,
    /// Kod/metin düzenleme alanı zemini.
    girinti: Color32,
}

const KOYU: Palet = Palet {
    // Lacivert ile gri arasında kurumsal grafit; saf siyah değil.
    zemin: Color32::from_rgb(0x1E, 0x21, 0x28),
    pencere: Color32::from_rgb(0x26, 0x2A, 0x33),
    extreme: Color32::from_rgb(0x17, 0x1A, 0x20),
    faint: Color32::from_rgb(0x2A, 0x2E, 0x38),
    metin: Color32::from_rgb(0xE3, 0xE6, 0xEC),
    metin_zayif: Color32::from_rgb(0x9A, 0xA3, 0xB0),
    vurgu: Color32::from_rgb(0x5B, 0x9B, 0xD5),
    hover: Color32::from_rgb(0x33, 0x38, 0x45),
    aktif: Color32::from_rgb(0x3D, 0x44, 0x52),
    ayirici: Color32::from_rgb(0x3A, 0x41, 0x50),
    secim: Color32::from_rgb(0x34, 0x52, 0x7A),
    hata: Color32::from_rgb(0xF2, 0x8B, 0x82),
    uyari: Color32::from_rgb(0xFD, 0xD6, 0x63),
    girinti: Color32::from_rgb(0x1A, 0x1D, 0x24),
};

const ACIK: Palet = Palet {
    // Koyu paletin aynı kurumsal ailesinin aydınlık karşılığı.
    zemin: Color32::from_rgb(0xF7, 0xF8, 0xFA),
    pencere: Color32::WHITE,
    extreme: Color32::from_rgb(0xEC, 0xEE, 0xF1),
    faint: Color32::from_rgb(0xEE, 0xF0, 0xF3),
    metin: Color32::from_rgb(0x23, 0x27, 0x2E),
    metin_zayif: Color32::from_rgb(0x5F, 0x66, 0x72),
    vurgu: Color32::from_rgb(0x3E, 0x7C, 0xB1),
    hover: Color32::from_rgb(0xE4, 0xEB, 0xF3),
    aktif: Color32::from_rgb(0xD8, 0xE2, 0xEF),
    ayirici: Color32::from_rgb(0xC9, 0xCE, 0xD6),
    secim: Color32::from_rgb(0xC9, 0xDA, 0xEE),
    hata: Color32::from_rgb(0xB3, 0x26, 0x1E),
    uyari: Color32::from_rgb(0x8A, 0x64, 0x00),
    girinti: Color32::WHITE,
};

/// Paleti tam bir egui görseline dönüştürür; her iki mod aynı yoldan geçer.
fn gorsel_olustur(p: &Palet, koyu: bool) -> Visuals {
    let mut v = if koyu { Visuals::dark() } else { Visuals::light() };
    let kenarlik = Stroke::new(1.0_f32, p.ayirici);

    v.panel_fill = p.zemin;
    v.window_fill = p.pencere;
    v.extreme_bg_color = p.extreme;
    v.faint_bg_color = p.faint;
    v.text_edit_bg_color = Some(p.girinti);
    v.code_bg_color = p.girinti;
    v.weak_text_color = Some(p.metin_zayif);
    v.hyperlink_color = p.vurgu;
    v.warn_fg_color = p.uyari;
    v.error_fg_color = p.hata;
    v.window_stroke = kenarlik;
    v.window_corner_radius = CornerRadius::same(8);
    v.menu_corner_radius = CornerRadius::same(6);

    v.widgets.noninteractive.fg_stroke = Stroke::new(1.0_f32, p.metin);
    v.widgets.noninteractive.bg_fill = p.zemin;
    v.widgets.noninteractive.bg_stroke = kenarlik;

    v.widgets.inactive.fg_stroke = Stroke::new(1.0_f32, p.metin);
    v.widgets.inactive.bg_fill = p.faint;
    v.widgets.inactive.weak_bg_fill = p.faint;
    v.widgets.inactive.bg_stroke = kenarlik;
    v.widgets.inactive.corner_radius = CornerRadius::same(6);

    v.widgets.open.fg_stroke = Stroke::new(1.0_f32, p.metin);
    v.widgets.open.bg_fill = p.faint;
    v.widgets.open.weak_bg_fill = p.faint;
    v.widgets.open.bg_stroke = kenarlik;
    v.widgets.open.corner_radius = CornerRadius::same(6);

    v.widgets.hovered.fg_stroke = Stroke::new(1.0_f32, if koyu { Color32::WHITE } else { p.metin });
    v.widgets.hovered.bg_fill = p.hover;
    v.widgets.hovered.weak_bg_fill = p.hover;
    v.widgets.hovered.bg_stroke = Stroke::new(1.0_f32, p.vurgu);
    v.widgets.hovered.corner_radius = CornerRadius::same(6);

    v.widgets.active.fg_stroke = Stroke::new(1.0_f32, if koyu { Color32::WHITE } else { p.metin });
    v.widgets.active.bg_fill = p.aktif;
    v.widgets.active.weak_bg_fill = p.aktif;
    v.widgets.active.bg_stroke = Stroke::new(1.5_f32, p.vurgu);
    v.widgets.active.corner_radius = CornerRadius::same(6);

    v.selection.bg_fill = p.secim;
    v.selection.stroke = Stroke::new(1.0_f32, p.vurgu);
    v
}

/// Pencerenin egui çiziminden önce temizlendiği zemin rengi
/// (ilk kare ve yeniden boyutlandırma boşluklarında görünür).
pub fn zemin_rengi(koyu: bool) -> Color32 {
    if koyu { KOYU.zemin } else { ACIK.zemin }
}

/// Tema görselini uygular ve dokunmatik stil (44px hedef, geniş aralıklar) ayarlar.
pub fn tema_uygula(ctx: &egui::Context, koyu: bool) {
    ctx.set_visuals(gorsel_olustur(if koyu { &KOYU } else { &ACIK }, koyu));
    ctx.global_style_mut(|stil| {
        stil.spacing.interact_size = egui::vec2(DOKUNMA_HEDEFI, DOKUNMA_HEDEFI);
        stil.spacing.button_padding = egui::vec2(14.0, 12.0);
        stil.spacing.item_spacing = egui::vec2(14.0, 10.0);
        stil.spacing.icon_spacing = 8.0;
    });
}
