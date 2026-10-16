//! Bounded, pointer-owned touch input in the game's 1024x576 logical UI space.
use std::cell::Cell;
use std::collections::VecDeque;
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Action {
    Jump,
    Interact,
    Fire,
    Vehicle,
    Crouch,
    Prone,
    Sprint,
    Map,
    Inventory,
    Menu,
    Reload,
    Parachute,
    Slot(u8),
}
#[derive(Clone, Copy)]
pub struct Button {
    pub action: Action,
    pub rect: [f32; 4],
    pub label: &'static str,
}
pub const BUTTONS: [Button; 12] = [
    Button {
        action: Action::Jump,
        rect: [930., 435., 78., 60.],
        label: "JUMP",
    },
    Button {
        action: Action::Interact,
        rect: [840., 435., 78., 60.],
        label: "USE",
    },
    Button {
        action: Action::Fire,
        rect: [930., 360., 78., 60.],
        label: "FIRE",
    },
    Button {
        action: Action::Vehicle,
        rect: [840., 510., 168., 54.],
        label: "CAR / EXIT",
    },
    Button {
        action: Action::Crouch,
        rect: [660., 435., 78., 60.],
        label: "CROUCH",
    },
    Button {
        action: Action::Prone,
        rect: [660., 360., 78., 60.],
        label: "CRAWL",
    },
    Button {
        action: Action::Sprint,
        rect: [28., 320., 90., 48.],
        label: "RUN",
    },
    Button {
        action: Action::Map,
        rect: [430., 130., 65., 44.],
        label: "MAP",
    },
    Button {
        action: Action::Inventory,
        rect: [500., 130., 65., 44.],
        label: "BAG",
    },
    Button {
        action: Action::Menu,
        rect: [570., 130., 65., 44.],
        label: "MENU",
    },
    Button {
        action: Action::Reload,
        rect: [750., 435., 78., 60.],
        label: "RELOAD",
    },
    Button {
        action: Action::Parachute,
        rect: [750., 360., 78., 60.],
        label: "CHUTE",
    },
];
#[derive(Clone, Copy, Debug)]
enum Role {
    Stick,
    Look,
    Button(Action),
}
#[derive(Clone, Copy, Debug)]
struct Finger {
    id: i64,
    role: Role,
    last: [f32; 2],
}
#[derive(Default)]
pub struct TouchControls {
    fingers: Vec<Finger>,
    pub movement: [f32; 2],
    look: [f32; 2],
    pressed: VecDeque<Action>,
    jump_pending: Cell<bool>,
}
impl TouchControls {
    pub fn clear(&mut self) {
        self.fingers.clear();
        self.movement = [0.; 2];
        self.look = [0.; 2];
        self.pressed.clear();
        self.jump_pending.set(false);
    }
    pub fn down(&mut self, id: i64, x: f32, y: f32) {
        if !x.is_finite()
            || !y.is_finite()
            || self.fingers.len() >= 10
            || self.fingers.iter().any(|f| f.id == id)
        {
            return;
        }
        let p = [x * 1024., y * 576.];
        let role = if let Some(b) = BUTTONS.iter().find(|b| contains(b.rect, p)) {
            Role::Button(b.action)
        } else if p[0] < 220.
            && p[1] > 300.
            && !self.fingers.iter().any(|f| matches!(f.role, Role::Stick))
        {
            Role::Stick
        } else if (496. ..=558.).contains(&p[1]) && (222. ..790.).contains(&p[0]) {
            Role::Button(Action::Slot(((p[0] - 222.) / 82.).floor() as u8 + 1))
        } else if p[0] > 300. && !self.fingers.iter().any(|f| matches!(f.role, Role::Look)) {
            Role::Look
        } else {
            return;
        };
        if let Role::Button(a) = role {
            if a == Action::Jump {
                self.jump_pending.set(true);
            }
            if self.pressed.len() < 32 {
                self.pressed.push_back(a);
            }
        }
        self.fingers.push(Finger { id, role, last: p });
        self.motion(id, x, y);
    }
    pub fn motion(&mut self, id: i64, x: f32, y: f32) {
        if !x.is_finite() || !y.is_finite() {
            return;
        }
        if let Some(f) = self.fingers.iter_mut().find(|f| f.id == id) {
            let p = [x * 1024., y * 576.];
            match f.role {
                Role::Stick => {
                    let dx = (p[0] - 150.) / 68.;
                    let dy = (p[1] - 445.) / 68.;
                    let len = dx.hypot(dy);
                    let scale = if len < 0.12 {
                        0.
                    } else {
                        ((len - 0.12) / 0.88).min(1.) / len
                    };
                    self.movement = [dx * scale, -dy * scale];
                }
                Role::Look => {
                    self.look[0] += (p[0] - f.last[0]).clamp(-256., 256.);
                    self.look[1] += (p[1] - f.last[1]).clamp(-144., 144.);
                }
                Role::Button(_) => {}
            }
            f.last = p;
        }
    }
    pub fn up(&mut self, id: i64) {
        if self
            .fingers
            .iter()
            .any(|f| f.id == id && matches!(f.role, Role::Stick))
        {
            self.movement = [0.; 2];
        }
        self.fingers.retain(|f| f.id != id);
    }
    pub fn held(&self, a: Action) -> bool {
        self.fingers
            .iter()
            .any(|f| matches!(f.role,Role::Button(b) if b==a))
    }
    pub fn take_jump(&self) -> bool {
        self.jump_pending.replace(false)
    }
    pub fn pop(&mut self) -> Option<Action> {
        self.pressed.pop_front()
    }
    pub fn take_look(&mut self) -> [f32; 2] {
        std::mem::replace(&mut self.look, [0.; 2])
    }
}
fn contains(r: [f32; 4], p: [f32; 2]) -> bool {
    p[0] >= r[0] && p[0] <= r[0] + r[2] && p[1] >= r[1] && p[1] <= r[1] + r[3]
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn simultaneous_move_look_and_jump() {
        let mut c = TouchControls::default();
        c.down(1, 150. / 1024., 390. / 576.);
        c.down(2, 0.6, 0.5);
        c.down(3, 970. / 1024., 460. / 576.);
        c.motion(2, 0.65, 0.5);
        assert!(c.movement[1] > 0.5);
        assert!(c.held(Action::Jump));
        assert!(c.take_look()[0] > 40.);
        c.up(2);
        assert!(c.movement[1] > 0.5);
        c.up(1);
        assert_eq!(c.movement, [0.; 2]);
    }
    #[test]
    fn cancelled_contacts_never_leave_actions_stuck() {
        let mut c = TouchControls::default();
        c.down(8, 880. / 1024., 460. / 576.);
        assert!(c.held(Action::Interact));
        c.clear();
        assert!(!c.held(Action::Interact));
        assert_eq!(c.pop(), None);
    }
    #[test]
    fn release_is_owned_by_finger_and_press_is_once() {
        let mut c = TouchControls::default();
        c.down(4, 970. / 1024., 390. / 576.);
        c.motion(4, 0.2, 0.2);
        assert_eq!(c.pop(), Some(Action::Fire));
        assert_eq!(c.pop(), None);
        c.up(3);
        assert!(c.held(Action::Fire));
        c.up(4);
        assert!(!c.held(Action::Fire));
    }
    #[test]
    fn joystick_is_clamped_and_dead_zone_is_stable() {
        let mut c = TouchControls::default();
        c.down(1, 150. / 1024., 445. / 576.);
        assert_eq!(c.movement, [0.; 2]);
        c.motion(1, -2., 3.);
        assert!(c.movement[0].hypot(c.movement[1]) <= 1.0001);
    }
    #[test]
    fn contacts_and_queue_are_bounded() {
        let mut c = TouchControls::default();
        for i in 0..100 {
            c.down(i, 0.95, 0.8);
        }
        assert!(c.fingers.len() <= 10);
        assert!(c.pressed.len() <= 32);
    }
    #[test]
    fn short_jump_tap_survives_release_until_the_next_simulation_step() {
        let mut c = TouchControls::default();
        c.down(1, 970. / 1024., 460. / 576.);
        c.up(1);
        assert!(!c.held(Action::Jump));
        assert!(c.take_jump());
        assert!(!c.take_jump());
        c.down(2, 970. / 1024., 460. / 576.);
        c.clear();
        assert!(!c.take_jump());
    }
}
