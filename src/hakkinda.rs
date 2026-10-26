use gpui_kit::assets::IconName as Ikon;
use gpui_kit::component::button::{Button, ButtonVariants};
use gpui_kit::component::{ActiveTheme, Icon};
use gpui_kit::prelude::*;
use gpui_kit::*;

use crate::app::GoruntuleyiciUygulama;
use crate::arayuz::golge;
use crate::sabitler::{
    ADRES, E_POSTA, FIRMA, GELISTIRICI, POSTA_KODU, SURUM, VERGI_DAIRESI, VERGI_NUMARASI, WEB,
};

impl GoruntuleyiciUygulama {
    pub(crate) fn hakkinda_penceresi(&self, cx: &mut Context<Self>) -> AnyElement {
        let bilgi_satiri = |etiket: &'static str, deger: String, cx: &App| {
            div()
                .flex()
                .items_start()
                .gap_3()
                .py(px(3.))
                .child(
                    div()
                        .w(px(116.))
                        .text_size(px(10.))
                        .text_color(cx.theme().muted_foreground)
                        .child(etiket),
                )
                .child(
                    div()
                        .flex_1()
                        .text_size(px(11.))
                        .text_color(cx.theme().foreground)
                        .child(deger),
                )
        };

        let kisayol = |tus: &'static str, aciklama: &'static str, cx: &App| {
            div()
                .flex()
                .items_center()
                .justify_between()
                .gap_3()
                .py(px(3.))
                .child(
                    div()
                        .text_size(px(10.))
                        .text_color(cx.theme().muted_foreground)
                        .child(aciklama),
                )
                .child(
                    div()
                        .px(px(6.))
                        .py(px(2.))
                        .rounded(px(4.))
                        .bg(cx.theme().secondary)
                        .text_size(px(9.))
                        .font_family(cx.theme().mono_font_family.clone())
                        .text_color(cx.theme().foreground)
                        .child(tus),
                )
        };

        let mut kart = div()
            .w(px(620.))
            .max_w(relative(0.92))
            .flex_shrink_0()
            .p_6()
            .rounded(px(14.))
            .bg(cx.theme().popover)
            .border_1()
            .border_color(cx.theme().border)
            .shadow(golge())
            .on_mouse_down(MouseButton::Left, |_, _, cx| cx.stop_propagation());

        kart = kart.child(
            div()
                .flex()
                .items_start()
                .justify_between()
                .child(
                    div()
                        .flex()
                        .items_center()
                        .gap_3()
                        .child(
                            div()
                                .size(px(56.))
                                .rounded(px(12.))
                                .overflow_hidden()
                                .bg(cx.theme().primary)
                                .when_some(self.logo.clone(), |el, logo| {
                                    el.child(img(logo).size(px(56.)))
                                }),
                        )
                        .child(
                            div()
                                .flex()
                                .flex_col()
                                .child(
                                    div()
                                        .text_size(px(18.))
                                        .font_weight(FontWeight::BOLD)
                                        .child("SP GROUP PDF Görüntüleyici"),
                                )
                                .child(
                                    div()
                                        .text_size(px(10.))
                                        .text_color(cx.theme().muted_foreground)
                                        .child(format!("Sürüm {}", SURUM)),
                                ),
                        ),
                )
                .child(
                    Button::new("hakkinda_kapat")
                        .icon(Icon::new(Ikon::X))
                        .ghost()
                        .compact()
                        .accessibility_label("Kapat")
                        .on_click(cx.listener(|this, _, _, cx| {
                            this.hakkinda_acik = false;
                            cx.notify();
                        })),
                ),
        );

        kart = kart
            .child(div().h(px(18.)))
            .child(div().h(px(1.)).w_full().bg(cx.theme().border))
            .child(div().h(px(14.)))
            .child(
                div()
                    .text_size(px(11.))
                    .font_weight(FontWeight::SEMIBOLD)
                    .child("Kurum Bilgileri"),
            )
            .child(div().h(px(6.)))
            .child(bilgi_satiri("Geliştiren", GELISTIRICI.to_string(), cx))
            .child(bilgi_satiri("Firma", FIRMA.to_string(), cx))
            .child(bilgi_satiri("Vergi Dairesi", VERGI_DAIRESI.to_string(), cx))
            .child(bilgi_satiri(
                "Vergi Numarası",
                VERGI_NUMARASI.to_string(),
                cx,
            ))
            .child(bilgi_satiri("Posta Kodu", POSTA_KODU.to_string(), cx))
            .child(bilgi_satiri("Adres", ADRES.to_string(), cx))
            .child(bilgi_satiri("E-posta", E_POSTA.to_string(), cx))
            .child(bilgi_satiri("Web", WEB.to_string(), cx))
            .child(div().h(px(16.)))
            .child(div().h(px(1.)).w_full().bg(cx.theme().border))
            .child(div().h(px(14.)))
            .child(
                div()
                    .text_size(px(11.))
                    .font_weight(FontWeight::SEMIBOLD)
                    .child("Klavye Kısayolları"),
            )
            .child(div().h(px(6.)))
            .child(
                div()
                    .flex()
                    .gap_6()
                    .child(
                        div()
                            .flex_1()
                            .child(kisayol("Ctrl+O", "Belge aç", cx))
                            .child(kisayol("Ctrl+W", "Sekmeyi kapat", cx))
                            .child(kisayol("Ctrl+F", "Belgede ara", cx))
                            .child(kisayol("Ctrl+B", "Kenar paneli", cx))
                            .child(kisayol("Alt+T/S/A/M", "Panel seç", cx))
                            .child(kisayol("Ctrl+0", "Gerçek boyut", cx)),
                    )
                    .child(
                        div()
                            .flex_1()
                            .child(kisayol("← / →", "Önceki / sonraki sayfa", cx))
                            .child(kisayol("PgUp / PgDn", "Sayfa değiştir", cx))
                            .child(kisayol("Home / End", "İlk / son sayfa", cx))
                            .child(kisayol("Ctrl + Tekerlek", "Yakınlaştır", cx))
                            .child(kisayol("Ctrl+Shift+D", "Tema değiştir", cx))
                            .child(kisayol("F1", "Hakkında", cx))
                            .child(kisayol("Esc", "Kaplamayı kapat", cx)),
                    ),
            )
            .child(div().h(px(18.)))
            .child(div().h(px(1.)).w_full().bg(cx.theme().border))
            .child(div().h(px(12.)))
            .child(
                div()
                    .flex()
                    .items_center()
                    .justify_between()
                    .child(
                        div()
                            .text_size(px(9.))
                            .text_color(cx.theme().muted_foreground)
                            .child("© 2026 SP GROUP ENERJİ"),
                    )
                    .child(
                        div()
                            .text_size(px(9.))
                            .text_color(cx.theme().muted_foreground)
                            .child("GPUI Kit • MuPDF"),
                    ),
            );

        div()
            .absolute()
            .top_0()
            .left_0()
            .size_full()
            .bg(hsla(0., 0., 0., 0.5))
            .flex()
            .items_center()
            .justify_center()
            .p_4()
            .on_mouse_down(
                MouseButton::Left,
                cx.listener(|this, _, _, cx| {
                    this.hakkinda_acik = false;
                    cx.notify();
                }),
            )
            .child(kart)
            .into_any_element()
    }
}
