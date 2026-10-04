#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod app;
mod hata;
mod hakkinda;
mod isci;
mod mesaj;
mod sabitler;
mod sekme;
mod tema;

use eframe::egui;
use sabitler::{PENCERE_GENISLIK, PENCERE_YUKSEKLIK};

// Pencere ikonu: assets/logo.png derleme zamaninda binary'ye gomulur.
fn logo_icon() -> Option<egui::IconData> {
    let veri = include_bytes!("../assets/logo.png");
    let img = image::load_from_memory(veri).ok()?;
    let rgba = img.to_rgba8();
    let (genislik, yukseklik) = (rgba.width(), rgba.height());
    Some(egui::IconData {
        rgba: rgba.into_vec(),
        width: genislik,
        height: yukseklik,
    })
}

// Windows giris noktasi: eframe uzerinden yerel (native) pencere baslatir.
// persist_window: false -> eski pencere geometrisi geri yuklenmez (tasma onlenir).
// Not: with_maximized bilerek kullanilmaz; eframe pencereyi gizli olusturur ve
// Windows gizli pencerede maximize'i tutarsiz uygular (sagda bosluk sorunu).
// Bunun yerine pencere gorunur olduktan sonra ilk frame'de
// ViewportCommand::Maximized(true) gonderilir (bkz. app.rs).
fn main() -> eframe::Result<()> {
    let mut viewport = egui::ViewportBuilder::default()
        .with_inner_size([PENCERE_GENISLIK, PENCERE_YUKSEKLIK])
        .with_min_inner_size([600.0, 400.0])
        .with_title("SP GROUP PDF Görüntüleyici")
        .with_app_id("spgroup_pdf_goruntuleyici");
    if let Some(icon) = logo_icon() {
        viewport = viewport.with_icon(icon);
    }

    let secenekler = eframe::NativeOptions {
        viewport,
        persist_window: false,
        renderer: eframe::Renderer::Glow,
        ..Default::default()
    };

    eframe::run_native(
        "SP GROUP PDF Görüntüleyici",
        secenekler,
        Box::new(|_olusturma_baglami| Ok(Box::new(app::GoruntuleyiciUygulama::yeni()))),
    )
}
