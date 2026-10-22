# JBRS 0.7.6 verification

165 workspace tests passed, including imported component collisions, east-facing door operation, robbery at the supplied register and a physically arriving patrol entering the new shop. Existing player, train/railway, fleet, save recovery and robbery regression tests passed. Formatting and clippy with warnings denied passed. Source crates are identical in the standalone Windf workspace.

The road audit checks 1643 land centerline points with zero deviations over 0.2 m between deck and ground. Lane obstruction/terrain-cover and train-envelope tests also pass. Deliberate water bridges and the mountain tunnel are excluded from the land grading measurement; this is not exhaustive testing of every point on a continuous surface.

Linux GL2.1 rendered the imported exterior/interior and the updated map; screenshots are included. A driving run produced finite nonzero SDL disk audio (peak 0.0512). Host GLES2 with touch controls rendered the interior for 60 frames. Windows GNU EXE and audio device example ran via Wine; both passed. Android API29 ARM64 APK was rebuilt and passed signature, manifest, ELF/JNI and archive checks using the existing signing identity.

The demonstration WAV uses the real mixer and peaks at 0.1342. A test stresses all eight engines plus sirens and footsteps with a maximum output of 0.85. SDL queue/pause/resume test also confirms an 80 ms pause between pumps does not empty the queue. Queue filling remains bounded; arbitrarily long stalls can still cause an underrun. These checks demonstrate digital headroom and buffering, not physical-speaker sound quality.

Native Windows hardware, physical Android phone and target Ironlake hardware were not available. FPS shown in logs is host-only. Source model attribution: supplied by the user; no additional authorship or license is asserted. Artifact hashes are in artifacts.json.
