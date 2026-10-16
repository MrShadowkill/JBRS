# Windows port verification

Environment: Linux cloud host, Rust stable 1.90.0; Windows GNU target linked with MinGW-w64 GCC 14. SDL2 2.30.11 x64, official release binaries.

| Check | Result |
|---|---|
| `cargo fmt --all -- --check` | Passed |
| `cargo clippy --workspace --all-targets -- -D warnings` (Linux) | Passed |
| Same Clippy command with `--target x86_64-pc-windows-gnu` | Passed |
| `cargo check --workspace --all-targets --target x86_64-pc-windows-msvc` | Passed; compile check, no MSVC executable linked |
| `cargo build --release --locked --target x86_64-pc-windows-gnu` | Passed; distributable EXE |
| `cargo test --workspace --target x86_64-pc-windows-gnu --no-run` | All Windows test executables built; subsequently executed through Wine: 100 passed, 1 existing navigation failure |
| `cargo build --release --locked` (Linux) | Passed |
| `cargo test --workspace` (Linux) | 100 passed; 1 existing navigation failure |
| Linux runtime, J-STORE laser scene | 90 frames rendered, Mesa OpenGL 2.1 / GLSL 1.20 under Xvfb; no reported GL errors |
| Windows PE architecture | PE32+ x86-64 |
| Windows EXE under Wine 10.0 | Passed: menu, skyline, 4 J-STORE scenes, all 8 vehicle models |
| `run.cmd` from another working directory | Passed through Wine `cmd.exe`; clean exit code 0 |
| Default Windows save path and Unicode path | Passed: real normal startup/quit, JSON readback, repeated Unicode-path save |
| Windows native runtime / MSVC build script execution | Not tested: this host runs Linux, with no Windows VM |

Existing failure: `world::tests::navigation_exits_cell_block_without_crossing_walls`, stuck at `-144.30794,-204.92383`. Reproduced before the Windows port in the original workspace. It is not marked ignored or bypassed.

The executable imports Windows system DLLs and `SDL2.dll`. Neither it nor SDL2 imports extra MinGW runtime DLLs. The portable package includes SDL2 and license/attribution files. SDL2 hashes and upstream archive URLs are recorded under `vendor/SDL2`.

The Windows-specific GL loader, process memory query and save replacement were exercised by the actual Windows binaries under Wine 10.0. Rendering used Mesa llvmpipe with OpenGL 2.1 and GLSL 1.20 forced; screenshots were inspected. Each scene rendered 90 frames; each of the eight vehicle scenes rendered 60 frames. Normal launch tests sent a WM_DELETE_WINDOW request, allowing the real SDL quit/save path to finish. Both initial and replacement Unicode-path saves were parsed back as valid version-1 JSON.

Logs, screenshots and machine-readable counts: [verification/windows/summary.json](verification/windows/summary.json). No game source changes were needed for these checks. Wine was installed only in the external cloud tools directory; no Wine dependency or extra DLL was added to the Windows package.

Native Windows 11 GPU drivers, interactive controls, audio and native MSVC packaging still require testing on a Windows 11 machine. Wine is a compatibility layer, not a native Windows 11 test. No target-hardware FPS claim is made.

Source snapshot includes the current game crates, Cargo manifests/lockfile, embedded models, authoring tools and project documentation. Old builds, archives, screenshots, the Git directory and the obsolete `WIND` helper folder are excluded. Root game source files remain unchanged by this port.
