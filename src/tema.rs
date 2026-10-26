use std::rc::Rc;

use gpui_kit::component::{Theme, ThemeConfig, ThemeMode};
use gpui_kit::{App, Rgba};

/// Belge tuvali: koyu modda sayfanın arkasındaki derin grafit.
pub const TUVAL_KOYU: u32 = 0x0D1015;
/// Belge tuvali: açık modda yumuşak gri.
pub const TUVAL_ACIK: u32 = 0xE5E9EF;

pub fn tuval_rengi(koyu: bool) -> Rgba {
    gpui_kit::rgb(if koyu { TUVAL_KOYU } else { TUVAL_ACIK })
}

fn yapilandirma(koyu: bool) -> ThemeConfig {
    let mut yapilandirma = ThemeConfig {
        is_default: true,
        name: if koyu {
            "SP Grafit Koyu".into()
        } else {
            "SP Grafit Açık".into()
        },
        mode: if koyu {
            ThemeMode::Dark
        } else {
            ThemeMode::Light
        },
        font_size: Some(13.0),
        font_family: Some("Segoe UI".into()),
        mono_font_family: Some("Consolas".into()),
        mono_font_size: Some(12.0),
        radius: Some(6),
        radius_lg: Some(10),
        shadow: Some(true),
        ..Default::default()
    };

    let c = &mut yapilandirma.colors;
    if koyu {
        c.background = Some("#12151B".into());
        c.foreground = Some("#E7EAF0".into());
        c.border = Some("#2B3240".into());
        c.muted = Some("#1B1F27".into());
        c.muted_foreground = Some("#98A2B3".into());
        c.primary = Some("#5B9BD5".into());
        c.primary_foreground = Some("#0B1220".into());
        c.primary_hover = Some("#74B0E4".into());
        c.primary_active = Some("#4A87BF".into());
        c.secondary = Some("#232936".into());
        c.secondary_foreground = Some("#E7EAF0".into());
        c.secondary_hover = Some("#2A3140".into());
        c.secondary_active = Some("#333C4D".into());
        c.accent = Some("#263241".into());
        c.accent_foreground = Some("#E7EAF0".into());
        c.button = Some("#202631".into());
        c.button_foreground = Some("#E7EAF0".into());
        c.button_hover = Some("#2A3140".into());
        c.button_active = Some("#333C4D".into());
        c.input = Some("#3A4454".into());
        c.ring = Some("#5B9BD5".into());
        c.selection = Some("#2B4A6B".into());
        c.caret = Some("#74B0E4".into());
        c.popover = Some("#1F242E".into());
        c.popover_foreground = Some("#E7EAF0".into());
        c.list = Some("#191D25".into());
        c.list_hover = Some("#232A36".into());
        c.list_active = Some("#2B3647".into());
        c.list_active_border = Some("#5B9BD5".into());
        c.list_even = Some("#1B1F27".into());
        c.list_head = Some("#171B22".into());
        c.tab_bar = Some("#151920".into());
        c.tab = Some("#191D25".into());
        c.tab_active = Some("#232A36".into());
        c.tab_foreground = Some("#98A2B3".into());
        c.tab_active_foreground = Some("#E7EAF0".into());
        c.title_bar = Some("#12161C".into());
        c.title_bar_border = Some("#2B3240".into());
        c.sidebar = Some("#161A21".into());
        c.sidebar_foreground = Some("#E7EAF0".into());
        c.sidebar_border = Some("#2B3240".into());
        c.sidebar_accent = Some("#232A36".into());
        c.sidebar_accent_foreground = Some("#E7EAF0".into());
        c.sidebar_primary = Some("#5B9BD5".into());
        c.sidebar_primary_foreground = Some("#0B1220".into());
        c.skeleton = Some("#232A36".into());
        c.scrollbar = Some("#1B1F27".into());
        c.scrollbar_thumb = Some("#3A4454".into());
        c.scrollbar_thumb_hover = Some("#4A5568".into());
        c.danger = Some("#F08078".into());
        c.danger_foreground = Some("#2A0B08".into());
        c.danger_hover = Some("#F49A93".into());
        c.danger_active = Some("#D96A63".into());
        c.success = Some("#5BC49A".into());
        c.success_foreground = Some("#06251A".into());
        c.success_hover = Some("#77D2AC".into());
        c.success_active = Some("#45A57F".into());
        c.warning = Some("#E8C468".into());
        c.warning_foreground = Some("#2B2106".into());
        c.warning_hover = Some("#F0D486".into());
        c.warning_active = Some("#C9A64E".into());
        c.link = Some("#74B0E4".into());
        c.link_hover = Some("#8CC2EF".into());
        c.link_active = Some("#4A87BF".into());
    } else {
        c.background = Some("#EEF1F5".into());
        c.foreground = Some("#1D2430".into());
        c.border = Some("#D5DBE3".into());
        c.muted = Some("#F1F3F6".into());
        c.muted_foreground = Some("#5B6675".into());
        c.primary = Some("#2F6FA8".into());
        c.primary_foreground = Some("#FFFFFF".into());
        c.primary_hover = Some("#3D82C0".into());
        c.primary_active = Some("#255A8B".into());
        c.secondary = Some("#E9EDF2".into());
        c.secondary_foreground = Some("#1D2430".into());
        c.secondary_hover = Some("#DFE5EC".into());
        c.secondary_active = Some("#D4DCE5".into());
        c.accent = Some("#E3ECF7".into());
        c.accent_foreground = Some("#1D2430".into());
        c.button = Some("#FFFFFF".into());
        c.button_foreground = Some("#1D2430".into());
        c.button_hover = Some("#F1F3F6".into());
        c.button_active = Some("#E8ECF1".into());
        c.input = Some("#C9D1DB".into());
        c.ring = Some("#2F6FA8".into());
        c.selection = Some("#C9DDF2".into());
        c.caret = Some("#2F6FA8".into());
        c.popover = Some("#FFFFFF".into());
        c.popover_foreground = Some("#1D2430".into());
        c.list = Some("#FFFFFF".into());
        c.list_hover = Some("#F1F3F6".into());
        c.list_active = Some("#E3ECF7".into());
        c.list_active_border = Some("#2F6FA8".into());
        c.list_even = Some("#F7F9FB".into());
        c.list_head = Some("#F1F3F6".into());
        c.tab_bar = Some("#E9EDF2".into());
        c.tab = Some("#EAEEF3".into());
        c.tab_active = Some("#FFFFFF".into());
        c.tab_foreground = Some("#5B6675".into());
        c.tab_active_foreground = Some("#1D2430".into());
        c.title_bar = Some("#E9EDF2".into());
        c.title_bar_border = Some("#D5DBE3".into());
        c.sidebar = Some("#F3F5F8".into());
        c.sidebar_foreground = Some("#1D2430".into());
        c.sidebar_border = Some("#D5DBE3".into());
        c.sidebar_accent = Some("#E3ECF7".into());
        c.sidebar_accent_foreground = Some("#1D2430".into());
        c.sidebar_primary = Some("#2F6FA8".into());
        c.sidebar_primary_foreground = Some("#FFFFFF".into());
        c.skeleton = Some("#E3E7EC".into());
        c.scrollbar = Some("#E9EDF2".into());
        c.scrollbar_thumb = Some("#B8C1CC".into());
        c.scrollbar_thumb_hover = Some("#9AA6B4".into());
        c.danger = Some("#B3261E".into());
        c.danger_foreground = Some("#FFFFFF".into());
        c.danger_hover = Some("#C93A32".into());
        c.danger_active = Some("#8F1D17".into());
        c.success = Some("#1F7A55".into());
        c.success_foreground = Some("#FFFFFF".into());
        c.success_hover = Some("#2A9167".into());
        c.success_active = Some("#176343".into());
        c.warning = Some("#8A6400".into());
        c.warning_foreground = Some("#FFFFFF".into());
        c.warning_hover = Some("#A77A00".into());
        c.warning_active = Some("#6E5000".into());
        c.link = Some("#2F6FA8".into());
        c.link_hover = Some("#3D82C0".into());
        c.link_active = Some("#255A8B".into());
    }
    yapilandirma
}

/// SP Grafit temasını kaydeder ve seçilen modu uygular.
pub fn tema_kur(cx: &mut App, koyu: bool) {
    let acik = Rc::new(yapilandirma(false));
    let koyu_yapilandirma = Rc::new(yapilandirma(true));
    Theme::update(cx, |theme| {
        theme.light_theme = acik;
        theme.dark_theme = koyu_yapilandirma;
    });
    tema_uygula(cx, koyu);
}

pub fn tema_uygula(cx: &mut App, koyu: bool) {
    Theme::change(
        if koyu {
            ThemeMode::Dark
        } else {
            ThemeMode::Light
        },
        None,
        cx,
    );
}
