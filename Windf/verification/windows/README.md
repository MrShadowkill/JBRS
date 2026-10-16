# Windows binary runtime checks

Actual x86-64 Windows executables were run with Wine 10.0, in an isolated win64 prefix, on the Linux cloud host. Rendering used Xvfb and Mesa llvmpipe with `MESA_GL_VERSION_OVERRIDE=2.1`, `MESA_GLSL_VERSION_OVERRIDE=120`, `LIBGL_ALWAYS_SOFTWARE=1`.

- `tests-*.log`: all 19 Windows unit-test binaries, run with `--test-threads=1`; 100 passed, one existing navigation test failed.
- `scene-*.log` and corresponding PNG: six smoke scenes, 90 frames each.
- `vehicle-0.log` through `vehicle-7.log`: all eight vehicle models, 60 frames each; `akari-r7.png` shows vehicle 7.
- `default-save.log`: normal startup through `wine cmd /c .../Windf/run.cmd`, from outside the game folder, then a WM_DELETE_WINDOW request. Exit 0; `%LOCALAPPDATA%/JBRS-Windf/save.json` was read back and parsed.
- `unicode-save-*.log`: two normal launches with `JBRS_SAVE` pointing at `Збереження/save.json`. Both exited 0; both saves parsed successfully. This exercises replacement of an existing file through MoveFileExW.
- `summary.json`: counts and tested cases; `finish-results.json`: launcher/save outcomes.

Typical rendering command (with Wine and Xvfb installed):

```sh
WINEDEBUG=-all LIBGL_ALWAYS_SOFTWARE=1 MESA_GL_VERSION_OVERRIDE=2.1 MESA_GLSL_VERSION_OVERRIDE=120 JBRS_SCENE=jstore-lasers-new JBRS_FRAMES=90 xvfb-run -a wine dist/jbrs.exe --smoke-test
```

Unit binaries were previously built with `cargo test --workspace --target x86_64-pc-windows-gnu --no-run`. SDL2 was located using WINEPATH pointing at the dist directory. Initial cloud Wine relocation/bootstrap issues and a harness pipe timeout were resolved before rerunning the affected tests. They required no game code changes. The final known failure is recorded in `tests-world-*.log`.

Software-renderer timing is not a Windows 11 hardware performance measurement. Native Windows interaction, GPU drivers, audio and MSVC build scripts remain untested here.
