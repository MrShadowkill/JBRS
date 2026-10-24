# Uploaded sound functions

Eight `jbrs.sound.v1` JSON documents supplied by the user for this integration. Original descriptive names and effect settings are retained in `source/*.json` and `manifest.json`. These are user-provided recordings converted to sine functions, not original JBRS recordings; no new license or authorship is asserted. Names identifying Freesound/community/yarzur_ofc are retained as supplied. Other three source names are opaque IDs; additional source/license metadata was not included in the uploads.

| Upload | Stored name | Use |
|---|---|---|
| 1.json | pavement_a | concrete/asphalt, variant A |
| 2.json | pavement_b | concrete/asphalt, variant B |
| v8…edited.json | engine_low | sedan, diesel |
| car…aggressive…json | engine_aggressive | compact, sports six, rotary |
| v8…json | engine_v8 | patrol V8, hypercar |
| мет шаг.json | metal_a | metal, variant A |
| метшаг.json | metal_b | metal, variant B |
| трава.json | grass | grass |

Run `python3 procedural_audio/assets/compile.py` from the repository root to regenerate PCM16 files and the manifest using only Python's standard library. The compiler validates the sine schema, reconstructs Hann overlap-add frames, applies JSON pitch/gain once, trims near-silent padding, and fades endpoints over 5 ms. Files are embedded with `include_bytes!`: no runtime file loading, parser, FFT, or new Rust dependency.

Engine loops crossfade 1024 samples and keep phase independently per car. RPM/load changes playback rate smoothly; vehicle IDs add a stable small tuning offset. An electric car retains its procedural motor, without a combustion recording. Four bounded footstep voices preserve tails, alternate the two pavement/metal recordings and vary pitch ±3%, gain ±6% deterministically. Pavement/grass are mixed at 0.8 and metal at 0.18 after the user's effects, preserving subdued footsteps. Water and police sirens retain procedural synthesis.
