fn main() {
    if std::env::var("CARGO_CFG_TARGET_OS").as_deref() != Ok("freebsd") {
        // Tauri requires an RGBA PNG. Normalize the artwork into target/ so
        // builds work with palette or RGB source PNGs without changing assets.
        let image = image::open("icons/icon.png")
            .expect("cannot read application icon")
            .to_rgba8();
        std::fs::create_dir_all("target").expect("cannot create target directory");
        image.save("target/volund-icon-rgba.png")
            .expect("cannot write RGBA application icon");
        println!("cargo:rerun-if-changed=icons/icon.png");
        tauri_build::build();
    }
}
