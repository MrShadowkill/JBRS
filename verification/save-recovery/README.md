# JBRS 0.7.5 — Invalid save recovery

160 workspace tests passed, including repair with progress retention, byte-exact original backup before saving, collision-safe backup numbering, malformed JSON fallback, and no reset on I/O failure. Formatting, clippy with warnings denied, Linux release and Windows GNU release builds passed.

Actual Windows EXE ran via Wine with invalid quality and a truncated JSON document, using the normal load path (no --smoke-test). Both reached the menu, accepted Q, saved and exited cleanly. The invalid-settings case retained balance 19425 and owned vehicles [0,7]. Both original documents survived byte-exact in save.recovery-1.json. See windows.json and per-case logs. Linux driving smoke completed 60 frames with nonzero finite audio. Android ARM64 APK rebuilt and passed package/signature checks (see ../audio/android-apk.json).

The user's exact save was not supplied, so its particular invalid field is unknown. Native Windows hardware was not available; Windows runtime checks used Wine.
