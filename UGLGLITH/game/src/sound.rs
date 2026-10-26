impl Game {
    fn track_footsteps(&mut self, previous: Vec3, sprint: bool) {
        let ground = self.world.walking_surface(self.player.pos) + 1.;
        if self.driving.is_some()
            || self.heists.riding.is_some()
            || (self.player.pos.y - ground).abs() > 0.15
            || self.player.vy.abs() > 0.5
        {
            self.foot_distance = 0.;
            return;
        }
        let d = self.player.pos.distance(previous);
        if d > 1. {
            self.foot_distance = 0.;
            return;
        } // Teleports and respawns never become footsteps.
        self.foot_distance += d;
        let stride = if self.stance > 0 {
            1.1
        } else if sprint {
            2.2
        } else {
            1.65
        };
        if self.foot_distance >= stride {
            self.foot_distance -= stride;
            self.foot_events = self.foot_events.saturating_add(1).min(2);
        }
    }
    fn sound_scene(&mut self) -> procedural_audio::Scene {
        use procedural_audio::{Engine, Scene, Surface};
        let steps = std::mem::take(&mut self.foot_events);
        if self.modal() {
            return Scene::default();
        }
        let p = self.player.pos;
        let mut scene = Scene {
            active: true,
            glitch: self.glitches.parameters()[0],
            absurd_mode: self.chaos.absurd.audio(),
            steps,
            ..Default::default()
        };
        for (i, c) in self.cars.iter().enumerate() {
            let driver = self.traffic.drivers.iter().find(|d| d.car == i);
            let own = self.driving == Some(i);
            let siren = driver.is_some_and(|d| d.police && d.incident);
            let running = own || driver.is_some_and(|d| !d.parked);
            if !running && !siren {
                continue;
            }
            let distance = (p.distance(c.pos).powi(2) + (p.y - c.pos.y).powi(2)).sqrt();
            if distance > 100. {
                continue;
            }
            let gain = if own {
                0.23
            } else {
                0.2 / (1. + (distance / 13.).powi(2))
            };
            let pan = if own {
                0.
            } else {
                ((c.pos.x - p.x) * self.yaw.cos() + (c.pos.z - p.z) * self.yaw.sin())
                    / distance.max(1.)
            };
            let engine = Engine {
                id: i as u32,
                kind: c.kind,
                speed: c.speed,
                maximum: CATALOG[c.kind].max_speed,
                throttle: if own {
                    self.sound_throttle
                } else if running {
                    0.35
                } else {
                    0.
                },
                gain,
                pan,
                siren,
            };
            if let Some(at) = scene.engines.iter().position(|e| e.gain < engine.gain) {
                for j in (at + 1..scene.engines.len()).rev() {
                    scene.engines[j] = scene.engines[j - 1];
                }
                scene.engines[at] = engine;
            }
        }
        scene.surface = if steps == 0 {
            Surface::Pavement
        } else if self.world.water_depth(p) > 0.3 {
            Surface::Water
        } else if terrain::railway::nearest(p).0 < 3. {
            Surface::Metal
        } else if self.world.interior(p).is_some()
            || self
                .world
                .roads
                .iter()
                .any(|r| r.distance(p).0 < r.width / 2. + 2.)
        {
            Surface::Pavement
        } else {
            Surface::Grass
        };
        scene
    }
}
#[cfg(test)]
mod sound_tests {
    use super::*;
    #[test]
    fn footsteps_require_grounded_motion_and_are_consumed_once() {
        let mut g = Game::from_save(Default::default()).unwrap();
        g.menu = false;
        g.player.pos = Vec3::new(80., 1., 0.);
        for _ in 0..12 {
            let old = g.player.pos;
            g.player.pos.x += 0.2;
            g.track_footsteps(old, false);
        }
        assert!(g.sound_scene().steps > 0);
        assert_eq!(g.sound_scene().steps, 0);
        g.player.pos.y += 3.;
        for _ in 0..20 {
            let old = g.player.pos;
            g.player.pos.x += 0.2;
            g.track_footsteps(old, false);
        }
        assert_eq!(g.sound_scene().steps, 0);
    }
    #[test]
    fn distant_cars_are_inaudible_and_menus_mute() {
        let mut g = Game::from_save(Default::default()).unwrap();
        g.menu = false;
        g.driving = Some(0);
        g.player.pos = g.cars[0].pos;
        let scene = g.sound_scene();
        assert!(scene.engines.iter().any(|e| e.id == 0 && e.gain > 0.2));
        g.driving = None;
        g.player.pos = Vec3::new(10000., 1., 10000.);
        assert!(g.sound_scene().engines.iter().all(|e| e.gain == 0.));
        g.menu = true;
        assert!(!g.sound_scene().active);
    }
    #[test]
    fn bursts_reach_audio_and_pause_in_menus() {
        let mut g = Game::from_save(Default::default()).unwrap();
        g.menu = false;
        g.glitches.burst();
        g.glitches.tick(0.2);
        assert_eq!(g.sound_scene().glitch, 1.);
        let before = g.glitches.parameters();
        g.menu = true;
        g.simulate(0., 0., false, false, false);
        assert_eq!(g.glitches.parameters(), before);
        assert_eq!(g.sound_scene().glitch, 0.);
    }
}
#[cfg(test)]
mod siren_tests {
    use super::*;
    #[test]
    fn only_dispatched_police_use_sirens() {
        let mut g = Game::from_save(Default::default()).unwrap();
        g.menu = false;
        let d = g.traffic.drivers.iter_mut().find(|d| d.police).unwrap();
        let id = d.car;
        d.incident = true;
        g.player.pos = g.cars[id].pos;
        assert!(g
            .sound_scene()
            .engines
            .iter()
            .any(|e| e.id == id as u32 && e.siren));
        g.traffic
            .drivers
            .iter_mut()
            .find(|d| d.car == id)
            .unwrap()
            .incident = false;
        assert!(g.sound_scene().engines.iter().all(|e| !e.siren));
    }
}
