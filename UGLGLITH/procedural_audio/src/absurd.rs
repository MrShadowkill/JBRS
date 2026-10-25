//! Original procedural modem/croak/beep/squeak/chime voices; no recorded OS audio.
use super::{advance, wave, RATE};
#[derive(Default)]
pub(super) struct AbsurdSynth {
    phase: f32,
    second: f32,
    ticks: u64,
    gain: f32,
}
impl AbsurdSynth {
    pub fn pan(&self, mode: u8, table: &[f32; 2049]) -> f32 {
        if mode == 2 {
            let period = RATE as u64 * 8;
            wave(
                table,
                (self.ticks % period) as f32 * std::f32::consts::TAU / period as f32,
            )
        } else {
            0.
        }
    }
    pub fn sample(&mut self, mode: u8, table: &[f32; 2049]) -> f32 {
        let active = (1..=5).contains(&mode);
        self.gain += ((if active { 0.08 } else { 0. }) - self.gain) * 0.002;
        if !active && self.gain < 0.00001 {
            self.gain = 0.;
            return 0.;
        }
        let t = (self.ticks % RATE as u64) as f32 / RATE as f32;
        let epoch = self.ticks / (RATE as u64 / 8);
        let (hz, gate) = match mode {
            1 => (
                [660., 1320., 990., 1760., 440., 2200., 880., 1540.][epoch as usize % 8],
                if epoch.is_multiple_of(11) { 0. } else { 1. },
            ),
            2 => (75. + t * 130., (1. - (t * 4.).fract()).powi(3)),
            3 => (
                880.,
                if t < 0.12 || (0.24..0.36).contains(&t) {
                    1.
                } else {
                    0.
                },
            ),
            4 => (1200. - t * 900., (1. - t).powi(4)),
            5 => (
                [523.25, 659.25, 783.99, 1046.5, 392., 329.63][5 - epoch as usize % 6],
                0.6,
            ),
            _ => (220., 0.),
        };
        advance(&mut self.phase, hz);
        advance(&mut self.second, hz * 1.47);
        self.ticks = self.ticks.wrapping_add(1);
        (wave(table, self.phase) + 0.3 * wave(table, self.second)) * self.gain * gate
    }
}

#[cfg(test)]
mod tests {
    use crate::{Scene, Synth, RATE};
    #[test]
    fn all_original_voices_are_bounded_and_block_size_independent() {
        for mode in 1..=5 {
            let scene = Scene {
                active: true,
                absurd_mode: mode,
                ..Default::default()
            };
            let mut a = Synth::default();
            let mut b = Synth::default();
            let mut x = vec![0.; RATE as usize * 2];
            let mut y = x.clone();
            a.render(&scene, &mut x);
            for block in y.chunks_mut(512) {
                b.render(&scene, block);
            }
            assert_eq!(x, y);
            assert!(x.iter().all(|v| v.is_finite() && v.abs() <= 0.85001));
            assert!(x.iter().any(|v| v.abs() > 0.01));
        }
    }
}
