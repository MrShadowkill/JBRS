# 8.0.7GG verification

Workspace release tests: 191 passed, 1 display-dependent test ignored. Clippy completed successfully. New tests cover the exact 10-second boundary, hit testing, background timer, fresh-launch locking and zero allocations in countdown/hit testing.

Linux: Xvfb/Mesa startup warning rendered, explicit acknowledgment followed by 480 smoke frames. Windows x64: same gate followed by 60 smoke frames under Wine. Android ARM64: APK signature verified for API29, manifest/package, DEX, AArch64 ELF linkage and JNI entry points checked (ANDROID_VERIFICATION.json).

No physical Windows or Android device test. Linux executable requires glibc >=2.35, SDL2 and OpenGL2.1. Android APK uses a local preview signing certificate, not a Play Store release identity.
