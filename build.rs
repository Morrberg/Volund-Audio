fn main() {
    if std::env::var("CARGO_CFG_TARGET_OS").as_deref() != Ok("freebsd") {
        tauri_build::build();
    }
    println!("cargo:rerun-if-changed=assets/icon.ico");
    if std::env::var("CARGO_CFG_TARGET_OS").as_deref() == Ok("windows") {
        let mut resource = winresource::WindowsResource::new();
        resource.set_icon("assets/icon.ico");
        resource.set("ProductName", "Völund Audio");
        resource.set("InternalName", "volund-audio");
        resource.set("OriginalFilename", "volund-audio.exe");
        resource.set("FileDescription", "Völund Audio - Stem Editor, Mix & Mastering");
        resource.compile().expect("kunde inte bädda in programikonen");
    }
}
