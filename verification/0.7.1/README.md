# JBRS 0.7.1 verification

Release base: `5f6ef0a` (latest main fetched before packaging). The old pre-recovery local draft is preserved separately in Git stash and was not applied over newer gameplay changes.

- Stable Rust 1.90.0, Linux cloud host.
- `cargo fmt --all -- --check`: passed.
- `cargo test --workspace`: **147 passed, 0 failed**.
- `cargo clippy --workspace --all-targets -- -D warnings`: passed.
- Linux release, Windows GNU release and Android ARM64 release: built successfully as package 0.7.1.
- Linux GL 2.1 / GLSL 1.20: menu, train and J-Store, 60 frames each, C13HPT pack at night; clean exit.
- Windows executable through Wine 10.0: menu and train, 60 frames each, C13HPT pack at night; clean exit.
- GLES 2.0 on host Mesa with touch UI: train, 60 frames, C13HPT at night; clean exit.
- APK: versionCode 3; API 29 signature, manifest, ARM64 ELF/JNI and CRC checks passed. Same preview signing identity.
- ZIP CRC and packaged binary hashes checked after packaging; shaders and texture packs are included.

Native Windows 11, physical Android hardware and Intel Ironlake performance were not tested. Software-renderer logs are host observations, not target-hardware FPS claims. Historical verification reports elsewhere describe their earlier builds.

See `SHA256SUMS` and `artifacts.json` for downloadable artifact hashes, `desktop-scenes.json` / `windows-scenes.json` for launches, and `android-apk.json` for APK verification.
