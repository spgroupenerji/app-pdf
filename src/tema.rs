// Göz yormayan aydınlık/karanlık tema paletleri ve dokunmatik stil uygulayıcısı.
use egui::{Color32, Stroke, Visuals};
use crate::sabitler::DOKUNMA_HEDEFI;

// Yumuşak hex renk çözücü; geçersiz girdide varsayılan renge döner.
fn renk(hex: &str, varsayilan: Color32) -> Color32 {
    Color32::from_hex(hex).unwrap_or(varsayilan)
}

/// Göz yormayan aydınlık tema görseli (saf beyaz/siyah yerine kırık tonlar).
pub fn gorsel_acik() -> Visuals {
    let mut v = Visuals::light();
    let zemin = renk("#FAFAF8", Color32::WHITE);
    let metin = renk("#2B2B2E", Color32::BLACK);
    let vurgu = renk("#3E7CB1", Color32::BLUE);
    v.panel_fill = zemin;
    v.window_fill = Color32::WHITE;
    v.extreme_bg_color = renk("#F0F0EE", Color32::WHITE);
    v.faint_bg_color = renk("#EDEDF0", Color32::WHITE);
    v.widgets.noninteractive.fg_stroke = Stroke::new(1.0_f32, metin);
    v.widgets.noninteractive.bg_fill = zemin;
    v.widgets.inactive.fg_stroke = Stroke::new(1.0_f32, renk("#4A4A52", Color32::BLACK));
    v.widgets.inactive.bg_fill = renk("#F2F2F5", zemin);
    v.widgets.hovered.fg_stroke = Stroke::new(1.0_f32, renk("#1A1A1D", Color32::BLACK));
    v.widgets.hovered.bg_fill = renk("#E6EEF7", zemin);
    v.widgets.active.fg_stroke = Stroke::new(1.0_f32, renk("#1A1A1D", Color32::BLACK));
    v.widgets.active.bg_fill = renk("#DDE6F2", zemin);
    v.selection.bg_fill = renk("#D6E4F0", vurgu);
    v.hyperlink_color = vurgu;
    v
}

/// Göz yormayan karanlık tema görseli (saf siyah yerine koyu lacivert-gri).
pub fn gorsel_koyu() -> Visuals {
    let mut v = Visuals::dark();
    let zemin = renk("#1B1D22", Color32::from_gray(20));
    let metin = renk("#D6D6DA", Color32::WHITE);
    let vurgu = renk("#5B9BD5", Color32::BLUE);
    v.panel_fill = zemin;
    v.window_fill = renk("#23252C", Color32::from_gray(30));
    v.extreme_bg_color = renk("#16181C", Color32::from_gray(15));
    v.faint_bg_color = renk("#2A2D34", Color32::from_gray(35));
    v.widgets.noninteractive.fg_stroke = Stroke::new(1.0_f32, metin);
    v.widgets.noninteractive.bg_fill = zemin;
    v.widgets.inactive.fg_stroke = Stroke::new(1.0_f32, renk("#A8A8B2", Color32::from_gray(160)));
    v.widgets.inactive.bg_fill = renk("#2A2D34", zemin);
    v.widgets.hovered.fg_stroke = Stroke::new(1.0_f32, Color32::WHITE);
    v.widgets.hovered.bg_fill = renk("#2E323B", zemin);
    v.widgets.active.fg_stroke = Stroke::new(1.0_f32, Color32::WHITE);
    v.widgets.active.bg_fill = renk("#363B46", zemin);
    v.selection.bg_fill = renk("#3A506B", vurgu);
    v.hyperlink_color = vurgu;
    v
}

/// Tema görselini uygular ve dokunmatik stil (48px hedef, geniş aralıklar) ayarlar.
pub fn tema_uygula(ctx: &egui::Context, koyu: bool) {
    ctx.set_visuals(if koyu { gorsel_koyu() } else { gorsel_acik() });
    ctx.global_style_mut(|stil| {
        stil.spacing.interact_size = egui::vec2(DOKUNMA_HEDEFI, DOKUNMA_HEDEFI);
        stil.spacing.button_padding = egui::vec2(14.0, 12.0);
        stil.spacing.item_spacing = egui::vec2(14.0, 10.0);
        stil.spacing.icon_spacing = 8.0;
    });
}
