# Поточна збірка: 0.7.6

Поточні тести міксу, запасу рівня, SDL-черги та запуску гри: [../0.7.6/README.md](../0.7.6/README.md). Наведені нижче результати — історична перевірка 0.7.4.

# JBRS 0.7.4 — uploaded sound integration

Eight supplied sine-function JSON documents are preserved in `procedural_audio/assets/source/`; the reproducible standard-library Python compiler reconstructs, applies JSON effects, trims padding and emits embedded PCM16. No JSON/FFT or asset I/O happens during gameplay. Four footstep voices preserve tails and alternate surface recordings with small deterministic pitch/gain variation. Eight car profiles use three crossfaded recordings with individual tuning; electric retains procedural synthesis.

Validation results for this release are in `clips-*.log`, `clips-checks.json`, `android-apk.json`, `windows.json` and `artifacts.json`. The preview uses the same synthesizer as the game. Earlier `tone-*` and other logs describe previous releases.

No physical phone, native Windows 11 audio device, loudspeaker listening or target Ironlake hardware test was performed. Host dummy/disk audio and Wine validate the pipeline and generated signal, not physical-device audio quality or hardware performance.
