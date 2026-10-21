//! Bounded hybrid audio: uploaded sine-function clips and procedural voices.
mod clips;
mod device;
pub use device::Audio;
use std::f32::consts::{FRAC_PI_2, TAU};
pub const RATE: u32 = 22050;
pub const VOICES: usize = 8;
#[derive(Clone, Copy, Default)]
pub struct Engine {
    pub id: u32,
    pub kind: usize,
    pub speed: f32,
    pub maximum: f32,
    pub throttle: f32,
    pub gain: f32,
    pub pan: f32,
    pub siren: bool,
}
#[derive(Clone, Copy, Default)]
pub enum Surface {
    #[default]
    Pavement,
    Grass,
    Metal,
    Water,
}
#[derive(Default)]
pub struct Scene {
    pub engines: [Engine; VOICES],
    pub steps: u8,
    pub surface: Surface,
    pub active: bool,
}
/// Idle RPM, redline, firing pulses per revolution, harmonic mix, phase, electric.
const PROFILES: [(f32, f32, f32, f32, f32, bool); 8] = [
    (780., 6600., 2., 0.32, 0.0, false),
    (900., 6200., 1.5, 0.23, 0.7, false),
    (650., 6200., 4., 0.48, 1.1, false),
    (900., 11000., 1., 0.12, 1.57, true),
    (650., 4300., 3., 0.58, 2.1, false),
    (900., 8500., 3., 0.28, 2.7, false),
    (850., 10000., 6., 0.22, 0.4, false),
    (1000., 9500., 2., 0.42, 1.9, false),
];
#[derive(Clone, Copy)]
struct Voice {
    id: u32,
    input: Engine,
    phase: f32,
    modulation: f32,
    siren_phase: f32,
    siren_sweep: f32,
    hz: f32,
    gain: f32,
    pan: f32,
    siren_gain: f32,
    clip_position: f32,
    clip_rate: f32,
}
impl Default for Voice {
    fn default() -> Self {
        Self {
            id: u32::MAX,
            input: Engine::default(),
            phase: 0.,
            modulation: 0.,
            siren_phase: 0.,
            siren_sweep: 0.,
            hz: 30.,
            gain: 0.,
            pan: 0.,
            siren_gain: 0.,
            clip_position: 0.,
            clip_rate: 0.7,
        }
    }
}
pub struct Synth {
    voices: [Voice; VOICES],
    table: [f32; 2049],
    foot: f32,
    foot_phase: f32,
    foot_surface: Surface,
    foot_side: f32,
    steps: clips::Steps,
}
impl Default for Synth {
    fn default() -> Self {
        Self {
            voices: [Voice::default(); VOICES],
            table: std::array::from_fn(|i| (i as f32 * TAU / 2048.).sin()),
            foot: 10.,
            foot_phase: 0.,
            foot_surface: Surface::Pavement,
            foot_side: 1.,
            steps: clips::Steps::default(),
        }
    }
}
fn wave(table: &[f32; 2049], phase: f32) -> f32 {
    let x = phase.rem_euclid(TAU) * (2048. / TAU);
    let i = (x as usize).min(2047);
    let t = (x - i as f32).clamp(0., 1.);
    table[i] + (table[i + 1] - table[i]) * t
}
fn advance(phase: &mut f32, hz: f32) {
    *phase = (*phase + TAU * hz / RATE as f32).rem_euclid(TAU);
}
fn safe(v: f32, min: f32, max: f32) -> f32 {
    if v.is_finite() {
        v.clamp(min, max)
    } else {
        min
    }
}
impl Synth {
    pub fn step(&mut self, surface: Surface) {
        self.steps.trigger(surface);
        self.foot = if matches!(surface, Surface::Water) {
            0.
        } else {
            10.
        };
        self.foot_phase = 0.;
        self.foot_surface = surface;
        self.foot_side = -self.foot_side;
    }
    pub fn render(&mut self, scene: &Scene, out: &mut [f32]) {
        // Keep a vehicle on the same oscillator even when distance ranking changes.
        for v in &mut self.voices {
            if let Some(e) = scene.engines.iter().find(|e| e.gain > 0. && e.id == v.id) {
                v.input = *e;
            } else {
                v.input.gain = 0.;
                v.input.siren = false;
            }
        }
        for e in scene.engines.iter().filter(|e| e.gain > 0.) {
            if self.voices.iter().any(|v| v.id == e.id) {
                continue;
            }
            if let Some(v) = self
                .voices
                .iter_mut()
                .find(|v| v.input.gain == 0. && v.gain + v.siren_gain < 0.0001)
            {
                v.id = e.id;
                v.input = *e;
            }
        }
        let table = &self.table;
        for frame in out.chunks_exact_mut(2) {
            let (mut left, mut right) = (0., 0.);
            for v in self.voices.iter_mut() {
                let e = v.input;
                let (idle, redline, pulses, color, shift, electric) = PROFILES[e.kind.min(7)];
                let target = if scene.active {
                    safe(e.gain, 0., 0.3)
                } else {
                    0.
                };
                v.gain += (target - v.gain) * 0.0025;
                v.siren_gain += ((if scene.active && e.siren {
                    safe(e.gain, 0., 0.3)
                } else {
                    0.
                }) - v.siren_gain)
                    * 0.002;
                if target == 0. && v.gain + v.siren_gain < 0.00001 {
                    continue;
                }
                let speed = safe(e.speed.abs() / e.maximum.max(1.), 0., 1.);
                let gear = (speed * 5.).min(4.999);
                let load = safe(e.throttle.abs(), 0., 1.);
                let rpm = idle
                    + (redline - idle)
                        * ((gear.fract() * 0.65 + load * 0.23 + speed * 0.12).clamp(0., 1.));
                // Lower the whole engine register while preserving each voice's harmonics.
                let hz = 0.7
                    * if electric {
                        120. + speed * 1400.
                    } else {
                        rpm / 60. * pulses
                    };
                v.hz += (hz - v.hz) * 0.0008;
                v.pan += (safe(e.pan, -1., 1.) - v.pan) * 0.002;
                advance(&mut v.phase, v.hz);
                advance(&mut v.modulation, v.hz / pulses.max(1.) * 0.5);
                let wobble = wave(table, v.modulation) * if electric { 0.02 } else { 0.16 };
                let p = v.phase + wobble;
                let engine = (wave(table, p)
                    + color * wave(table, p * 2. + shift)
                    + 0.18 * wave(table, p * 3. + FRAC_PI_2)
                    + 0.07 * wave(table, p * 4. + shift * 2.))
                    * (if electric {
                        speed * 0.6 + load * 0.4
                    } else {
                        0.55 + load * 0.45
                    });
                advance(&mut v.siren_sweep, 0.65);
                let sweep = 0.7 * (850. + 350. * wave(table, v.siren_sweep));
                advance(&mut v.siren_phase, sweep);
                let siren =
                    wave(table, v.siren_phase) + 0.2 * wave(table, v.siren_phase * 2. + FRAC_PI_2);
                let target_rate = 0.6 + (rpm - idle) / (redline - idle) * 0.85;
                v.clip_rate += (target_rate - v.clip_rate) * 0.0008;
                let recorded =
                    clips::engine(e.kind.min(7), e.id, &mut v.clip_position, v.clip_rate);
                let motor = if electric {
                    engine * 0.65
                } else {
                    recorded * (0.75 + load * 0.35) + engine * 0.12
                };
                let sample = motor * v.gain + siren * v.siren_gain * 0.65;
                left += sample * (0.7 - 0.3 * v.pan);
                right += sample * (0.7 + 0.3 * v.pan);
            }
            if self.foot < 0.22 {
                let t = self.foot;
                let base = match self.foot_surface {
                    Surface::Pavement => 150.,
                    Surface::Grass => 95.,
                    Surface::Metal => 340.,
                    Surface::Water => 430.,
                };
                advance(
                    &mut self.foot_phase,
                    base * 0.3 * (1. + (-t * 65.).exp() * 1.8),
                );
                let attack = (t * 900.).min(1.);
                let envelope = attack * (-t * 32.).exp() * 0.075;
                let s = (wave(table, self.foot_phase)
                    + 0.45 * wave(table, self.foot_phase * 2.73 + FRAC_PI_2)
                    + 0.18 * wave(table, self.foot_phase * 5.1 + 0.8))
                    * envelope;
                left += s * (0.7 + self.foot_side * 0.08);
                right += s * (0.7 - self.foot_side * 0.08);
                self.foot += 1. / RATE as f32;
            }
            let (step_left, step_right) = self.steps.render();
            left += step_left;
            right += step_right;
            // Fixed headroom and a soft limiter protect mixed emergency scenes.
            frame[0] = left / (1. + left.abs());
            frame[1] = right / (1. + right.abs());
        }
        if out.len() % 2 == 1 {
            if let Some(last) = out.last_mut() {
                *last = 0.;
            }
        }
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn voices_are_distinct_bounded_and_deterministic() {
        let mut signatures = Vec::new();
        for kind in 0..8 {
            let mut scene = Scene {
                active: true,
                ..Default::default()
            };
            scene.engines[0] = Engine {
                id: 1,
                kind,
                gain: 0.25,
                speed: 22.,
                maximum: 60.,
                throttle: 0.8,
                ..Default::default()
            };
            let mut a = Synth::default();
            let mut b = Synth::default();
            let mut x = vec![0.; RATE as usize];
            let mut y = x.clone();
            a.render(&scene, &mut x);
            b.render(&scene, &mut y);
            assert_eq!(x, y);
            assert!(x.iter().all(|v| v.is_finite() && v.abs() < 1.));
            assert!(x.iter().any(|v| v.abs() > 0.02));
            signatures.push(x[1000..1100].to_vec());
        }
        for i in 0..8 {
            for j in 0..i {
                assert_ne!(signatures[i], signatures[j]);
            }
        }
    }
    #[test]
    fn steps_decay_and_silence_stays_silent() {
        let mut s = Synth::default();
        let scene = Scene::default();
        let mut out = vec![0.; RATE as usize * 4];
        s.render(&scene, &mut out);
        assert!(out.iter().all(|v| *v == 0.));
        s.step(Surface::Pavement);
        s.render(&scene, &mut out);
        assert!(out.iter().any(|v| v.abs() > 0.02));
        assert!(out[out.len() - 100..].iter().all(|v| *v == 0.));
    }
    #[test]
    fn siren_pans_and_streaming_is_block_size_independent() {
        let mut scene = Scene {
            active: true,
            ..Default::default()
        };
        scene.engines[0] = Engine {
            id: 9,
            gain: 0.2,
            pan: 1.,
            siren: true,
            ..Default::default()
        };
        let mut a = Synth::default();
        let mut b = Synth::default();
        let mut x = vec![0.; 4096];
        let mut y = x.clone();
        a.render(&scene, &mut x);
        for block in y.chunks_mut(512) {
            b.render(&scene, block);
        }
        assert_eq!(x, y);
        let l: f32 = x.iter().step_by(2).map(|x| x * x).sum();
        let r: f32 = x.iter().skip(1).step_by(2).map(|x| x * x).sum();
        assert!(r > l * 1.5);
    }
}
#[cfg(test)]
mod continuity_tests {
    use super::*;
    #[test]
    fn distance_ranking_changes_do_not_restart_oscillators() {
        let mut scene = Scene {
            active: true,
            ..Default::default()
        };
        scene.engines[0] = Engine {
            id: 10,
            kind: 0,
            gain: 0.2,
            maximum: 60.,
            speed: 20.,
            ..Default::default()
        };
        scene.engines[1] = Engine {
            id: 20,
            kind: 6,
            gain: 0.15,
            maximum: 72.,
            speed: 40.,
            ..Default::default()
        };
        let mut a = Synth::default();
        let mut b = Synth::default();
        let mut x = vec![0.; 2048];
        let mut y = x.clone();
        a.render(&scene, &mut x);
        b.render(&scene, &mut y);
        a.render(&scene, &mut x);
        scene.engines.swap(0, 1);
        b.render(&scene, &mut y);
        assert_eq!(x, y);
    }
}

#[cfg(test)]
mod phase_boundary_tests {
    use super::*;
    #[test]
    fn phase_wrap_handles_negative_roundoff() {
        let s = Synth::default();
        for p in [-f32::MIN_POSITIVE, -1e-8, -TAU, 0., TAU, TAU * 4.] {
            let v = wave(&s.table, p);
            assert!(v.is_finite() && v.abs() < 0.001);
        }
    }
}
