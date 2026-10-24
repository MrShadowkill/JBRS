impl Game {
    fn touch_event(
        &mut self,
        e: &renderer_gl21::Event,
        touch: &mut mobile_controls::TouchControls,
    ) -> bool {
        match e.kind() {
            0x700..=0x702 => {
                let (id, x, y) = e.finger();
                if e.kind() == 0x701 {
                    touch.up(id);
                    return true;
                }
                if e.kind() == 0x702 {
                    touch.motion(id, x, y);
                    return true;
                }
                if !self.modal() && self.touch_shop() && (285. ..335.).contains(&(y * 576.)) {
                    if (660. ..738.).contains(&(x * 1024.)) {
                        touch.clear();
                        self.supplies(true);
                        return true;
                    }
                    if (750. ..828.).contains(&(x * 1024.)) {
                        touch.clear();
                        self.supplies(false);
                        return true;
                    }
                }
                if self.garage && (430. ..480.).contains(&(y * 576.)) {
                    let delta = if (206. ..326.).contains(&(x * 1024.)) {
                        Some(0.8)
                    } else if (698. ..818.).contains(&(x * 1024.)) {
                        Some(1.25)
                    } else {
                        None
                    };
                    if let Some(delta) = delta {
                        self.save.sensitivity = (self.save.sensitivity * delta).clamp(0.0005, 0.02);
                        self.save();
                        touch.clear();
                        return true;
                    }
                }
                if (378. ..680.).contains(&(x * 1024.)) && (14. ..72.).contains(&(y * 576.)) {
                    touch.clear();
                    self.ui_click(x * 1024., y * 576.);
                    return true;
                }
                if self.heists.quiz {
                    touch.clear();
                    if let Some(key) = crate::mobile::quiz_key(x * 1024., y * 576.) {
                        self.key_action(key);
                    }
                } else if self.modal()
                    || (self.cursor
                        && (!(430. ..635.).contains(&(x * 1024.))
                            || !(130. ..174.).contains(&(y * 576.))))
                {
                    touch.clear();
                    self.ui_click(x * 1024., y * 576.);
                } else {
                    touch.down(id, x, y);
                }
                true
            }
            0x400..=0x402 if e.touch_mouse() => true,
            0x300 if e.key() == 1073742094 => {
                if !e.repeat() {
                    touch.clear();
                    self.key_action(27);
                }
                true
            }
            _ => false,
        }
    }
    fn touch_actions(&mut self, touch: &mut mobile_controls::TouchControls) {
        use mobile_controls::Action;
        while let Some(a) = touch.pop() {
            let key = match a {
                Action::Interact | Action::Vehicle => Some(101),
                Action::Crouch => {
                    if !self.modal() {
                        self.crouched = !self.crouched;
                        self.prone = false;
                    }
                    None
                }
                Action::Fire => {
                    if !self.modal() {
                        self.fire();
                    }
                    None
                }
                Action::Prone => Some(122),
                Action::Map => Some(109),
                Action::Menu => Some(27),
                Action::Inventory => Some(9),
                Action::Reload => Some(114),
                Action::Parachute => Some(112),
                Action::Slot(n) => Some(48 + i32::from(n)),
                Action::Jump if self.cuffs.active => Some(32),
                _ => None,
            };
            if let Some(k) = key {
                self.key_action(k);
            }
            if self.modal() || self.cursor {
                touch.clear();
                break;
            }
        }
    }
    fn touch_shop(&self) -> bool {
        self.world.interior(self.player.pos).is_some_and(|id| {
            matches!(
                self.world.buildings[id].name,
                "Grocery" | "Donut Shop" | "Gas Station"
            )
        })
    }
    fn draw_touch(&self, r: &mut Renderer) {
        use mobile_controls::{Action, BUTTONS};
        if !self.modal() && self.touch_shop() {
            for (x, label) in [(660., "WATER"), (750., "FOOD")] {
                r.panel([x, 285., 78., 50.], [0.02, 0.18, 0.27], 0.95, 12.);
                r.label(x + 6., 300., 1.8, label, [1.; 3]);
            }
        }
        if self.garage {
            for (x, label) in [(206., "SENS -"), (698., "SENS +")] {
                r.panel([x, 430., 120., 50.], [0.02, 0.18, 0.27], 0.95, 12.);
                r.label(x + 12., 445., 1.7, label, [1.; 3]);
            }
            r.label(
                370.,
                445.,
                1.7,
                &format!("TOUCH {:.2}x", self.save.sensitivity / 0.003),
                [0.8, 0.95, 1.],
            );
        }
        if self.heists.quiz {
            r.panel([335., 200., 350., 310.], [0.02, 0.06, 0.09], 0.94, 18.);
            for (i, label) in [
                "1", "2", "3", "4", "5", "6", "7", "8", "9", "DEL", "0", "OK",
            ]
            .iter()
            .enumerate()
            {
                let x = 350. + (i % 3) as f32 * 105.;
                let y = 215. + (i / 3) as f32 * 70.;
                r.panel([x, y, 95., 60.], [0.08, 0.3, 0.42], 0.95, 10.);
                r.label(x + 20., y + 15., 2., label, [1.; 3]);
            }
            return;
        }
        for b in BUTTONS {
            if self.modal()
                || (self.cursor
                    && !matches!(b.action, Action::Map | Action::Menu | Action::Inventory))
            {
                continue;
            }
            r.panel(b.rect, [0.015, 0.08, 0.13], 0.72, 14.);
            r.label(
                b.rect[0] + 8.,
                b.rect[1] + 15.,
                1.8,
                b.label,
                [0.85, 0.96, 1.],
            );
        }
        if !self.modal() && !self.cursor {
            r.panel([75., 370., 150., 150.], [0.02, 0.13, 0.2], 0.55, 75.);
            r.panel(
                [
                    126. + self.mobile_stick[0] * 44.,
                    421. - self.mobile_stick[1] * 44.,
                    48.,
                    48.,
                ],
                [0.2, 0.65, 0.85],
                0.8,
                24.,
            );
            r.label(
                90.,
                535.,
                1.1,
                if self.driving.is_some() {
                    "STEER / GAS"
                } else {
                    "MOVE"
                },
                [0.8, 0.95, 1.],
            );
        }
    }
}

#[cfg(test)]
mod touch_tests {
    use super::*;
    use mobile_controls::{Action, TouchControls};
    fn finger(kind: u32, id: i64, x: f32, y: f32) -> renderer_gl21::Event {
        let mut e = renderer_gl21::Event { data: [0; 56] };
        e.data[0..4].copy_from_slice(&kind.to_ne_bytes());
        e.data[16..24].copy_from_slice(&id.to_ne_bytes());
        e.data[24..28].copy_from_slice(&x.to_ne_bytes());
        e.data[28..32].copy_from_slice(&y.to_ne_bytes());
        e
    }
    #[test]
    fn mobile_event_routes_menu_tap_then_owns_and_releases_movement() {
        let mut g = Game::from_save(Default::default()).unwrap();
        let mut touch = TouchControls::default();
        assert!(g.menu);
        // The prisoner's menu choice uses normalized SDL touch coordinates.
        g.touch_event(&finger(0x700, 1, 360. / 1024., 300. / 576.), &mut touch);
        assert!(!g.menu);
        assert_eq!(g.player.team, Team::Prisoner);
        g.touch_event(&finger(0x700, 2, 150. / 1024., 390. / 576.), &mut touch);
        g.touch_event(&finger(0x700, 3, 970. / 1024., 460. / 576.), &mut touch);
        assert!(touch.movement[1] > 0.5);
        assert!(touch.held(Action::Jump));
        g.touch_event(&finger(0x701, 2, 150. / 1024., 390. / 576.), &mut touch);
        assert_eq!(touch.movement, [0.; 2]);
        assert!(touch.held(Action::Jump));
    }
    #[test]
    fn hotbar_last_slot_and_crouch_are_available_without_keyboard() {
        let mut g = Game::from_save(Default::default()).unwrap();
        g.menu = false;
        let mut touch = TouchControls::default();
        g.touch_event(&finger(0x700, 1, 760. / 1024., 530. / 576.), &mut touch);
        g.touch_actions(&mut touch);
        assert_eq!(g.slot, 6);
        g.touch_event(&finger(0x701, 1, 0., 0.), &mut touch);
        g.touch_event(&finger(0x700, 2, 690. / 1024., 460. / 576.), &mut touch);
        g.touch_actions(&mut touch);
        assert!(g.crouched);
    }
    #[test]
    fn quiz_can_enter_and_delete_digits_with_touch() {
        let mut g = Game::from_save(Default::default()).unwrap();
        g.menu = false;
        g.heists.quiz = true;
        let mut touch = TouchControls::default();
        g.touch_event(&finger(0x700, 1, 380. / 1024., 240. / 576.), &mut touch);
        assert_eq!(g.heists.answer, "1");
        g.touch_event(&finger(0x700, 2, 380. / 1024., 445. / 576.), &mut touch);
        assert_eq!(g.heists.answer, "");
    }
    #[test]
    fn android_back_closes_panels_and_clears_held_contacts() {
        let mut g = Game::from_save(Default::default()).unwrap();
        let mut touch = TouchControls::default();
        touch.down(1, 150. / 1024., 390. / 576.);
        let e = renderer_gl21::Event::synthetic_key(270, 0);
        assert!(g.touch_event(&e, &mut touch));
        assert!(!g.menu);
        assert_eq!(touch.movement, [0.; 2]);
    }
    #[test]
    fn quick_fire_tap_consumes_one_item_and_shop_buttons_preserve_crouch() {
        let mut g = Game::from_save(Default::default()).unwrap();
        g.menu = false;
        g.slot = 3;
        g.save.water = 3;
        let mut touch = TouchControls::default();
        g.touch_event(&finger(0x700, 1, 970. / 1024., 390. / 576.), &mut touch);
        g.touch_event(&finger(0x701, 1, 970. / 1024., 390. / 576.), &mut touch);
        g.touch_actions(&mut touch);
        assert_eq!(g.save.water, 2);
        g.player.pos = Vec3::new(210., 1., 244.);
        g.save.balance = 20;
        g.touch_event(&finger(0x700, 2, 790. / 1024., 310. / 576.), &mut touch);
        assert_eq!(g.save.food, 1);
        assert_eq!(g.save.balance, 8);
        g.touch_event(&finger(0x700, 3, 690. / 1024., 460. / 576.), &mut touch);
        g.touch_actions(&mut touch);
        assert!(g.crouched);
        assert_eq!(g.save.food, 1);
        assert_eq!(g.save.balance, 8);
    }
}
