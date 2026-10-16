# Android preview verification

The deliverable is `dist/JBRS-Android10-arm64.apk`, a locally signed preview targeting Android 10 / API 29 on ARM64. The APK has not been installed or launched on an Android device or Android emulator in this environment. These results do not establish device compatibility or phone FPS.

| Check | Result |
|---|---|
| Stable Rust formatting and strict workspace Clippy | Passed |
| Android ARM64 strict Clippy (`-p jbrs --lib --target aarch64-linux-android`) | Passed |
| `cargo test --workspace` | **115 passed, 1 existing failure** |
| Game tests using the GLES backend | Passed; includes touch-to-game action tests |
| Android native build | Passed; Rust cdylib + SDL2 2.30.11, NDK 27.2.12479018, API 29 |
| Java and DEX | Passed; compile SDK 34, Build Tools 35.0.0, Java 21 |
| APK signature, restricted to API 29 verification | Passed; APK signature scheme v3 |
| Manifest | Package `dev.jbrs.mobile`; min/target SDK 29; ARM64; GLES2; landscape launcher |
| Native packaging | ELF64 AArch64; SDL_main, SDL Android JNI and HID registration entry points present; needed libraries are packaged SDL2 or Android public system libraries |
| APK data | ZIP CRC checked, DEX and license files present |
| GLES2 rendering on Linux/Mesa | **Six scenes × 90 frames**, forced ES 2.0; GL validation passed |
| Mobile world/UI rendering | 640×360 offscreen world target and 1024×576 UI on the test host |
| Desktop OpenGL 2.1 regression | 90-frame J-STORE scene passed |
| Real Android runtime, controls, lifecycle and performance | **Not tested** — no connected Android device/emulator |

The inherited failing test is `world::tests::navigation_exits_cell_block_without_crossing_walls`, stuck at `-144.30794,-204.92383`. It also failed in the source game before this port. It remains enabled and is recorded in `verification/workspace-tests.log`.

Touch tests cover simultaneous movement/look/jump, pointer-owned release, bounded contact/action queues, cancellation, short jump taps between physics steps, menu and hotbar taps, last weapon slot, crouch, Android Back, keypad entry/deletion, one-item consumption per quick fire tap and buying supplies without overloading the crouch button. Existing game tests continue to cover heist rules, vehicles, inventory and persistence. They do not replace physical touchscreen testing.

GLES2 scenes: menu, city skyline, J-STORE lasers, apartment kitchen, apartment bathroom and Akari R7. Actual captures and logs are in `verification/`. They were rendered by the GLES2 backend on **Linux + Xvfb + Mesa llvmpipe**, not captured from a phone. Host timings in the logs are not Android hardware performance measurements.

The package uses the same shared gameplay/world source. A separate `renderer_gles2` implements GLSL ES 1.00 shaders, explicit vertex attributes, VBOs, CPU transforms and conversion of authored dynamic quads to triangles. No desktop fixed-function GL entry points are imported by the Android game library. SDL2 itself also ships its optional system GLES1/audio support; the game requests and uses GLES2.

The Android Activity inherits SDL's surface and lifecycle implementation and forwards gesture Back to native key events. Background handlers save and clear contacts; default save paths use SDL's Android private storage directory. Physical pause/resume, surface recreation, OS process death, gesture navigation and app updates still need device testing.

Build command: `python3 Android/build_android.py`; SDK/NDK overrides are described in README.md. APK verification can be repeated with:

```sh
python3 Android/verify_apk.py --build-tools "$ANDROID_SDK_ROOT/build-tools/35.0.0" --llvm-bin "$ANDROID_NDK_HOME/toolchains/llvm/prebuilt/linux-x86_64/bin" --output Android/verification/apk.json
```

Google SDK/NDK archives used in this cloud session were checked against checksums in Google's repository metadata. The SDL source archive SHA-256 is pinned in `SOURCES.json`. The preview signing key is outside the repository and APK. `dist/SHA256SUMS` and `verification/apk.json` identify the exact delivered binary.
