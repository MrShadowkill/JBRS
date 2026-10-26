//! Sine JSON is reconstructed offline, then embedded: no JSON parsing or FFT in game.
use super::Surface;
const PAVEMENT_A: &[u8] = include_bytes!("../assets/pavement_a.pcm");
const PAVEMENT_B: &[u8] = include_bytes!("../assets/pavement_b.pcm");
const METAL_A: &[u8] = include_bytes!("../assets/metal_a.pcm");
const METAL_B: &[u8] = include_bytes!("../assets/metal_b.pcm");
const GRASS: &[u8] = include_bytes!("../assets/grass.pcm");
const LOW: &[u8] = include_bytes!("../assets/engine_low.pcm");
const AGGRESSIVE: &[u8] = include_bytes!("../assets/engine_aggressive.pcm");
const V8: &[u8] = include_bytes!("../assets/engine_v8.pcm");
fn at(data: &[u8], i: usize) -> f32 {
    data.get(i * 2..i * 2 + 2)
        .map(|v| i16::from_le_bytes([v[0], v[1]]) as f32 / 32768.)
        .unwrap_or(0.)
}
fn sample(data: &[u8], position: f32) -> f32 {
    let i = position as usize;
    let t = position - i as f32;
    at(data, i) * (1. - t) + at(data, i + 1) * t
}
#[derive(Clone, Copy)]
struct Step {
    data: &'static [u8],
    position: f32,
    rate: f32,
    gain: f32,
    side: f32,
}
impl Default for Step {
    fn default() -> Self {
        Self {
            data: &[],
            position: 0.,
            rate: 1.,
            gain: 0.,
            side: 0.,
        }
    }
}
pub struct Steps {
    voices: [Step; 4],
    next: usize,
    count: u32,
    random: u32,
}
impl Default for Steps {
    fn default() -> Self {
        Self {
            voices: [Step::default(); 4],
            next: 0,
            count: 0,
            random: 0x6743a219,
        }
    }
}
impl Steps {
    pub fn trigger(&mut self, surface: Surface) {
        self.random ^= self.random << 13;
        self.random ^= self.random >> 17;
        self.random ^= self.random << 5;
        let variation = (self.random & 65535) as f32 / 65535.;
        let alternate = self.count % 2 == 1;
        let (data, volume) = match surface {
            Surface::Pavement => (if alternate { PAVEMENT_B } else { PAVEMENT_A }, 0.8),
            Surface::Grass => (GRASS, 0.8),
            Surface::Metal => (if alternate { METAL_B } else { METAL_A }, 0.18),
            Surface::Water => return,
        };
        self.voices[self.next] = Step {
            data,
            position: 0.,
            rate: 0.97 + variation * 0.06,
            gain: volume * (0.94 + ((self.random >> 16) as f32 / 65535.) * 0.12),
            side: if alternate { 0.08 } else { -0.08 },
        };
        self.count = self.count.wrapping_add(1);
        self.next = (self.next + 1) % self.voices.len();
    }
    pub fn render(&mut self) -> (f32, f32) {
        let (mut left, mut right) = (0., 0.);
        for v in &mut self.voices {
            if v.position >= (v.data.len() / 2) as f32 {
                continue;
            }
            let s = sample(v.data, v.position) * v.gain;
            left += s * (0.7 - v.side);
            right += s * (0.7 + v.side);
            v.position += v.rate;
        }
        (left, right)
    }
}
/// Return a crossfaded, resampled engine loop. Electric keeps its synthesized voice.
pub fn engine(kind: usize, id: u32, position: &mut f32, rate: f32) -> f32 {
    let (data, tuning, level) = match kind {
        0 => (LOW, 0.82, 1.),
        1 => (AGGRESSIVE, 0.78, 2.),
        2 => (V8, 0.85, 1.),
        3 => return 0.,
        4 => (LOW, 0.64, 1.),
        5 => (AGGRESSIVE, 1.02, 2.),
        6 => (V8, 1.14, 1.),
        _ => (AGGRESSIVE, 1.12, 2.),
    };
    let fade = 1024.;
    let period = (data.len() / 2) as f32 - fade - 1.;
    let p = (*position + (id % 101) as f32 * 173.).rem_euclid(period);
    let mut value = sample(data, p);
    if p < fade {
        let t = p / fade;
        value = value * t + sample(data, p + period) * (1. - t);
    }
    *position = (*position + rate * tuning * (0.985 + (id % 17) as f32 * 0.002)).rem_euclid(period);
    value * level
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn step_variants_preserve_tails_and_finish() {
        let mut s = Steps::default();
        s.trigger(Surface::Pavement);
        let a: Vec<_> = (0..30000).map(|_| s.render()).collect();
        s.trigger(Surface::Pavement);
        let b: Vec<_> = (0..30000).map(|_| s.render()).collect();
        assert_ne!(a, b);
        assert!(a.iter().any(|x| x.0.abs() > 0.01));
        assert_eq!(s.render(), (0., 0.));
        s.trigger(Surface::Metal);
        for _ in 0..3000 {
            s.render();
        }
        s.trigger(Surface::Grass);
        assert_eq!(
            s.voices
                .iter()
                .filter(|v| v.position < (v.data.len() / 2) as f32)
                .count(),
            2
        );
    }
    #[test]
    fn loops_are_finite_and_crossfade_wrap_is_continuous() {
        for kind in 0..8 {
            let mut p = 0.;
            for _ in 0..100000 {
                let s = engine(kind, 13, &mut p, 1.3);
                assert!(s.is_finite() && s.abs() <= 2.);
            }
        }
        let period = (LOW.len() / 2) as f32 - 1025.;
        let mut a = period - 0.01;
        let mut b = 0.;
        assert!((engine(0, 0, &mut a, 1.) - engine(0, 0, &mut b, 1.)).abs() < 0.002);
    }
}
