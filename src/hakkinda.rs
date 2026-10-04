// Hakkında penceresi: geliştirici ve firma bilgileri + logo.
use egui::{ColorImage, RichText, TextureOptions, Window};
use crate::sabitler::{
    SURUM, GELISTIRICI, FIRMA, VERGI_DAIRESI, VERGI_NUMARASI, POSTA_KODU, ADRES, E_POSTA, WEB,
};

/// Hakkında penceresini çizer; pencere kapatılırsa bayrağı temizler.
pub fn hakkinda_penceresi(ctx: &egui::Context, acik: &mut bool) {
    if !*acik {
        return;
    }
    let id = egui::Id::new("hakkinda");
    // İç "Kapat" butonu bu bayrağı setler; .open(acik) borrow'u bitince ana bayrağa aktarılır.
    let mut kapat_istek = false;
    Window::new("ℹ Hakkında")
        .id(id)
        .collapsible(false)
        .resizable(false)
        .anchor(egui::Align2::CENTER_CENTER, egui::vec2(0.0, 0.0))
        // X kapatma butonunu gösterir; tıklanınca bayrağı false yapar.
        .open(acik)
        .show(ctx, |ui| {
            ui.vertical_centered(|ui| {
                // Logo dokusunu bir kez yukleyip onbellekte tut (her frame PNG decode onlenir).
                let doku_id = egui::Id::new("hakkinda_logo_doku");
                let mut doku = ui.ctx().data_mut(|d| d.get_persisted::<egui::TextureHandle>(doku_id));
                if doku.is_none() {
                    if let Ok(img) = image::load_from_memory(include_bytes!("../assets/logo.png")) {
                        let rgba = img.to_rgba8();
                        let boyut = [rgba.width() as usize, rgba.height() as usize];
                        let renk = ColorImage::from_rgba_unmultiplied(boyut, &rgba);
                        let yeni_doku = ui.ctx().load_texture("hakkinda_logo", renk, TextureOptions::LINEAR);
                        ui.ctx().data_mut(|d| d.insert_persisted(doku_id, yeni_doku.clone()));
                        doku = Some(yeni_doku);
                    }
                }
                if let Some(doku) = doku {
                    ui.image((doku.id(), egui::vec2(96.0, 96.0)));
                    ui.add_space(8.0);
                }
                ui.heading(RichText::new("SP GROUP PDF Görüntüleyici").size(22.0).strong());
                ui.add_space(4.0);
                ui.label(RichText::new(format!("Sürüm: {}", SURUM))
                    .size(13.0)
                    .color(ui.visuals().weak_text_color()));
                ui.add_space(16.0);
                ui.separator();
                ui.add_space(12.0);

                ui.label(RichText::new("Geliştiren").size(13.0).strong());
                ui.label(RichText::new(GELISTIRICI).size(15.0));
                ui.add_space(8.0);
                ui.label(RichText::new(FIRMA).size(14.0).strong());
                ui.add_space(14.0);

                ui.label(RichText::new("📇 Kurum Bilgileri").strong());
                ui.add_space(6.0);
                ui.label(format!("Vergi Dairesi: {}", VERGI_DAIRESI));
                ui.label(format!("Vergi Numarası: {}", VERGI_NUMARASI));
                ui.label(format!("Posta Kodu: {}", POSTA_KODU));
                ui.label(format!("Adres: {}", ADRES));
                ui.add_space(14.0);

                ui.label(RichText::new("📞 İletişim").strong());
                ui.add_space(6.0);
                ui.add(egui::Hyperlink::new(format!("mailto:{}", E_POSTA)));
                ui.add(egui::Hyperlink::new(format!("https://{}", WEB)));
                ui.add_space(12.0);
                ui.separator();
                ui.add_space(8.0);
                if ui.button("✕ Kapat").clicked() {
                    kapat_istek = true;
                }
                ui.label(RichText::new("© 2026 SP GROUP ENERJİ")
                    .size(11.0)
                    .color(ui.visuals().weak_text_color()));
            });
        });
    if kapat_istek {
        *acik = false;
    }
}
