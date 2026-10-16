fn main() {
    println!("cargo:rerun-if-env-changed=SDL2_LIB_DIR");
    if std::env::var("CARGO_CFG_TARGET_OS").as_deref() == Ok("windows") {
        let dir = std::env::var_os("SDL2_LIB_DIR")
            .map(std::path::PathBuf::from)
            .unwrap_or_else(|| {
                std::path::PathBuf::from(std::env::var_os("CARGO_MANIFEST_DIR").unwrap_or_default())
                    .join("../vendor/SDL2/lib")
            });
        println!("cargo:rustc-link-search=native={}", dir.display());
    }
}
