//! Fixed-size stutter, bit crushing, dropouts and digital squeals, before the limiter.
use super::{safe, RATE};
pub(super) struct Glitch {
    history: [[f32; 2]; 4096],
    cursor: usize,
    ticks: u64,
    mix: f32,
    held: [f32; 2],
    phase: f32,
}
impl Default for Glitch {
    fn default() -> Self {
        Self {
            history: [[0.; 2]; 4096],
            cursor: 0,
            ticks: 0,
            mix: 0.,
            held: [0.; 2],
            phase: 0.,
        }
    }
}
impl Glitch {
    pub fn process(&mut self, dry: [f32; 2], amount: f32) -> [f32; 2] {
        let target = safe(amount, 0., 1.);
        self.mix += (target - self.mix) * 0.003;
        if self.mix < 0.00001 && target == 0. {
            self.mix = 0.;
        }
        self.history[self.cursor] = dry;
        let epoch = self.ticks / (RATE as u64 / 7);
        let span = 90 + (epoch as usize % 11) * 37;
        let offset = 700 + self.ticks as usize % span;
        let repeat = self.history[(self.cursor + 4096 - offset) % 4096];
        if self.ticks.is_multiple_of(5 + epoch % 13) {
            self.held = repeat.map(|v| ((v * 18.).clamp(-1., 1.) * 7.).round() / 7.);
        }
        self.phase = (self.phase + (140. + (epoch % 17) as f32 * 83.) / RATE as f32) % 1.;
        let buzz = if self.phase < 0.5 { 0.11 } else { -0.11 };
        let gate = if (self.ticks / 600).is_multiple_of(5) {
            0.
        } else {
            1.
        };
        let wet = [
            (self.held[0] * 0.36 + buzz) * gate,
            (self.held[1] * -0.36 - buzz) * gate,
        ];
        self.cursor = (self.cursor + 1) % 4096;
        self.ticks = self.ticks.wrapping_add(1);
        [
            dry[0] + (wet[0] - dry[0]) * self.mix,
            dry[1] + (wet[1] - dry[1]) * self.mix,
        ]
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn dry_audio_is_exact_and_corruption_recovers() {
        let mut dsp = Glitch::default();
        assert_eq!(dsp.process([0.2, -0.3], 0.), [0.2, -0.3]);
        let mut changed = false;
        for _ in 0..RATE {
            let out = dsp.process([0.2, -0.3], 1.);
            changed |= out != [0.2, -0.3];
            assert!(out.iter().all(|v| v.is_finite() && v.abs() < 1.));
        }
        assert!(changed);
        for _ in 0..RATE {
            dsp.process([0.; 2], 0.);
        }
        assert_eq!(dsp.process([0.2, -0.3], 0.), [0.2, -0.3]);
    }
}
