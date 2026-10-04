// Windows exe ikonu: tek kaynak assets/logo.png derleme sirasinda 256x256
// .ico'ya donusturulur ve kaynak (resource) olarak exe'ye gomulur.
// Uretilen dosyalar OUT_DIR'e yazilir; kaynak agaci temiz kalir.
fn main() {
    println!("cargo:rerun-if-changed=assets/logo.png");
    println!("cargo:rerun-if-env-changed=UYGULAMA_SURUMU");
    let surum = std::env::var("UYGULAMA_SURUMU")
        .unwrap_or_else(|_| env!("CARGO_PKG_VERSION").to_string());
    println!("cargo:rustc-env=UYGULAMA_SURUMU={}", surum);
    if std::env::var("CARGO_CFG_TARGET_OS").as_deref() != Ok("windows") {
        return;
    }

    let out_dir = std::env::var("OUT_DIR").expect("OUT_DIR tanimli olmali");

    // PNG'yi oku ve ICO sinirina (256x256) sabitle
    let resim = image::open("assets/logo.png").expect("assets/logo.png okunamadi");
    let resim = resim.resize_exact(256, 256, image::imageops::FilterType::Lanczos3);
    let rgba = resim.to_rgba8();
    let (genislik, yukseklik) = rgba.dimensions();

    // ICO uret
    let ikon_resim = ico::IconImage::from_rgba_data(genislik, yukseklik, rgba.into_raw());
    let mut ikon_dizini = ico::IconDir::new(ico::ResourceType::Icon);
    ikon_dizini.add_entry(ico::IconDirEntry::encode(&ikon_resim).expect("ICO encode hatasi"));
    let ico_yolu = std::path::Path::new(&out_dir).join("logo.ico");
    let mut dosya = std::fs::File::create(&ico_yolu).expect("logo.ico olusturulamadi");
    ikon_dizini.write(&mut dosya).expect("logo.ico yazilamadi");

    // RC.EXE eski bir ANSI aracidir; .rc icindeki mutlak yolda Turkce karakter
    // kod sayfasina takilir. Kok dizinine goreli, ileri bollu ASCII yol kullan.
    let rc_yolu = std::path::Path::new(&out_dir).join("icon.rc");
    let kok = std::env::var("CARGO_MANIFEST_DIR").expect("CARGO_MANIFEST_DIR tanimli olmali");
    let goreceli_ikon = std::path::Path::new(&ico_yolu)
        .strip_prefix(&kok)
        .expect("OUT_DIR kok dizini altinda olmali");
    let rc_icerik = format!(
        "1 ICON \"{}\"",
        goreceli_ikon.display().to_string().replace('\\', "/")
    );
    std::fs::write(&rc_yolu, rc_icerik).expect("icon.rc yazilamadi");
    embed_resource::compile(&rc_yolu, embed_resource::NONE)
        .manifest_optional()
        .expect("Ikon kaynagi gomulemedi");
}
