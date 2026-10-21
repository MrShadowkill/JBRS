//! Render exactly the same synthesizer used by the game to a WAV for listening.
use procedural_audio::*;
use std::{
    fs::File,
    io::{self, Write},
};
fn main() -> Result<(), Box<dyn std::error::Error>> {
    let path = std::env::args()
        .nth(1)
        .unwrap_or_else(|| "jbrs-audio-preview.wav".into());
    let mut pcm = Vec::new();
    let mut synth = Synth::default();
    let mut block = [0.; 1024];
    // Eight 2-second engine revs, four walking surfaces, then an emergency siren.
    for segment in 0..13 {
        for n in 0..86 {
            let mut scene = Scene {
                active: true,
                ..Default::default()
            };
            if segment < 8 {
                scene.engines[0] = Engine {
                    id: segment as u32,
                    kind: segment,
                    speed: n as f32 / 86. * 55.,
                    maximum: 60.,
                    throttle: 0.8,
                    gain: 0.25,
                    ..Default::default()
                };
            } else if segment < 12 {
                if n % 18 == 0 {
                    synth.step(
                        [
                            Surface::Pavement,
                            Surface::Grass,
                            Surface::Metal,
                            Surface::Water,
                        ][segment - 8],
                    );
                }
            } else {
                scene.engines[0] = Engine {
                    id: 99,
                    kind: 2,
                    siren: true,
                    gain: 0.22,
                    pan: 0.2,
                    ..Default::default()
                };
            }
            synth.render(&scene, &mut block);
            for v in block {
                pcm.extend_from_slice(&((v * 32767.) as i16).to_le_bytes());
            }
        }
    }
    let len = u32::try_from(pcm.len()).map_err(|_| io::Error::other("preview too large"))?;
    let mut f = File::create(&path)?;
    f.write_all(b"RIFF")?;
    f.write_all(&(36 + len).to_le_bytes())?;
    f.write_all(b"WAVEfmt ")?;
    f.write_all(&16u32.to_le_bytes())?;
    f.write_all(&1u16.to_le_bytes())?;
    f.write_all(&2u16.to_le_bytes())?;
    f.write_all(&RATE.to_le_bytes())?;
    f.write_all(&(RATE * 4).to_le_bytes())?;
    f.write_all(&4u16.to_le_bytes())?;
    f.write_all(&16u16.to_le_bytes())?;
    f.write_all(b"data")?;
    f.write_all(&len.to_le_bytes())?;
    f.write_all(&pcm)?;
    println!("Wrote {path}: eight engine profiles, four footstep surfaces, siren");
    Ok(())
}
