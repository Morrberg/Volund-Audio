fn main() {
    println!("cargo:rerun-if-changed=assets/icon.ico");
    if std::env::var("CARGO_CFG_TARGET_OS").as_deref() == Ok("windows") {
        let mut resource = winresource::WindowsResource::new();
        resource.set_icon("assets/icon.ico");
        resource.set("ProductName", "Vocal Cleaner");
        resource.set("FileDescription", "Vocal Cleaner - Stem Editor");
        resource.compile().expect("kunde inte bädda in programikonen");
    }
}
