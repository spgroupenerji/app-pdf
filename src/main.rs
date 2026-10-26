#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod app;
mod arayuz;
mod hakkinda;
mod hata;
mod isci;
mod mesaj;
mod sabitler;
mod sekme;
mod tema;

use app::{
    AramaPaneli, DosyaAc, Hakkinda, Icindekiler, IlkSayfa, KaplamaKapat, KenarPaneli, MetinPaneli,
    OncekiSayfa, Sayfalar, SekmeKapat, SonSayfa, SonrakiSayfa, TemaDegistir, Uzaklastir,
    Yakinlastir, ZumSifirla,
};
use gpui_kit::assets::Assets;
use gpui_kit::*;

fn main() {
    gpui_kit::application().with_assets(Assets).run(|cx| {
        gpui_kit::init(cx);
        let koyu_tema = app::UygulamaAyarlari::tema_yukle();
        tema::tema_kur(cx, koyu_tema);
        cx.bind_keys([
            KeyBinding::new("ctrl-o", DosyaAc, None),
            KeyBinding::new("ctrl-w", SekmeKapat, None),
            KeyBinding::new("ctrl-f", AramaPaneli, None),
            KeyBinding::new("ctrl-b", KenarPaneli, None),
            KeyBinding::new("ctrl-0", ZumSifirla, None),
            KeyBinding::new("ctrl-shift-d", TemaDegistir, None),
            KeyBinding::new("f1", Hakkinda, None),
            KeyBinding::new("alt-t", Icindekiler, None),
            KeyBinding::new("alt-s", Sayfalar, None),
            KeyBinding::new("alt-a", AramaPaneli, None),
            KeyBinding::new("alt-m", MetinPaneli, None),
            KeyBinding::new("ctrl-=", Yakinlastir, None),
            KeyBinding::new("ctrl--", Uzaklastir, None),
            KeyBinding::new("left", OncekiSayfa, None),
            KeyBinding::new("pageup", OncekiSayfa, None),
            KeyBinding::new("right", SonrakiSayfa, None),
            KeyBinding::new("pagedown", SonrakiSayfa, None),
            KeyBinding::new("home", IlkSayfa, None),
            KeyBinding::new("end", SonSayfa, None),
            KeyBinding::new("escape", KaplamaKapat, None),
        ]);

        let gorunum = gpui_kit::open_window(app::baslik_cubugu_secenekleri(), cx, |window, cx| {
            let uygulama = cx.new(|cx| app::GoruntuleyiciUygulama::yeni(window, cx));
            uygulama.update(cx, |this, cx| this.dongu_baslat(cx));
            if let Some(yol) = std::env::args().nth(1) {
                uygulama.update(cx, |this, cx| this.dosya_ac(&yol, window, cx));
            }
            uygulama
        })
        .expect("Pencere açılamadı");
        drop(gorunum);
    });
}
