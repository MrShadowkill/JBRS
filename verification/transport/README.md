# Transport update verification — 2026-10-16

Host: Linux cloud workspace, Rust stable 1.90.0. Rendering checks use Xvfb and Mesa llvmpipe. These are compatibility checks, not measurements on Intel Ironlake or a phone.

| Check | Result |
|---|---|
| `cargo fmt --all -- --check` | Passed |
| `cargo clippy --workspace --all-targets -- -D warnings` | Passed |
| `cargo test --workspace` | **125 passed, 0 failed, 0 ignored** |
| `cargo build --release` | Passed; Linux executable in `bin/jbrs` |
| Windows GNU release + Clippy for all workspace targets | Passed; `Windf/dist/jbrs.exe` |
| Desktop GL 2.1 / GLSL 1.20 smoke scenes | Menu, traffic, river, train, locomotive, museum, J-Store, dispatch; exit 0 |
| Final desktop executable | Traffic + train, 60 frames each; exit 0 |
| Windows executable through Wine 10.0 | Menu, traffic, river, train, J-Store; 60 frames each; exit 0 |
| GLES 2.0 with touch controls, host Mesa | Traffic, river, train, J-Store; 60 frames each; exit 0 |
| Android 10 ARM64 APK | Rebuilt, same preview signing certificate, versionCode 2; API 29 signature, manifest, DEX, CRC and ELF/JNI checks passed |
| Native Windows 11 / physical Android device | **Not tested**: neither platform is available on this host |

New regression coverage checks fast car impacts and weak restitution, falling to a drivable riverbed, disconnected overpasses, physical patrol arrival, forest clearance, flat non-solid water, roads free of terrain overlap, the full rail corridor, bounded railway grades and non-overlapping wagon spacing. The previously failing cell-block navigation test now passes. Existing end-to-end cases still exercise patrol arrival followed by entering a shop, factory or apartment and arresting the player; closed-door/floor line-of-sight tests also pass.

`map-audit.log` records the current world inventory. `*-scenes.json` records actual launches. PNGs show rendered results. `android-apk.json` records packaging checks and explicitly reports the absence of a device runtime test. Source synchronization into `Windf` is checked before packaging.

Branding exceptions are limited to mandatory original copyright notices, the Linux legacy-save migration identifier, and unmodified historical verification logs. Current UI, commands and executable filenames use JBRS. Historical reports elsewhere in the repository describe their earlier builds; this report supersedes their navigation-failure note.
