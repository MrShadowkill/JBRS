use engine_core::Vec3;
#[derive(Clone, Copy)]
pub struct Site {
    pub name: &'static str,
    pub x: f32,
    pub z: f32,
    pub reward: u32,
    pub questions: u32,
    pub delay: f32,
}
pub const SITES: [Site; 6] = [
    Site {
        name: "Jewelry",
        x: 260.,
        z: 190.,
        reward: 320,
        questions: 0,
        delay: 240.,
    },
    Site {
        name: "Museum",
        x: -200.,
        z: -630.,
        reward: 400,
        questions: 1,
        delay: 240.,
    },
    Site {
        name: "Power Plant",
        x: 720.,
        z: -170.,
        reward: 450,
        questions: 7,
        delay: 180.,
    },
    Site {
        name: "Casino",
        x: 370.,
        z: 240.,
        reward: 500,
        questions: 5,
        delay: 180.,
    },
    Site {
        name: "Gas Station",
        x: -650.,
        z: 40.,
        reward: 70,
        questions: 0,
        delay: 60.,
    },
    Site {
        name: "Freight Train",
        x: 0.,
        z: 0.,
        reward: 350,
        questions: 0,
        delay: 180.,
    },
];
#[derive(Default)]
pub struct Heists {
    pub site: Option<usize>,
    pub cooldown: [f32; 6],
    pub elapsed: f32,
    pub progress: f32,
    pub bag: u32,
    pub solved: u32,
    pub quiz: bool,
    pub answer: String,
    pub expression: String,
    pub expected: i32,
    pub dispatched: bool,
    pub strikes: u32,
    pub bomb: f32,
    pub opened: bool,
    pub train_gold: [usize; 3],
    pub train_count: usize,
    pub riding: Option<usize>,
    random: u64,
}
impl Heists {
    pub fn new(seed: u64) -> Self {
        let mut s = Self {
            random: seed,
            ..Default::default()
        };
        s.train_count = (s.next() % 3 + 1) as usize;
        for i in 0..s.train_count {
            loop {
                let n = s.next() as usize % 25;
                if !s.train_gold[..i].contains(&n) {
                    s.train_gold[i] = n;
                    break;
                }
            }
        }
        s
    }
    fn next(&mut self) -> u64 {
        self.random = self
            .random
            .wrapping_mul(6364136223846793005)
            .wrapping_add(1442695040888963407);
        self.random >> 32
    }
    pub fn start(&mut self, id: usize) -> bool {
        if id >= 6 || self.site.is_some() || self.bag > 0 || self.cooldown[id] > 0. {
            return false;
        }
        self.site = Some(id);
        self.elapsed = 0.;
        self.progress = 0.;
        self.solved = 0;
        self.strikes = 0;
        self.dispatched = false;
        self.opened = id != 5;
        true
    }
    pub fn question(&mut self) {
        let a = (self.next() % 12 + 2) as i32;
        let x = (self.next() % 20 + 1) as i32;
        let b = (self.next() % 30 + 1) as i32;
        self.expected = x;
        self.expression = match self.next() % 3 {
            0 => format!("{} * x + {} = {}", a, b, a * x + b),
            1 => {
                self.expected = x * a;
                format!("x / {} + {} = {}", a, b, x + b)
            }
            _ => format!("x + {} = {}", b, x + b),
        };
        self.answer.clear();
        self.quiz = true;
    }
    pub fn submit(&mut self) -> bool {
        if self.answer.parse::<i32>().ok() != Some(self.expected) {
            self.answer.clear();
            return false;
        }
        self.solved += 1;
        let n = self.site.map(|id| SITES[id].questions).unwrap_or(0);
        if self.solved >= n {
            self.quiz = false;
        } else {
            self.question();
        }
        true
    }
    pub fn tick(&mut self, dt: f32) {
        for c in &mut self.cooldown {
            *c = (*c - dt).max(0.);
        }
        if self.site.is_some() {
            self.elapsed += dt;
        }
        if self.bomb > 0. {
            self.bomb = (self.bomb - dt).max(0.);
            if self.bomb == 0. {
                self.opened = true;
            }
        }
    }
    pub fn finish(&mut self) {
        if let Some(id) = self.site {
            self.cooldown[id] = 300. + (self.next() % 601) as f32;
        }
        self.site = None;
        self.quiz = false;
        self.progress = 0.;
        self.strikes = 0;
        self.riding = None;
    }
    pub fn train(clock: f32, wagon: usize) -> (Vec3, f32) {
        let a = clock * 0.014 - wagon as f32 * 0.013;
        let x = 800. * a.cos();
        let z = 750. * a.sin();
        let y = terrain::height(x, z).max(0.) + 1.;
        (Vec3::new(x, y, z), (-800. * a.sin()).atan2(750. * a.cos()))
    }
    pub fn gold(&self, wagon: usize) -> bool {
        self.train_gold[..self.train_count].contains(&wagon)
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn questions_are_solvable_and_gate_completion() {
        let mut h = Heists::new(9);
        h.start(2);
        for _ in 0..7 {
            h.question();
            h.answer = "-1".into();
            assert!(!h.submit());
            h.answer = h.expected.to_string();
            assert!(h.submit());
        }
        assert_eq!(h.solved, 7);
        assert!(!h.quiz);
    }
    #[test]
    fn cooldown_and_gold() {
        for seed in 1..20 {
            let mut h = Heists::new(seed);
            assert!((1..=3).contains(&h.train_count));
            assert_eq!((0..25).filter(|i| h.gold(*i)).count(), h.train_count);
            assert!(h.start(5));
            h.finish();
            assert!((300. ..=900.).contains(&h.cooldown[5]));
            assert!(!h.start(5));
        }
    }
}
